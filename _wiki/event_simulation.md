# Event-Driven Digital Simulation (rhdl crate)

## 什麼是數位模擬

數位模擬（digital simulation）是用軟體來建模數位電路的行為。給定一組輸入訊號的變化，模擬器計算出電路中所有閘（gate）與觸發器（flip-flop）的輸出，使開發者能在沒有實際 FPGA 或 ASIC 的情況下驗證邏輯正確性。eda4 專案的 `verilog2rust/src/rhdl/` 實作了一個輕量級的事件驅動模擬引擎，專供轉換後的 Verilog 設計執行。

## 事件驅動 vs. 循環式 vs. 編譯式模擬

- **事件驅動（event-driven）**：只在訊號發生變化時才重新評估受影響的閘。rhdl 採用此方法──每個閘的 `eval()` 檢查輸出是否改變，若無改變則停止迭代。
- **循環式（cycle-based）**：每個時脈週期只計算一次所有閘的穩定狀態，不追蹤週期內的細部變化。速度較快，但無法建模非同步邏輯或時序細節。
- **編譯式（compiled-code）**：將設計編譯成主機機器碼直接執行（如 Verilator）。rhdl 不走此路線，而是透過 Rust 的泛型與閉包在執行期動態評估。

## 四值邏輯（Level）

rhdl 使用標準的四值邏輯系統來表示數位訊號的狀態，定義在 `signal.rs:8-14`：

| 列舉值 | 意義 | 顯示 |
|--------|------|------|
| `Level::L` | 邏輯 0（低電位） | `0` |
| `Level::H` | 邏輯 1（高電位） | `1` |
| `Level::X` | 未知（unknown） | `X` |
| `Level::Z` | 高阻抗（high-impedance） | `Z` |

`Level::from_bool(b)` 將 `bool` 轉為 `L` 或 `H`；`to_bool()` 則將 `L`/`H` 轉回 `Some(bool)`，對 `X`/`Z` 回傳 `None`。

## Level 運算（truth tables）

所有邏輯運算實作在 `Level` 的方法中（`signal.rs:29-74`），遵循 IEEE 1164 標準。

### not()

| input | output |
|-------|--------|
| L | H |
| H | L |
| X | X |
| Z | X |

### and(a, b)

| a\\b | L | H | X | Z |
|------|---|---|---|---|
| L | L | L | L | L |
| H | L | H | X | X |
| X | L | X | X | X |
| Z | L | X | X | X |

### or(a, b)

| a\\b | L | H | X | Z |
|------|---|---|---|---|
| L | L | H | X | X |
| H | H | H | H | H |
| X | X | H | X | X |
| Z | X | H | X | X |

### xor(a, b)

| a\\b | L | H | X | Z |
|------|---|---|---|---|
| L | L | H | X | X |
| H | H | L | X | X |
| X | X | X | X | X |
| Z | X | X | X | X |

`nand(a, b)` = `and(a, b).not()`；`nor(a, b)` = `or(a, b).not()`。

## Wire 與 WireRef

`Wire` 是單一位元訊號的容器（`signal.rs:94-104`）：

```rust
pub struct Wire {
    pub value: Level,
    pub name: String,
}
```

`WireRef` 是共享的可變參考（`signal.rs:106`）：

```rust
pub type WireRef = Rc<RefCell<Wire>>;
```

採用 `Rc<RefCell<...>>` 模式的原因：
- **共享所有權**：同一個訊號可能被多個閘當成輸入或輸出（扇出 fan-out）。
- **內部可變性**：閘在 `eval()` 時需要修改其所連接的 Wire 的值，但 Rust 的所有權規則不允許同時有多個可變參考；`RefCell` 提供了執行期借用檢查來繞過此限制。

輔助函數（`signal.rs:108-168`）：
- `wire(name)`：建立新的 WireRef，名稱自動加上遞增編號
- `bus(name, width)`：建立一組 Vec\<WireRef\>
- `get(w)` / `set(w, val)`：讀寫單一訊號
- `get_bus(b)` / `set_bus(b, vals)`：讀寫整組匯流排
- `bus_to_u16(b)` / `u16_to_bus(b, val)`：匯流排與 u16 之間的轉換
- `val_to_bits(val, width)` / `bits_to_u64(bits)`：u64 值與位元向量的轉換

## 訊號變化如何傳播

每個閘（gate）實作一個 `eval(&mut self)` 方法。方法內部：
1. 讀取所有輸入的 WireRef（透過 `get()`）
2. 執行邏輯運算
3. 若計算結果與輸出 Wire 的當前值不同（`if get(&self.y) != v`），則寫入新值（`set(&self.y, v)`）

寫入新值後，下一輪的 `eval()` 就會偵測到變化而繼續傳遞。這種「只在變化時寫入」的機制是事件驅動的核心。

## Sim 引擎

