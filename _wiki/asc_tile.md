# ASC 格式與 FPGA Tile 架構

## 什麼是 ASC

ASC (ASCII Place-and-Route Output) 是一種純文字格式，用來描述 FPGA 的完整配置狀態。它由 nextpnr 或 v2f-pnr 在佈局佈線完成後產生，並由 icepack 或 v2f-bitstream 讀取，最終打包成二進位位元流 (`.bin`)。

ASC 格式源自 Project IceStorm 生態系，是 iCE40 FPGA 開源工具鏈的中間交換格式。它記錄了每個 tile 中 LUT、Flip-Flop、Carry 的配置，以及所有 routing mux 的設定值。

## ASC 檔案結構

一個 ASC 檔案由以下章節組成：

### .device 指令

指定目標裝置名稱，例如 `HX8K-CT256`、`HX1K-TQ144`、`LP1K-CM36` 等。

```
.device HX8K-CT256
```

### .logic_tile 指令

定義一個 Logic Tile 的配置，包含位置、LUT 設定、FF 設定、Carry 設定與繞線資訊：

```
.logic_tile <col> <row>
  .lut <output> <input0> <input1> <input2> <input3> <hex_init>
  .ff <output> [ce=<ce_bit>] [sr=<sr_bit>]
  .carry <output> [ci=<ci_bit>]
  .wiring <col1> <row1> <col2> <value>
```

### .io_tile 指令

定義一個 I/O Tile 的配置，包含位置與 pad 綁定：

```
.io_tile <col> <row>
  .pad <index> <name>
  .wiring <col1> <row1> <col2> <value>
```

### .wiring 指令

繞線設定，獨立於 tile 區塊之外亦可出現。格式為：

```
.wiring <col> <row> <bit_index> <value>
```

其中 `bit_index` 對應到 CRAM 中特定 routing mux 的位置，`value` 為 0 或非 0。

### .synckey

同步金鑰，用於位元流驗證：

```
.synckey 0x12345678
```

## ASC 範例

以下是一個包含一個 logic tile 與一個 io tile 的完整 ASC 檔案：

```
.module top
.io_tile 0 0
  .pad 0 clk
  .pad 1 led
.logic_tile 1 1
  .lut 0 0 0 0 0 "0123"
  .ff 0 ce=1 sr=0
  .carry 0 ci=0
  .wiring 0 0 0 128
.synckey 0x12345678
```

這個範例展示了：
- IO Tile 在 `(0,0)`，綁定了兩個 pad：`clk` 和 `led`
- Logic Tile 在 `(1,1)`，包含一個 LUT（初始化值 `0x0123`）、一個 Flip-Flop（啟用 clock enable 與 set/reset）、以及一個 Carry 單元
- 一條繞線連線從 `(0,0)` 到 `(1,1)`，設定 bit 128

## 資料結構

### LogicTileConfig

Rust 中對應的結構定義於 `v2f-bitstream/src/asc.rs:39-44`：

```rust
pub struct LogicTileConfig {
    pub pos: TilePos,
    pub luts: Vec<LutConfig>,
    pub ffs: Vec<FfConfig>,
    pub carries: Vec<CarryConfig>,
    pub wiring: Vec<WiringEntry>,
}
```

每個 Logic Tile 可以包含多個 LUT、FF、Carry 以及繞線條目。

### IoTileConfig

對應 I/O Tile 的結構：

```rust
pub struct IoTileConfig {
    pub pos: TilePos,
    pub pads: Vec<PadConfig>,
    pub wiring: Vec<WiringEntry>,
}
```

### LutConfig

4-input LUT 的配置：

```rust
pub struct LutConfig {
    pub output: u32,
    pub inputs: [u32; 4],
    pub init: u16,
}
```

`output` 是 LUT 輸出連線的位元索引，`inputs` 是四個輸入的位元索引，`init` 是一個 16-bit 的初始值，作為 LUT 的真值表 (truth table)。例如 `init = 0x0123` 表示 LUT 的輸出為 `inputs` 四個位元組合的某種布林函數結果。

### FfConfig

D Flip-Flop 的配置：

```rust
pub struct FfConfig {
    pub output: u32,
    pub ce: Option<u32>,
    pub sr: Option<u32>,
}
```

`output` 是 FF 輸出連線的位元索引，`ce` 是 clock enable 的位元索引（可選），`sr` 是 set/reset 的位元索引（可選）。

