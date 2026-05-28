# Verilog2Rust / ruHDL / rhdl

## 概述

Verilog2Rust 是 eda4 專案中的一個工具鏈，可將 Verilog HDL 硬體描述語言轉換為可執行的 Rust 程式碼。轉換後的 Rust 程式碼利用 rhdl（Rust HDL）執行時期函式庫進行模擬，使設計者能夠在不依賴傳統 Verilog 模擬器的情況下，直接在 Rust 生態系中進行硬體模擬與驗證。

整個專案位於 `verilog2rust/` 目錄下，是一個同時提供二進位工具與程式庫的 Rust crate。其核心流程為：解析 Verilog 原始碼、建立抽象語法樹（AST）、生成 Rust 結構體與實作程式碼，最後透過 rustc 編譯並執行。

Verilog2Rust 的命名轉換遵循 Rust 慣例：模組名稱轉為 PascalCase，訊號名稱轉為 snake_case。

## 為何要轉譯為 Rust

傳統硬體設計的模擬通常依賴專用模擬器，但 Rust 作為系統程式語言提供了多項優勢。首先，Rust 的零成本抽象（zero-cost abstractions）使生成的模擬程式碼具有接近原生的執行效能，這對於需要大量測試向量的驗證工作至關重要。

其次，Rust 的型別系統和所有權模型在編譯期即保證記憶體安全，消除了模擬程式碼中的懸置指標、資料競爭等常見錯誤。這意味著硬體設計者可以專注於邏輯驗證，而無需擔心底層記憶體管理問題。

第三，Rust 擁有完善的測試基礎設施。`#[test]` 屬性和 `cargo test` 工作流程使得將 Verilog testbench 轉換為 Rust 單元測試成為自然的選擇。設計者可以將驗證整合到標準的 CI/CD 管道中。

最後，Rust 豐富的套件生態系（crates.io）提供了各種工具庫，可用於擴展模擬功能，例如波形輸出、覆蓋率分析等。

## 與傳統 Verilog 模擬器的比較

傳統的 Verilog 模擬器如 Icarus Verilog、Verilator 和 ModelSim 各有其定位。Icarus Verilog 是一個開源的編譯式模擬器，支援 IEEE 1364-2005 標準的大部分功能。Verilator 則以高效能著稱，它將 Verilog 轉譯為 C++ 進行週期精確（cycle-accurate）模擬。ModelSim 是商業級的模擬器，提供完整的除錯和波形檢視功能。

Verilog2Rust 與這些工具最大的不同在於其輸出目標語言為 Rust。與 Verilator 的 Verilog-to-C++ 路線類似，Verilog2Rust 採用 Verilog-to-Rust 的策略，但 Rust 的記憶體安全保證和現代語言特性使其在開發體驗上具有獨特優勢。

與 Icarus Verilog 相比，Verilog2Rust 的模擬效能通常更高，因為生成的 Rust 程式碼經 rustc 最佳化後可直接在原生硬體上執行。與 ModelSim 這樣的商業工具相比，Verilog2Rust 完全免費且開源，適合整合到自動化工作流程中。

## 管線架構

Verilog2Rust 的轉換管線包含以下階段：

1. **詞法分析**：將 Verilog 原始碼分解為 Token 串列。支援的關鍵字包括 `module`、`input`、`output`、`wire`、`reg`、`assign`、`always`、`if`、`case`、`for`、`forever`、`posedge`、`negedge` 等。也處理 Verilog 數值字面量（如 `4'd3`、`1'b0`）和字串字面量。

2. **語法分析**：遞迴下降解析器將 Token 串列轉換為 AST（抽象語法樹）。解析器支援模組宣告、埠宣告、變數宣告、連續賦值、程序區塊、閘級實例化、子模組實例化、運算式（包括二元運算、一元運算、串接、複製、位元選取、部分選取）等結構。

3. **程式碼生成**：遍歷 AST 並生成對應的 Rust 程式碼。每個 Verilog 模組轉換為一個 Rust 結構體，並實作 `new()`、`eval()` 和 `run()` 方法。

4. **編譯與執行**：生成的 Rust 程式碼透過 `rustc` 編譯為可執行檔，然後執行。支援 `.rhdl` 檔案格式的直接編譯與執行。

## AST 結構

AST 的定義位於 `src/verilog/ast.rs`，包含以下核心型別：

