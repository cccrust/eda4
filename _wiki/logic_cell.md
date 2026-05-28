# Logic Cell (PLB — Programmable Logic Block)

## 概述

Logic Cell（邏輯單元）是 FPGA 中最基本的可程式化建構區塊，不同廠商有不同命名：

| 廠商 | 名稱 |
|------|------|
| Xilinx (AMD) | CLB (Configurable Logic Block) / SLICE |
| Altera (Intel) | LE (Logic Element) / ALM (Adaptive Logic Module) |
| Lattice | PLB (Programmable Logic Block) / Logic Cell |
| Microsemi | Logic Tile |

儘管名稱各異，核心概念一致：一個 Logic Cell 包含一組查找表（LUT）、一個正反器（DFF）、選擇器（MUX）以及選用的進位鏈（Carry Chain），能夠實現任意組合邏輯函數、暫存狀態、以及算術運算。

在 iCE40 架構中，每個 Logic Cell 對應一個 **ICESTORM_LC** 原語（primitive），封裝了 LUT4、DFF 與專用進位邏輯。verilog2fpga 專案的合成器會將通用閘級網路（AND、OR、XOR、NOT、Mux、Adder/Subtractor）透過 techmap 映射為 `ICESTORM_LC` 元件，並將邏輯函數的真值表編碼為 `LUT_INIT` 參數：

```rust
CellKind::Lut { init } => {
    // init: u16 -> 4-LUT 的 16-bit 真值表
    params.insert("LUT_INIT", init.into());
    ("ICESTORM_LC".to_string(), params, conns)
}
```

---

## LUT (Look-Up Table) — 查找表

### 原理

LUT 是 Logic Cell 的核心，本質上是一個小型 SRAM，將輸入視為位址線，輸出為該位址儲存的位元值。一個 **N 輸入 LUT** 包含：

- N 條位址線（輸入）
- 2^N 個位元儲存單元（bitcell）
- 1 條資料輸出線

對於任意的 N 輸入布林函數，都可以用一個 N-LUT 實現。這是因為任何布林函數都可以表示為真值表（truth table），而真值表正好對應 SRAM 的內容。

### 4-LUT (iCE40)

iCE40 系列使用 **4-LUT**，具有：

- 4 個輸入：I0, I1, I2, I3
- 1 個輸出：O
- 16 個 bitcell，對應 16 位元的初始化值（`init: u16`）

真值表到 LUT 的映射方式：

```
init bit[0] = f(0,0,0,0)
init bit[1] = f(1,0,0,0)
init bit[2] = f(0,1,0,0)
init bit[3] = f(1,1,0,0)
...
init bit[15] = f(1,1,1,1)
```

在 verilog2fpga 的 netlist 中，4-LUT 表示為：

```rust
pub enum CellKind {
    Lut { init: u16 },
    // ...
}
```

4-LUT 可以實現所有 4 輸入布林函數，總計 2^16 = 65536 種可能函數。在 ASC 配置中，LUT 的 init 值以 4 位元十六進位字串表示（如 `"0123"`），解析代碼位於 `v2f-bitstream/src/asc.rs`：

```rust
let hex_str = parts[6].trim_matches('"');
let init = u16::from_str_radix(hex_str, 16)?;
cfg.luts.push(LutConfig { output, inputs: [i0, i1, i2, i3], init });
```

### 大 LUT：5-LUT 與 6-LUT

更大規模的 LUT 可以透過組合較小的 LUT 加上多工器來建構：

- **5-LUT**：需要 32 個 bitcell（32x1 SRAM），可以由兩個 4-LUT + 一個 2-to-1 MUX 構成，MUX 的選擇線為第 5 個輸入。
- **6-LUT**：需要 64 個 bitcell（64x1 SRAM），可以由四個 4-LUT + 三個 MUX（兩層樹狀結構）構成。

Xilinx 從 Spartan-6 / Virtex-5 世代開始採用 6-LUT 作為基本單元，與 iCE40 的 4-LUT 不同。

### Fracturable LUT（可分割 LUT）

現代 FPGA 的 LUT 通常支援「可分割」模式：一個大 LUT 可以拆分為兩個較小的獨立 LUT。例如：

- 一個 6-LUT 可以作為兩個獨立的 5-LUT 使用（共享部分輸入）
- 一個 5-LUT 可以作為兩個獨立的 4-LUT 使用

這提高了 LUT 的利用率，當設計中不需要那麼多寬輸入函數時，可以將一個 LUT 用於兩個窄邏輯函數。

---

## DFF (D Flip-Flop) — D 型正反器

### 基本結構

每個 Logic Cell 通常包含一個 DFF，作為時序邏輯的儲存元件。基本接腳：