### CarryConfig

Carry 鏈的配置：

```rust
pub struct CarryConfig {
    pub output: u32,
    pub ci: Option<u32>,
}
```

`output` 是 carry 輸出的位元索引，`ci` 是 carry-in 的位元索引（可選）。

### WiringEntry

繞線連線條目：

```rust
pub struct WiringEntry {
    pub bit_index: u32,
    pub value: u32,
}
```

每個 `WiringEntry` 對應一條 routing mux 的設定。`bit_index` 是 icestorm 格式的 wiring bit 編號，`value` 為設定值（通常為 0 或非 0，非 0 表示該 routing mux 被啟用）。

### PadConfig

IO Pad 的配置：

```rust
pub struct PadConfig {
    pub index: u32,
    pub name: String,
}
```

`index` 是 pad 編號，`name` 是對應的訊號名稱。

## ASC 解析流程

ASC 的解析實作於 `v2f-bitstream/src/asc.rs` 的 `parse_asc()` 函數中（第 102-271 行）。流程為：

1. **逐行掃描**：對輸入字串的每一行進行處理，忽略註解（`#` 之後的內容）與空白行
2. **累積 tile 資料**：使用 `cur_logic` 與 `cur_io` 兩個 `Option` 變數追蹤當前正在解析的 tile。遇到 `.logic_tile` 或 `.io_tile` 指令時，將前一個 tile 推入對應的 Vec
3. **指令匹配**：根據行首的指令名稱（如 `.lut`、`.ff`、`.carry`、`.wiring`、`.pad`）解析對應的欄位，填入當前的 tile 結構
4. **結束清理**：將最後一個未推入的 tile 加入 Vec
5. **回傳 AscFile**：包含 module 名稱、所有 logic tiles、所有 io tiles 以及 synckey

`AscFile` 結構定義於第 93-99 行：

```rust
pub struct AscFile {
    pub module: String,
    pub logic_tiles: Vec<LogicTileConfig>,
    pub io_tiles: Vec<IoTileConfig>,
    pub synckey: u32,
}
```

## Tile 抽象

在 iCE40 FPGA 架構中，tile 是 FPGA  fabric 的基本邏輯單元。每種 tile 類型佔用固定數量的 CRAM frame：

| Tile 類型 | Frame 數 | 說明 |
|-----------|---------|------|
| Logic Tile | 7 | 包含一個 4-LUT + DFF + carry chain（即一個 Logic Cell） |
| I/O Tile | 3 | 包含 pad 邏輯（SB_IO primitive） |
| BRAM Tile | 14 | 包含 4Kbit 的 block RAM |
| DSP Tile | 7 | 包含乘加器 (multiplier/accumulator) |

Tile 類型枚舉定義於 `v2f-db/src/tile.rs:4-9`：

```rust
pub enum TileType {
    Logic,
    Io,
    Bram,
    Dsp,
}
```

每個 TileType 都有對應的 `num_frames()` 方法，返回該類型佔用的 frame 數量（第 11-18 行）。

### Tile 位置

`TilePos` 結構表示 tile 在 FPGA 網格中的位置：

```rust
pub struct TilePos {
    pub row: u32,
    pub col: u32,
}
```

### Logic Tile（7 frames）

Logic Tile 是 FPGA 的核心運算單元，每個 Logic Tile 包含：
- **一個 4-input LUT**：可以實現任意 4 輸入的布林函數，透過 16-bit init 值定義真值表
- **一個 D Flip-Flop**：可配置 clock enable (CE) 與 set/reset (SR)
- **一個 Carry Chain**：用於實現加法器中的進位傳遞

這三者共同構成一個完整的 Logic Cell（也稱為 LC 或 Slice）。

### I/O Tile（3 frames）

I/O Tile 位於 FPGA 晶片的邊緣，負責晶片 pad 與內部 logic 之間的訊號傳遞。每個 I/O Tile 包含：
- **多個 pad 綁定**：將封裝的實體引腳映射到內部網路
- **SB_IO primitive**：可配置為輸入、輸出或雙向緩衝器

I/O Tile 通常分佈在 FPGA 的第一列、最後一列、第一行與最後一行，形成一個環繞 logic 區域的 I/O 邊界。

### BRAM Tile（14 frames）