`Sim` 結構體（`sim.rs:7-12`）管理整個模擬狀態：

```rust
pub struct Sim {
    comb: Vec<EvalFn>,    // 組合邏輯評估函數
    seq: Vec<EvalFn>,     // 循序邏輯評估函數（時脈邊緣觸發）
    pub clk: WireRef,     // 時脈訊號
    time: u64,            // 目前模擬時間
}
```

`EvalFn` 的類型是 `Rc<RefCell<dyn FnMut()>>`，也就是可變閉包的共享封裝。

### Sim::new()

建立新的模擬實例（`sim.rs:15-22`）：
- 內部時脈 clk 透過 `wire("clk")` 建立，初始值為 `Level::X`
- time 設為 0
- comb 與 seq 向量為空

### add_comb(f)

註冊一個組合邏輯評估函數（`sim.rs:24-26`）。此函數會在每次 `eval()` 時被呼叫。典型的用法是把閘的 `eval()` 包進閉包：

```rust
let mut gate = And::new(a, b, y);
sim.add_comb(move || gate.eval());
```

### add_seq(f)

註冊一個循序邏輯評估函數（`sim.rs:28-30`），只在時脈邊緣（posedge/negedge）時才被呼叫。

### eval() — 收斂迴圈

組合邏輯的評估（`sim.rs:32-46`）：

```rust
pub fn eval(&mut self) {
    for _ in 0..10 {
        let mut changed = false;
        for f in &self.comb {
            let orig = get(&self.clk);
            f.borrow_mut()();
            if get(&self.clk) != orig {
                changed = true;
            }
        }
        if !changed { break; }
    }
}
```

- 最多迭代 10 次（delta 週期上限）
- 每一輪呼叫所有組合函數
- 若所有函數都未改變任何訊號（經由 clk 的變化來偵測），則提前終止
- 若 10 輪後仍未收斂，則強制停止（避免組合迴圈永遠運轉）

### posedge()

模擬時脈正緣（`sim.rs:48-54`）：
1. 將 clk 設為 `Level::H`
2. 執行所有循序函數
3. 呼叫 `eval()` 讓組合邏輯收斂

### negedge()

模擬時脈負緣（`sim.rs:56-61`）：
1. 將 clk 設為 `Level::L`
2. 執行所有循序函數

注意 `negedge()` 後不呼叫 `eval()`，這是與 `posedge()` 的對稱差異。

### tick()

一個完整的時脈週期（`sim.rs:63-67`）：
1. `posedge()` — 正緣、循序邏輯、組合收斂
2. `negedge()` — 負緣、循序邏輯
3. `time += 1`

### run(cycles)

執行多個週期（`sim.rs:69-73`）：`tick()` 重複 cycles 次。

## 為什麼需要收斂迴圈

組合邏輯可能存在反饋路徑（combinatorial loops），例如：

```
a = not(b)
b = not(a)
```

當 `a` 改變時，`not(b)` 會讓 `b` 改變，進而又讓 `a` 改變，形成無限震盪。在真實硬體中這是不可用的設計，但模擬器仍需處理。收斂迴圈的設計方式是：

1. 每一輪（delta cycle）執行所有閘的評估
2. 若某一輪完全沒有任何訊號變化，表示已達到穩定狀態（fixpoint）
3. 設定最大迭代次數（10 輪）作為安全網

這與 Verilog 的 delta 週期概念一致：每個 delta 週期內，所有同時發生的事件先被處理，若引發新的變化則排入下一個 delta 週期。

## 與 Verilog 模擬語義的比較

### Verilog 的事件排程（stratified event scheduling）

Verilog 標準定義了四層事件區域：

| 區域 | 用途 |
|------|------|
| Active | 阻塞赋值（`=`）、$display、$monitor |
| Inactive | `#0` 延遲的阻塞赋值 |
| NBA (Non-Blocking Assignment) | 非阻塞赋值（`<=`）的更新 |
| Monitor | $strobe、$monitor 的輸出 |

模擬時間不推進時，事件會循環穿越這些區域直到收斂，然後才推進到下一個時間步。

### rhdl 的簡化模型

rhdl 的模擬引擎大幅簡化了事件排程：

- 沒有 Active / Inactive / NBA / Monitor 的分層
- `add_comb` 對應組合邏輯（類似 `assign`），`eval()` 反覆執行直到收斂
- `add_seq` 對應 `always @(posedge clk)` 中的非阻塞賦值
- 非阻塞賦值（`<=`）在翻成 Rust 時，會放在 `add_seq` 中，在 posedge/negedge 時一次性更新
- 沒有 `#delay`、`wait` 或 `@` 事件控制
- 沒有 timing 與 SDF 反標註

## 閘的評估（Gate evaluation）

所有閘都透過 `binary_gate!` 巨集定義（`gate.rs:3-25`），生成的模式一致：