- **D**：資料輸入
- **Q**：資料輸出
- **CLK**：時脈輸入（通常為正緣觸發，posedge）
- **CE**（選擇性）：時脈致能（Clock Enable）
- **SR**（選擇性）：同步或非同步的 Set/Reset

### iCE40 的 FF 配置

在 iCE40 架構中，每個 ICESTORM_LC 的 DFF 支援：

- 正緣觸發（posedge）時脈
- 選擇性的時脈致能（CE）
- 選擇性的同步 Set/Reset（SR）

在 ASC 配置中的表示：

```rust
pub struct FfConfig {
    pub output: u32,
    pub ce: Option<u32>,
    pub sr: Option<u32>,
}
```

當合成器遇到暫存器時（CellKind::Dff），techmap 會將其對應到 ICESTORM_LC 並設定 DFF_ENABLE 標誌。

### 輸出選擇

每個 Logic Cell 的最終輸出可以選擇：
- **LUT 輸出**（組合邏輯模式）
- **FF 輸出**（暫存器模式）
- LUT → FF 路徑（LUT 計算結果直接饋入 DFF，在一個 cell 內完成）

這組選擇由內部的 MUX 控制。

---

## Carry Chain — 進位鏈

### 算術運算的需求

加法器、減法器、計數器等算術運算需要進位傳播。最簡單的漣波進位加法器（Ripple-Carry Adder）中，每一位的進位輸出 (`cout`) 會饋入下一位的進位輸入 (`cin`)。

在一般的 LUT+FF 架構中實現加法器需要多個 LUT（一個用於和、一個用於進位），且延遲較大。為了解決這個問題，FPGA 在每個 Logic Cell 中加入了專用的快速進位鏈（Dedicated Carry Chain）。

### 進位邏輯原理

全加器（Full Adder）的邏輯式：

```
sum = a ^ b ^ cin
cout = (a & b) | (cin & (a ^ b))
```

在進位鏈實作中，通常定義兩個傳播訊號：

- **Generate (G)** = a & b（無論 cin 為何都產生進位）
- **Propagate (P)** = a ^ b（當 cin=1 時傳播進位）

則進位輸出為：

```
cout = G | (P & cin)
```

iCE40 的每個 Logic Cell 內含專用進位邏輯，在 netlist 中以 `CellKind::Carry` 表示：

```rust
CellKind::Carry => {
    // I0 = a, I1 = b, O = carry output
    ("$_CARRY_".to_string(), params, conns)
}
```

在 ASC 配置中：

```rust
pub struct CarryConfig {
    pub output: u32,
    pub ci: Option<u32>,
}
```

iCE40 的進位鏈是垂直鏈接的（由上到下），每個 Logic Tile 的進位輸出連接到下一個 Tile 的進位輸入。

### 快速進位架構

不同 FPGA 廠商採用不同的進位架構：

- **漣波進位（Ripple-Carry）**：最簡單，每個 cell 的 cout → 下一個 cell 的 cin，延遲與位元數線性成長。
- **Manchester 進位鏈**：使用傳輸電晶體加速進位傳播。
- **Brent-Kung / Kogge-Stone**：前綴加法器架構，可在 Log(N) 時間內完成進位，多用於高效能 FPGA。

---

## MUX — 多工器

Logic Cell 內部的多工器負責選擇訊號路徑，主要功能包括：

1. **LUT vs FF 輸出選擇**：選擇 cell 的輸出是 LUT 的組合結果或是 FF 的暫存結果。
2. **同步/非同步控制選擇**：選擇 FF 的時脈致能、Set/Reset 來源。
3. **Carry 路徑選擇**：選擇 cell 是否參與進位鏈。
4. **LUT 輸入選擇**：某些架構中，LUT 的輸入可以來自繞線資源或來自相鄰 cell。

在 netlist 中，2-to-1 MUX 以 `CellKind::Mux2` 表示，具有 A、B 兩個輸入、S 選擇線、Y 輸出。

---

## iCE40 專用：SB_LUT4 與 ICESTORM_LC

### 一個 Logic Tile 的結構

在 iCE40 架構中，每個 **Logic Tile**（邏輯瓷磚）對應到晶片實體佈局中的一個位置，包含一組完整的可程式化資源：

```
┌─────────────────────┐
│  4-LUT (SB_LUT4)    │  ← 16-bit init SRAM
├─────────────────────┤
│  DFF                │  ← CE, SR, posedge clock
├─────────────────────┤
│  Carry Chain        │  ← cin → cout (垂直鏈接)
├─────────────────────┤
│  繞線 MUX / Buffers │  ← 內部路由選擇
└─────────────────────┘
```