BRAM (Block RAM) 是 iCE40 FPGA 中的嵌入式記憶體區塊，每個 BRAM Tile 提供 4Kbit 的儲存空間。BRAM 可以配置為單埠或雙埠 RAM/ROM。

### DSP Tile（7 frames）

DSP Tile 包含硬體乘加器，用於實現數位訊號處理中的乘法與累加操作。在 iCE40 系列中，並不是所有型號都具備 DSP Tile（如 HX1K/LP1K 沒有，HX8K 有）。

## 各 iCE40 型號的 Tile 網格尺寸

不同 iCE40 型號的 tile 網格大小定義於 `v2f-db/src/ice40.rs:38-56`：

| 型號 | 列數 (rows) | 行數 (cols) | Logic Tile 網格 | 總 Frame 數 |
|------|------------|------------|----------------|------------|
| HX1K | 30 | 16 | 28 × 14 | 30 × (16×7 + 6) = 30 × 118 = 3540 |
| HX4K | 40 | 20 | 38 × 18 | 40 × (20×7 + 6) = 40 × 146 = 5840 |
| HX8K | 70 | 28 | 68 × 26 | 70 × (28×7 + 6) = 70 × 202 = 14140 |
| LP1K | 30 | 16 | 28 × 14 | 30 × (16×7 + 6) = 30 × 118 = 3540 |
| UP5K | 40 | 22 | 38 × 20 | 40 × (22×7 + 6) = 40 × 160 = 6400 |

其中 Logic Tile 網格 = (rows - 2) × cols，因為第一列與最後一列為 I/O Tile。左邊界與右邊界也為 I/O Tile。

## ASC 作為交換格式

ASC 在 v2f 工具鏈中扮演交換格式的角色：

```
.v (Verilog) → v2f-synth → .json (Netlist) → v2f-pnr → .asc → v2f-bitstream → .bin
```

- **nextpnr** 輸出 `.asc` 檔案
- **v2f-pnr** 同樣輸出 `.asc` 檔案，實作於 `v2f-pnr/src/asc_out.rs` 的 `write_asc()` 函數
- **icepack**（icestorm 工具）讀取 `.asc` 並打包為 `.bin`
- **v2f-bitstream** 的 `pack_bitstream()` 函數也讀取 ASC 資料（透過 `apply_asc_to_cram()`）並產生位元流

v2f-pnr 的 ASC 輸出實作（`asc_out.rs:6-26`）：

```rust
pub fn write_asc(placement: &Placement, routing: &Routing, arch: &ArchGraph) -> String {
    let mut s = String::new();
    writeln!(s, ".device {}", arch_device_name(arch)).ok();
    // ... 寫入 logic_tile 與 wiring 資訊
    s
}
```

## ASC Tile 與實體 FPGA 佈局的關係

ASC 檔案中的 tile 位置 (`col`, `row`) 直接對應到 FPGA 晶片上的實體位置：

- `row = 0` 與 `row = rows-1` 為頂部與底部的 I/O Tile 列
- `row = 1` 至 `row = rows-2` 為 Logic Tile 區域
- `col = 0` 與 `col = cols-1` 為左邊界與右邊界的 I/O Tile 行
- 中間的 `col` 值為 Logic Tile

這種佈局在 `v2f-pnr/src/arch.rs:42-48` 的 `ArchGraph::new()` 中定義：

```rust
let tile_type = if r == 0 || r == total_rows - 1 {
    TileType::Io
} else if c == 0 || c == total_cols - 1 {
    TileType::Io
} else {
    TileType::Logic
};
```

## ASC Wiring 到 CRAM Frame Bit 的映射

ASC 中的 `wiring` 條目需要映射到 CRAM 中實際的 frame 位元位置。這個映射由 `v2f-bitstream/src/asc.rs` 的 `apply_asc_to_cram()` 函數（第 277-342 行）處理：

### Logic Tile 的 wiring 映射

每個 Logic Tile 佔用 7 個 frame。`bit_index` 的編碼方式為：
- 高 bit 部分：`frame_within_tile`（0..6），決定該 bit 屬於 7 個 frame 中的哪一個
- 中間 bit 部分：`word`（0..32），決定該 bit 屬於 33 個 word 中的哪一個
- 低 bit 部分：`bit_within_word`（0..39），決定該 bit 屬於 word 內 40 個 bit 中的哪一個

計算邏輯（第 288-291 行）：

