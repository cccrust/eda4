# iCE40 裝置支援

verilog2fpga 支援 Lattice iCE40 系列全部五款裝置。以下是詳細規格矩陣。

## 裝置規格

| 屬性 | HX1K | HX4K | HX8K | LP1K | UP5K |
|------|------|------|------|------|------|
| **邏輯單元（LC）** | 1,280 | 3,520 | 7,680 | 1,280 | 5,280 |
| **乘法器** | — | — | — | — | 16 個（16×16 DSP） |
| **最大 I/O** | 45 | 71 | 206 | 45 | 206 |
| **封裝** | QN32 / CT256 | BGA121 / TQ144 | BGA256 / CT256 | QN32 / TQ144 | BGA256 / SG48 |
| **PLL** | — | — | — | — | 2 個 |
| **RAM (kb)** | — | — | — | — | 1,024 |
| **SRAM** | 16 kb | 16 kb | 16 kb | 16 kb | 64 kb |
| **Logic Tile 數量** | 16 cols × 30 rows | 24 cols × 52 rows | 32 cols × 52 rows | 16 cols × 30 rows | 52 cols × 38 rows |
| **PNR 佈局** | 16L × 30H | 24L × 52H | 32L × 52H | 16L × 30H | 52L × 38H |

## CRAM 幀數量

每個裝置的 CRAM 幀（frame）總數決定 BIN 檔案大小和晶片辨識：

| 裝置 | 總 Frames | 每行 Frames | 幀大小 | BIT 檔案大小 |
|------|-----------|-------------|--------|-------------|
| HX1K | 6,624 | 221 | 165 B | ~1.09 MB |
| HX4K | 19,872 | 328 | 165 B | ~3.28 MB |
| HX8K | 22,176 | 341 | 165 B | ~3.66 MB |
| LP1K | 6,624 | 221 | 165 B | ~1.09 MB |
| UP5K | 27,696 | 442 | 165 B | ~4.58 MB |

**幀大小固定**：所有 iCE40 裝置都是 165 bytes/frame = 1,320 bits/frame。

## Tile 組織

### Logic Tile（HX8K 為例）

- **數量**：32 cols × 52 rows = 1,664 tiles
- **每 Tile 幀數**：7
- **總 Logic 幀**：1,664 × 7 = 11,648

每個 Logic Tile 包含：
- 4 個 Logic Cells（LC），每個含 1 個 LUT4 + 1 個 DFF
- 局部互連矩陣（routing switch matrix）
- 攜帶鏈（carry chain）邏輯

### IO Tile

- **左邊 IO**：num_rows 個，位於 col = 0
- **右邊 IO**：num_rows 個，位於 col = num_cols + 1
- **每 Tile 幀數**：3

### BRAM / DSP Tile（僅特定裝置）

- **HX4K / LP1K**：無
- **UP5K**：有 DSP tile（col 6, 19）

## 幀尋址（Frame Addressing）

CRAM 幀的排列方式（從頂到底、從左到右）：

```
Frame 0         : row=0 全部 IO（左側）  - 3 幀
Frame 3         : row=1 全部 IO（左側）  - 3 幀
...
Frame [IO/row]  : row=MAX 左側 IO 尾
Frame [+1]      : row=0 logic col=0-31      - 7 幀 × 32 cols
Frame [+1+7*32] : row=1 logic col=0-31
...
Frame [IO+Logic] : row=0 全部 IO（右側）  - 3 幀
```

`CramAddrMap::tile_start_frame(pos, tile_type)` 實作此映射。

## v2f-core 中的裝置定義

```rust
pub enum Device {
    HX1K,   // 預設（?）
    HX4K,
    HX8K,   // v2f CLI 預設
    LP1K,
    UP5K,
}
```

```rust
impl Device {
    pub fn num_rows(&self) -> u32
    pub fn num_cols(&self) -> u32
    pub fn total_frames(&self) -> u32
    pub fn frames_per_row(&self) -> u32
    pub fn name(&self) -> &'static str
    pub fn nextpnr_flag(&self) -> &'static str  // --hx8k, --lp1k 等
}
```

## 支援限制

| 功能 | HX1K | HX4K | HX8K | LP1K | UP5K |
|------|:----:|:----:|:----:|:----:|:----:|
| Pure-Rust PNR | ✅ | ✅ | ✅ | ✅ | ✅ |
| 燒錄（JTAG）| ✅ | ✅ | ✅ | ✅ | ✅ |
| 燒錄（SPI）| ❌ | ❌ | ✅ | ❌ | ✅ |
| BRAM | ❌ | ❌ | ❌ | ❌ | ❌ |
| DSP | ❌ | ❌ | ❌ | ❌ | ❌ |
| yosys backend | ✅ | ✅ | ✅ | ✅ | ✅ |
| nextpnr backend | ✅ | ✅ | ✅ | ✅ | ✅ |

> **注意**：BRAM 和 DSP 尚未實作。目前 `v2f-synth` / `v2f-pnr` 只能處理純邏輯電路。FPGA 上的 Block RAM 和 DSP 區塊屬於「未來功能」。