### SB_LUT4 原語

Lattice iCE40 的 LUT 原語為 `SB_LUT4`，具有：

- 4 個資料輸入：I0, I1, I2, I3
- 1 個資料輸出：O
- 1 個參數：LUT_INIT（16-bit 無號整數）

Verilog 例化範例：

```verilog
SB_LUT4 #(.LUT_INIT(16'h8000)) u_and4 (
    .I0(a), .I1(b), .I2(c), .I3(d), .O(y)
);
// y = a & b & c & d
```

### CRAM Framing

每個 Logic Tile 在設定記憶體（CRAM）中佔據 **7 個 Frame**，每個 Frame 為 1320 bits（33 words x 40 bits）：

```rust
impl TileType {
    pub fn num_frames(&self) -> u32 {
        match self {
            TileType::Logic => 7,  // 一個 Logic Tile = 7 frames
            TileType::Io => 3,
            TileType::Bram => 14,
            TileType::Dsp => 7,
        }
    }
}
```

Frame 的位址計算採用 Row-major 排列：

```rust
pub fn tile_start_frame(&self, pos: &TilePos, tile_type: TileType) -> u32 {
    let frames_per_row = self.device.frames_per_row();
    let row_offset = pos.row * frames_per_row;
    let col_offset = pos.col * tile_type.num_frames();
    row_offset + col_offset
}
```

LUT 的 16-bit init 值被分散儲存在該 Tile 的 7 個 Frame 中的特定位元位置（由 icestorm 的 icebox 資料庫定義）。

---

## 邏輯單元密度

iCE40 系列各型號的邏輯單元數量與晶片尺寸：

| 裝置 | 邏輯列 (rows) | 邏輯行 (cols) | 邏輯 Tile 總數 | 約當 Logic Cells |
|------|:---:|:---:|:---:|:---:|
| HX1K | 30 | 16 | ~480 | ~960 |
| LP1K | 30 | 16 | ~480 | ~960 |
| HX4K | 40 | 20 | ~800 | ~3,520 |
| HX8K | 70 | 28 | ~1,960 | ~7,680 |
| UP5K | 40 | 22 | ~880 | ~5,280 |

> 注意：iCE40 的每個邏輯 Tile 實際包含 2 個 Logic Cells（2 x PLB），因此邏輯單元數約為 Tile 數的兩倍。verilog2fpga 的 `ArchGraph` 簡化模型將每個邏輯 Tile 視為一個可放置位置，實際物理密度以 Lattice 官方資料為準。

---

## 合成映射：閘極網路到 LUT

### 基本原理

合成工具（Synthesizer）的職責之一是將高階邏輯閘網路（與閘、或閘、反相器等）映射到目標 FPGA 的 LUT 結構中。這個過程稱為 **technology mapping（技術映射）**。

verilog2fpga 的 `techmap` 模組執行的正是這個工作。它將 netlist 中的通用 CellKind 轉換為 iCE40 的 ICESTORM_LC 表示：

- `CellKind::And` → 計算 2 輸入與閘的真值表
- `CellKind::Or` → 計算 2 輸入或閘的真值表
- `CellKind::Xor` → 計算 2 輸入 XOR 閘的真值表
- `CellKind::Not` → 計算反相器的真值表
- 組合後的函數 → 合併為單一的 LUT_INIT 值

例如，一個 `y = a & b | c & d` 的邏輯可以映射到一個 4-LUT 中：

```
I0=a, I1=b, I2=c, I3=d
LUT_INIT = 16'h8000  (y = 1 only when I0=1,I1=1 or I2=1,I3=1)
```

實際的 init 值計算方式：

```rust
// 假設函數為 y = (I0 & I1) | (I2 & I3)
// 真值表：
// addr=0000 -> y=0
// addr=0011 -> y=1 (I0=1,I1=1)
// addr=1100 -> y=1 (I2=1,I3=1)
// addr=1111 -> y=1
// init = 0b1000100000000000 = 0x8800
```

### LUT Sharing 與 Decomposition

當邏輯函數的輸入超過 LUT 的輸入數時（例如 6 輸入函數需要映射到 4-LUT），需要進行 **LUT decomposition**（LUT 分解）：

1. **Shannon Expansion（夏農擴展）**：將函數分解為 `f = x·f_x + x'·f_x'`，其中 `f_x` 和 `f_x'` 分別是將 x 固定為 1 和 0 時的子函數。
2. **BDD-based（二元決策圖）**：使用 BDD 進行函數分解。
3. **多層次分解**：透過多個 LUT 級聯實現寬輸入函數。

寬函數分解後，需要多個 LUT 級聯，並透過繞線資源連接，這會增加延遲。