```rust
let frame_sub = (w.bit_index / (FRAME_BITS / 7)) % 7;
let remaining = w.bit_index % (FRAME_BITS / 7);
let word = remaining / 40;
let bit = remaining % 40;
```

### I/O Tile 的 wiring 映射

每個 I/O Tile 佔用 3 個 frame，計算方式類似但除數不同（第 311-313 行）：

```rust
let frame_sub = (w.bit_index / (FRAME_BITS / 3)) % 3;
let remaining = w.bit_index % (FRAME_BITS / 3);
let word = remaining / 40;
let bit = remaining % 40;
```

### CramAddrMap 解析

有了 `(tile_pos, tile_type, frame_sub, word, bit)` 之後，透過 `CramAddrMap::resolve()`（`v2f-db/src/cram_addr.rs:35-48`）計算出絕對的 frame 編號：

```rust
pub fn resolve(&self, pos: &TilePos, tile_type: TileType,
               frame_within_tile: u32, word: u32, bit: u32) -> CramAddr {
    let start = self.tile_start_frame(pos, tile_type);
    CramAddr { frame: start + frame_within_tile, word, bit }
}
```

其中 `tile_start_frame()`（第 27-32 行）計算該 tile 在 CRAM 中的起始 frame：

```rust
pub fn tile_start_frame(&self, pos: &TilePos, tile_type: TileType) -> u32 {
    let frames_per_row = self.device.frames_per_row();
    let row_offset = pos.row * frames_per_row;
    let col_offset = pos.col * tile_type.num_frames();
    row_offset + col_offset
}
```

CRAM 採用 Row-major 排列：先掃完一列中所有 tile 的所有 frame，再換下一列。

### Bit 寫入

若 `w.value != 0`，則在對應 frame 中設定該 bit（第 300-304 行）：

```rust
if w.value != 0 {
    let bit_pos = (addr.word * 40 + addr.bit) as usize;
    f.set_bit(bit_pos);
}
```

### Synckey 寫入

SYNCKEY 被放置在 CRAM 的最後一個 frame 的固定位置（第 332-341 行），以 little-endian 4 位元組寫入 frame 的最高位元區域。

## CRAM 模型

CRAM (Configuration RAM) 是 FPGA 配置記憶體的抽象，定義於 `v2f-bitstream/src/cram.rs`：

```rust
pub struct Cram {
    pub device: Ice40Device,
    frames: Vec<Frame>,
}
```

每個 `Frame` 為 1320 bits（165 bytes），由 33 個 word 組成，每個 word 為 40 bits（`v2f-bitstream/src/frame.rs:6-9`）：

```rust
pub const FRAME_BITS: usize = 1320;
pub const FRAME_BYTES: usize = 165;
pub const WORDS_PER_FRAME: usize = 33;
pub const BITS_PER_WORD: usize = 40;
```

`Frame` 結構提供逐位元存取（`set_bit`、`get_bit`、`clear_bit`）以及 word 層級的讀寫（`set_word`、`get_word`）。

## 位元流打包

Cram 結構可透過 `pack_bitstream()`（`v2f-bitstream/src/pack.rs:37-56`）打包為標準 iCE40 位元流格式：

```
[0x00 × 32]           前導空白
[BitCount: u32 LE]    總位元數
[Frame0..FrameN]      每個 Frame = 165 bytes
[CRC32: u32 LE]       校驗碼
```

CRC-32 計算僅涵蓋 CRAM 資料部分（不含前導空白與位元數標頭），使用 IEEE 802.3 / PKZIP 標準多項式 `0xEDB88320`。

## 參考實作

- ASC 解析器：`v2f-bitstream/src/asc.rs`
- CRAM 模型：`v2f-bitstream/src/cram.rs`
- Frame 結構：`v2f-bitstream/src/frame.rs`
- 位元流打包：`v2f-bitstream/src/pack.rs`
- CRAM 位址映射：`v2f-db/src/cram_addr.rs`
- 裝置參數：`v2f-db/src/ice40.rs`
- Tile 類型：`v2f-db/src/tile.rs`
- ASC 輸出：`v2f-pnr/src/asc_out.rs`
- 架構網格：`v2f-pnr/src/arch.rs`
- ASC Fixture 檔案：`v2f-bitstream/_fixtures/minimal_hx1k.asc`、`v2f-bitstream/_fixtures/empty_hx1k.asc`