- **Module**：代表一個 Verilog 模組，包含名稱、埠列表、內部項目列表和參數表。
- **Port**：描述模組埠，包含方向（`PortDir`：Input、Output、Inout）、名稱和可選的寬度範圍（`Range`）。
- **Range**：定義位元範圍，包含 `msb` 和 `lsb`。
- **ModuleItem**：模組內部的項目列舉，包括 `Wire`（線網）、`Reg`（暫存器）、`Integer`（整數）、`Assign`（連續賦值）、`Always`（always 區塊）、`Initial`（initial 區塊）、`GateInst`（閘級實例化）和 `ModuleInst`（子模組實例化）。
- **Stmt**：程序語句列舉，包括 `BlockingAssign`（阻塞賦值）、`NonBlockingAssign`（非阻塞賦值）、`If`、`Case`、`For`、`Forever`、`SysCall`（系統呼叫如 `$display`）、`SysFinish`（`$finish`）和 `DelayStmt`（`#delay`）。
- **Expr**：運算式列舉，包括 `Number`（數值字面量）、`Ident`（識別字）、`Binary`（二元運算）、`Unary`（一元運算）、`Concat`（串接）、`Replicate`（複製）、`Select`（部分選取）、`BitSelect`（位元選取）和 `Cond`（條件運算式）。
- **BinaryOp** 與 **UnaryOp**：分別定義二元和一元運算子。

## 程式碼生成

程式碼生成器位於 `src/verilog/gen.rs`，是整個管線的核心。其生成的 Rust 程式碼遵循固定的模式：

### 結構體定義

每個 Verilog 模組對應一個 Rust 結構體：

- **埠**：`input` 和 `output` 埠成為 `pub` 欄位。寬度為 1 的埠型別為 `WireRef`；寬度大於 1 的埠型別為 `Vec<WireRef>`。
- **內部線網與暫存器**：成為私有欄位，型別同樣根據寬度決定。
- **閘級元件**：每個閘實例成為對應的 `Gate` 結構體欄位（如 `And`、`Or`、`Xor`、`Not`）。
- **子模組**：每個子模組實例成為父模組結構體中的一個欄位。

### new() 方法

建構子接收埠對應的 `WireRef` 或 `Vec<WireRef>` 參數，在函數體內建立內部訊號（使用 `wire()` 或 `bus()` 輔助函數），然後初始化所有欄位。

### eval() 方法

`eval()` 方法模擬組合邏輯的行為，依序呼叫：
1. 所有閘級元件的 `eval()` 方法
2. 所有子模組的 `eval()` 方法
3. 所有 `assign` 連續賦值語句
4. 所有組合邏輯 always 區塊（不含 `#delay` 的 always 區塊）

### run() 方法

`run()` 方法處理序向邏輯和 testbench 行為：
1. 呼叫子模組的 `run()` 方法
2. 執行不含 `#delay` 的 initial 區塊
3. 對含有 `#delay` 的 always 區塊，包裝在 `loop {}` 中（表示持續執行的時序邏輯）
4. 執行含有 `#delay` 的 initial 區塊（testbench 主體）

## rhdl 執行時期函式庫

rhdl（位於 `src/rhdl/`）是 Verilog2Rust 的模擬核心，提供所有必要的執行時期元件。

### Wire 與 WireRef

`Wire` 是模擬中的基本節點，包含一個 `Level` 值與一個名稱字串。為了實現共享可變狀態（硬體連線的本質），`Wire` 被封裝在 `Rc<RefCell<Wire>>` 中，型別別名為 `WireRef`。

輔助函數 `wire(name)` 建立一個新的 `WireRef`，`bus(name, width)` 建立一組 `WireRef` 向量。`get(w)` 和 `set(w, val)` 分別用於讀取和寫入 `WireRef` 的值。

### Level 四值邏輯

`Level` 列舉定義了四種邏輯狀態：

- **L**：邏輯 0（低電位）
- **H**：邏輯 1（高電位）
- **X**：未知狀態（不確定值）
- **Z**：高阻抗（浮接）

每個 `Level` 值實作了 `not()`、`and()`、`or()`、`xor()`、`nand()`、`nor()` 等邏輯運算方法，完整遵循四值邏輯的真值表。例如，`L AND X` 結果為 `L`（因為只要有一個輸入為 L，AND 輸出即為 L），而 `H AND X` 結果為 `X`。

### 閘級元件

rhdl 提供以下閘級結構體，每個都包含輸入 `a`（和 `b`）以及輸出 `y` 的 `WireRef`，並實作 `eval()` 方法：

- **And**：邏輯 AND 閘
- **Or**：邏輯 OR 閘
- **Xor**：邏輯 XOR 閘
- **Nand**：邏輯 NAND 閘
- **Nor**：邏輯 NOR 閘
- **Not**：邏輯 NOT 閘（單輸入）

這些閘均透過 `binary_gate!` 巨集生成，確保一致的實作模式。`eval()` 方法在輸出值發生變化時才寫入，以支援事件驅動的最佳化。

### Sim 模擬引擎

`Sim` 結構體提供事件驅動的模擬框架，包含以下核心功能：