---

## PNR 放置：Logic Cell 到 Tile

在 verilog2fpga 的 PNR（Place and Route）階段，`v2f-pnr` 負責將經過 techmap 後的 cell 放置到具體的 Tile 座標上。

### 放置區域

`ArchGraph` 定義了可放置的邏輯區域：

```rust
pub fn logic_tiles(&self) -> Vec<TileCoord> {
    self.tiles.iter()
        .filter(|t| t.tile_type == TileType::Logic)
        .map(|t| t.coord)
        .collect()
}
```

邏輯 Tile 位於晶片內部區域：第一列與最後一列、第一行與最後一行為 IO Tile，內部區域為 Logic Tile。

### 模擬退火演算法

放置演算法使用模擬退火（Simulated Annealing）進行最佳化：

```rust
pub fn place(placement: &mut Placement, arch: &ArchGraph) {
    // 初始高溫 → 隨機交換 cell 位置
    // 逐步降溫 → 接受 cost 改善或部分劣化解
    // cost 函數：線長 (wirelength) + 擁擠度
}
```

每個被放置的 cell 最終會對應到一個 `TileCoord { row, col }`，在 ASC 輸出中表示為 `.logic_tile <col> <row>`。

---

## CRAM 中的 LUT 配置儲存

### iCE40 CRAM 結構

CRAM（Configuration RAM）是 FPGA 的設定記憶體，儲存了所有 LUT 的真值表、FF 配置、繞線選擇位元等。iCE40 的 CRAM 結構：

- 每個 Frame：33 words x 40 bits = 1320 bits = 165 bytes
- 每個 Logic Tile：7 Frames
- Frame 組織方式：Row-major，每列從左到右排列所有 Tile 的 Frame

### LUT Init 的位元分佈

對於 iCE40，每個 4-LUT 的 16 位元 init 值被分布在 Tile 的 7 個 Frame 中的特定 word/bit 位置。這個對應關係由 Lattice 的布局決定，並在 icestorm 專案的 icebox 資料庫中有詳盡記錄。

在 verilog2fpga 中，目前透過 `apply_asc_to_cram` 函數處理 CRAM 寫入：

```rust
pub fn apply_asc_to_cram(asc: &AscFile, cram: &mut Cram, device: Ice40Device) {
    let addr_map = CramAddrMap::new(device);
    for tile in &asc.logic_tiles {
        for w in &tile.wiring {
            // 將 wiring bit 寫入對應 Frame 的對應位置
            let frame_sub = (w.bit_index / (FRAME_BITS / 7)) % 7;
            // ... 位址解析與寫入
        }
    }
}
```

---

## 歷史演進

### Xilinx

- **XC4000 / Spartan-3**（1990s-2000s）：4-LUT + 2 DFF per CLB
- **Virtex-5 / Spartan-6**（2006+）：6-LUT + 2 DFF per SLICE，一個 CLB 包含 2 個 SLICE
- **Virtex-6 / 7 Series**：6-LUT + 8 DFF per SLICE（可 Fracturable）
- **UltraScale / UltraScale+**：6-LUT + 16 DFF per SLICE，支援更豐富的 Fracturable 模式

### Altera (Intel)

- **FLEX 10K / Cyclone**（1990s-2000s）：4-LUT + 1 FF per LE
- **Stratix / Cyclone II**：4-LUT 支援 Fracturable 模式（兩個 3-LUT 或一個 4-LUT + 一個 3-LUT）
- **Stratix II 起採用 ALM**：8 輸入自適應邏輯模組，可拆分為多種 LUT 組合

### Lattice

- **iCE40**：4-LUT + DFF + Carry chain（每個 PLB），2 個 PLB 組成一個 Logic Tile
- **ECP5**：4-LUT + DFF + Carry，支援 Fracturable 模式
- **Nexus**：4-LUT + DFF + Carry，採用 28nm FD-SOI 製程

---

## 總結

Logic Cell（PLB）是 FPGA 可程式化邏輯的基石，透過 LUT 實現任意組合邏輯、透過 DFF 實現狀態儲存、透過 Carry Chain 實現高效算術運算。

verilog2fpga 專案在軟體層面完整實現了從 Verilog 解析 → 合成 → 技術映射 → 放置繞線 → 位元流打包的完整流程，其中 Logic Cell 是貫穿所有階段的中心抽象：

```
Verilog → AST → Netlist (CellKind::Lut) → ICESTORM_LC → ASC → CRAM → Bitstream
```

理解 Logic Cell 的內部結構對於 FPGA 開發者最佳化設計、理解面積約束、以及除錯時序問題至關重要。