```rust
pub fn eval(&mut self) {
    let v = get(&self.a).$op(get(&self.b));
    if get(&self.y) != v {
        set(&self.y, v);
    }
}
```

透過 `get()` 讀取輸入值、執行運算、比較輸出是否不同、只在變化時寫入。這確保了：
- 沒有變化就不會觸發後續的傳播
- 同一 delta 週期內，多次評估同一個閘也不會浪費

`binary_gate!` 產生了 `And`、`Or`、`Xor`、`Nand`、`Nor` 五種二元閘，外加手寫的 `Not`（`gate.rs:33-50`）。

## 單位元 vs. 匯流排模擬

rhdl 以單一位元（`WireRef`）為基本單位。匯流排以 `Vec<WireRef>` 表示，其中索引 0 對應 LSB。

- 單位元運算：直接使用 `Level::and()`、`Level::or()` 等方法
- 匯流排操作：透過 `bus_to_u16()` 將匯流排讀成 u16，進行算術後再以 `u16_to_bus()` 寫回

例如 Counter（`Counter.rhdl:26-35`）：

```rust
pub fn eval(&mut self) {
    if get(&self.rst) != Level::L {
        u16_to_bus(&self.q, 0);
    } else if get(&self.en) != Level::L {
        u16_to_bus(&self.q, (bus_to_u16(&self.q) as u64 + 1) & 255);
    }
}
```

## 邊緣偵測

rhdl 沒有實作獨立的邊緣偵測邏輯；時脈邊緣的處理由 `Sim` 引擎控制：

- `posedge()` 將 clk 從 `L` 設為 `H` 並執行 `seq` 函數
- `negedge()` 將 clk 從 `H` 設為 `L` 並執行 `seq` 函數

使用者不需要手動偵測 `posedge` / `negedge`，而是透過 `add_seq` 註冊的閉包在正確的邊緣被自動呼叫。

## prelude 模組

`mod.rs:5-12` 提供了一個 `prelude` 模組，方便使用：

```rust
pub mod prelude {
    pub use crate::rhdl::gate::{And, Nand, Nor, Not, Or, Xor};
    pub use crate::rhdl::signal::{
        bits_to_u64, bus, bus_to_u16, get, get_bus, set, set_bus, u16_to_bus, val_to_bits, wire,
        Level, WireRef,
    };
    pub use crate::rhdl::sim::Sim;
}
```

rhdl 檔案開頭只需 `use verilog2rust::rhdl::prelude::*;` 即可使用所有必要元件。

## helper 類型

`signal.rs` 提供的輔助函數將位元向量與整數值互轉：

- `bus_to_u16(b: &[WireRef]) -> u16` — 讀取匯流排，組合為 u16
- `u16_to_bus(b: &[WireRef], val: u16)` — 將 u16 寫入匯流排
- `val_to_bits(val: u64, width: usize) -> Vec<Level>` — u64 轉位元向量
- `bits_to_u64(bits: &[Level]) -> u64` — 位元向量轉 u64

這些函數在模擬 testbench 中頻繁使用，方便設定輸入與檢查輸出。

## Sim 的限制

目前的 rhdl 模擬器有以下已知限制：

- **沒有 `#delay`**：Verilog 的 `#10` 延遲無法表達。所有組合邏輯在同一個 delta 週期內收斂，沒有時間推進的概念（除了 `tick()` 的週期計數）。
- **沒有 `always @(*)` 的自動敏感列表**：使用者必須手動註冊 comb 閉包。
- **沒有 specify blocks**：無法描述模組內的時序規格（setup/hold 時間）。
- **沒有 SDF 反標註**：無法從標準延遲檔案（Standard Delay Format）讀取實體延遲。
- **沒有事件佇列分層**：所有組合邏輯共用同一個 eval 循環，不像 Verilog 有 Active / Inactive / NBA / Monitor 的分層。
- **單一時脈域**：只有一個 clk WireRef，沒有多相位時脈或非同步時脈域支援。

## 生成的 Rust 程式碼如何使用 Sim

以 `adder4_tb.rhdl` 為例，testbench 的主流程為：

```
main() → Sim::new() → add_comb → set 輸入 → eval() → assert 輸出
```

沒有使用 `add_seq`、`posedge` 或 `tick` 的純組合電路（如加法器），testbench 直接在 `eval()` 後檢查結果：

```rust
fn main() {
    let mut tb = Adder4Tb::new();
    tb.run();
}
```

而在 `Adder4Tb::run()` 中：

```rust
u16_to_bus(&self.a, 3);
u16_to_bus(&self.b, 5);
set(&self.cin, Level::L);
self.eval();  // 收斂組合邏輯
// 檢查 bus_to_u16(&self.sum) 是否等於 8
```

對於含觸發器的設計，則會使用 `Sim::posedge()` / `Sim::tick()` 來驅動時脈。