- **組合邏輯評估**：`eval()` 方法反覆迭代執行所有註冊的組合邏輯函數，最多 10 次，直到收斂（沒有訊號再變化為止）。這種迭代收斂策略確保組合迴圈（combinatorial loops）能夠穩定。
- **時脈管理**：`posedge()` 和 `negedge()` 方法分別觸發時脈的正緣和負緣，執行所有序向邏輯函數，然後重新評估組合邏輯。
- **時脈週期**：`tick()` 方法執行一個完整的時脈週期（正緣 + 負緣）。`run(cycles)` 執行指定數量的週期。

## 模擬模型

Verilog2Rust 採用混合模擬模型，區分組合邏輯和序向邏輯：

### 組合邏輯收斂

組合邏輯的評估採用疊代收斂演算法。在 `Sim::eval()` 中，系統反覆執行所有註冊的組合函數，每次迭代後檢查是否有訊號發生變化。如果某次迭代後所有訊號都穩定，則提前終止；否則繼續迭代，最多 10 次。這個策略確保了：

1. 組合邏輯網路的正確性——訊號變更會傳播到所有相依的閘
2. 終止性——最多 10 次迭代後強制停止，防止無限迴圈
3. 效能——大部分簡單電路在 1-2 次迭代內即可收斂

### 序向邏輯

序向邏輯（時脈驅動的 always 區塊）在 `posedge()` 或 `negedge()` 時觸發。模擬引擎先執行序向函數（更新觸發器狀態），然後重新評估組合邏輯，確保新的狀態值正確傳播。

### 時間延遲

Testbench 中的 `#delay` 語句在生成的 Rust 程式碼中被轉換為 `self.eval()` 呼叫。這意味著 Verilog2Rust 採用週期精確的模擬模型，而非事件驅動的精細時間模型。`#10` 這樣的延遲在功能上等同於「等待組合邏輯穩定」。

## 程式碼生成範例：HalfAdder

以下是一個簡單的半加器 Verilog 原始碼：

```verilog
module HalfAdder(a, b, sum, carry);
    input a, b;
    output sum, carry;

    xor u1(sum, a, b);
    and u2(carry, a, b);
endmodule
```

生成的 Rust 程式碼如下：

```rust
use verilog2rust::rhdl::prelude::*;

#[derive(Debug, Clone)]
pub struct HalfAdder {
    pub a: WireRef,
    pub b: WireRef,
    pub sum: WireRef,
    pub carry: WireRef,
    u1: Xor,
    u2: And,
}

impl HalfAdder {
    pub fn new(
        a: WireRef, b: WireRef, sum: WireRef, carry: WireRef,
    ) -> Self {
        HalfAdder {
            a: a.clone(), b: b.clone(), sum: sum.clone(), carry: carry.clone(),
            u1: Xor::new(a.clone(), b.clone(), sum.clone()),
            u2: And::new(a.clone(), b.clone(), carry.clone()),
        }
    }

    pub fn eval(&mut self) {
        self.u1.eval();
        self.u2.eval();
    }
}
```

可以看到，Verilog 的 `xor` 閘對應到 Rust 的 `Xor` 結構體，`and` 對應到 `And`。閘的連線順序為：第一個參數為輸出，其餘為輸入（遵循 Verilog 閘級實例化的慣例）。

## Testbench 生成

含有 initial 區塊的 Verilog 模組被視為 testbench。以下是一個簡單的 testbench 轉換示例：

```verilog
module Adder4_tb;
    reg [3:0] a, b;
    wire [3:0] sum;
    // ...
    initial begin
        $display("3 + 5 = %d", sum);
        a = 4'd3; b = 4'd5;
        #10;
        $finish;
    end
endmodule
```

生成的 Rust 程式碼中：

- `$display` 轉換為 `println!` 巨集，其格式化字串中的 `%d` 被映射為 Rust 的 `{}`，`%h` 映射為 `{:x}`，`%b` 映射為 `{:b}`。
- `$finish` 轉換為 `return;`。
- `#10` 轉換為 `self.eval();`——模擬組合邏輯穩定。
- `reg` 型別的變數仍是 `WireRef`（Verilog2Rust 不區分 wire 和 reg 的儲存語義）。
- 帶有 `$display` 或 `$monitor` 的系統呼叫都會生成 `println!`。

如果模組中含有 initial 區塊，生成的程式碼會自動加入 `fn main()` 作為入口點，建立 testbench 實例並呼叫 `run()` 方法。

## 匯流排處理

寬度大於 1 的訊號（如 `[3:0]`）被表示為 `Vec<WireRef>`，其中索引 0 對應最低位元（LSB）。rhdl 提供了以下匯流排輔助函數：

- **bus_to_u16(bus)**：將 `Vec<WireRef>` 讀取為 `u16` 整數值
- **u16_to_bus(bus, val)**：將 `u16` 整數值寫入 `Vec<WireRef>`
- **get_bus(bus)** / **set_bus(bus, vals)**：批次讀寫 `Level` 值
- **bits_to_u64(bits)** / **val_to_bits(val, width)**：在 `u64` 和 `Vec<Level>` 之間轉換

由於 Rust 的型別系統在編譯期檢查型別，匯流排寬度在生成程式碼時即已確定。目前的最大支援寬度為 16 位元（受限於 `bus_to_u16` 和 `u16_to_bus` 的實作）。

## 建置與執行工作流程

### Verilog 轉換為 Rust

```sh
# 將 Verilog 檔案轉換為 Rust 檔案
cargo run -- HalfAdder.v HalfAdder.rs

# 若不指定輸出，預設輸出到目前目錄
cargo run -- HalfAdder.v
```

### 編譯與執行 ruHDL

```sh
# 直接編譯並執行 .rhdl 檔案
cargo run -- adder4_tb.rhdl
```

### 程式庫快取

為避免每次執行都重新編譯 verilog2rust 函式庫，系統會將函式庫預先編譯為 `.rlib` 檔案並快取在 `/tmp/verilog2rust_rlib/`。後續的 `.rhdl` 檔案編譯時透過 `--extern` 參數連結這個已快取的函式庫，大幅縮短編譯時間。

### run.sh 與 run_tb.sh

專案提供的 `run.sh` 腳本會批次轉換 `verilog/` 目錄下所有 `.v` 檔案為 `.rhdl` 檔案。`run_tb.sh` 則會找出 testbench 檔案，編譯並執行它們，驗證模擬結果。

## 支援的 Verilog 功能

### 已支援

- **模組**：完整的模組定義與實例化，包括命名連接（`.port(wire)`）和位置連接
- **埠**：`input`、`output`、`inout` 方向宣告
- **資料型別**：`wire`、`reg`、`integer`，含陣列維度（`reg [7:0] mem [0:255]`）
- **連續賦值**：`assign` 語句
- **程序區塊**：`always @(posedge/negedge/*)` 和 `initial`
- **控制流程**：`if`/`else`、`case`/`default`、`for`、`forever`
- **閘級實例化**：`and`、`nand`、`or`、`nor`、`xor`、`xnor`、`not`、`buf`
- **運算式**：算術（`+`, `-`, `*`, `/`, `%`）、位元（`&`, `|`, `^`, `~`）、邏輯（`&&`, `||`, `!`）、比較（`==`, `!=`, `<`, `>`, `<=`, `>=`）、移位（`<<`, `>>`）
- **串接與複製**：`{a, b}` 和 `{n{expr}}`
- **位元選取與部分選取**：`sig[bit]` 和 `sig[msb:lsb]`
- **條件運算式**：`cond ? if_true : if_false`
- **系統任務**：`$display`、`$monitor`、`$finish`
- **時間延遲**：`#delay` 語句
- **巨集包含**：`` `include`` 指令（遞迴展開，含循環包含偵測）
- **參數**：`parameter` 宣告

### 未支援

- **連續 timing**：`specify` 區塊、`$setup`/`$hold` 等時序檢查
- **系統任務**：`$dumpfile`、`$dumpvars`、`$readmemh`、`$writememh` 等
- **UDP**：使用者定義原語（User-Defined Primitives）
- **延遲控制**：`@(event)` 事件控制（除 `@(posedge/negedge)` 外）
- **三態驅動**：`bufif0`/`bufif1`/`notif0`/`notif1` 閘（解析器可識別但程式碼生成未完整處理）
- **浮點數**：`real` 型別
- **generate 區塊**：`generate`/`endgenerate` 結構
- **匯流排寬度**：受限於 16 位元（`bus_to_u16`/`u16_to_bus`）

## 原始碼結構

```
verilog2rust/
  src/
    main.rs           # 二進位入口：解析命令列參數，調度轉換或執行
    lib.rs            # 程式庫入口：公開 rhdl 和 verilog 模組
    rhdl/
      mod.rs          # 模組宣告與 prelude（公開所有常用型別）
      signal.rs       # Wire、WireRef、Level、匯流排輔助函數
      gate.rs         # And、Or、Xor、Nand、Nor、Not 閘結構體
      sim.rs          # Sim 模擬引擎（事件驅動，時脈管理）
    verilog/
      mod.rs          # 模組宣告與公開 API
      ast.rs          # AST 型別定義（Module、Port、Stmt、Expr 等）
      parse.rs        # 詞法分析器與遞迴下降解析器
      gen.rs          # Rust 程式碼生成器（AST → Rust 原始碼）
      include.rs      # `include 指令展開
  verilog/            # 範例 Verilog 設計與 testbench
  tests/              # 整合測試
```
