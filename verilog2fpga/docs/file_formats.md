# 資料格式參考

verilog2fpga 使用三種主要資料格式：`.json`（網表）、`.asc`（Tile 佈局）、`.bin`（位元流）。本文件詳細說明各格式的結構與編碼。

## 1. JSON — 網表（Synthesis Output）

由 `v2f synth`（純 Rust）或 yosys 產生。描述電路的邏輯結構。

### 結構

```json
{
  "modules": {
    "top": {
      "ports": {
        "clk": { "direction": "input",  "bits": [0] },
        "led": { "direction": "output", "bits": [1] }
      },
      "cells": {
        "counter_0": {
          "type": "FDRE",
          "port_map": {
            "D":  { "bits": [2] },
            "Q":  { "bits": [3] },
            "CLK":{ "bits": [0] },
            "CE": { "bits": [] }
          }
        }
      },
      "netnames": {
        "counter[0]": { "bits": [4] }
      }
    }
  }
}
```

### 欄位說明

| 欄位 | 說明 |
|------|------|
| `modules.<name>.ports` | 頂層輸入/輸出埠 |
| `ports.<name>.direction` | `input` 或 `output` |
| `ports.<name>.bits` | 該埠的 wire 編號陣列 |
| `cells.<name>.type` | 細胞類型（`FDRE`、`LUT4`、`MUX` 等） |
| `cells.<name>.port_map` | 埠名到 bit 位置的映射 |
| `netnames.<name>.bits` | 命名的訊號的 wire 位置 |

---

## 2. ASC — Tile 佈局（PNR Output）

由 `v2f pnr` 或 nextpnr-ice40 產生。純文字格式，描述每個 Tile 的配置。

### 格式範例

```verilog
## 最小 ASC 檔案 — 一個 logic tile 含 LUT + 繞線
.module top
.io_tile 0 0
  .pad 0 clk
  .pad 1 led
.logic_tile 1 1
  .lut 0 0 0 0 0 "0123"
  .wiring 0 0 0 128
.synckey 0x12345678
```

### 指令參考

#### `.module <name>`
模組名稱（目前僅用於識別，無語意功能）。

#### `.io_tile <col> <row>`
IO Tile 設定。每一個 IO Tile 控制晶片边缘的多個 I/O pad。

```verilog
.io_tile 0 0
  .pad <index> <name>    # .pad 0 clk  將 pad 0 命名為 clk
  .wiring <bit_index> <col> <row> <value>   # 繞線設定
```

#### `.logic_tile <col> <row>`
Logic Tile（邏輯單元）設定。一個 Logic Tile 包含：
- 4 個邏輯單元（LC），每個含 1 個 LUT4 + 1 個 DFF
- 局部互連矩陣
- 攜帶鏈（Carry Chain）邏輯

```verilog
.logic_tile 1 1
  .lut <output> <in0> <in1> <in2> <in3> <hex>   # LUT 設定
  .wiring <bit_index> <col> <row> <value>        # 繞線設定
  .ff <output> <config>                           # DFF 設定
  .carry <output> <ci>                           # 攜帶鏈設定
```

**`.lut` 格式：**
```
.lut <output_port_num> <input0_port_num> <input1_port_num> <input2_port_num> <input3_port_num> "<4-char-hex>"
```
- `output` 到 `input3`：port 編號（0-31）
- 4-char hex：LUT 初始值（16 bits）。`"0123"` 表示 bits 0-15 對應值為 0x0, 0x1, 0x2, 0x3

**`.wiring` 格式：**
```
.wiring <bit_index> <col> <row> <value>
```
- `bit_index`：CRAM 內的位元索引（0-9239，用於 Logic Tile；0-3960，用於 IO Tile）
- `col`, `row`：目標位置（目前作為內部對齊欄位，通常填 0）
- `value`：設定值（非 0 表示 set，0 表示 clear）

#### `.synckey <hex>`
32-bit 同步金鑰。放在最後一個 CRAM frame 的最高位，用於識別 bitstream 版本或用戶資料。

### ASC 解析器（v2f-bitstream/asc.rs）

- `parse_asc()`：將 ASC 字串解析為 `AscFile` struct
- `apply_asc_to_cram()`：將 `AscFile` 應用到 `Cram`，寫入 wiring bits
- 支援 `.` 開頭的指令、`##` 註解、空行

---

## 3. BIN — 位元流（Pack Output）

由 `v2f pack` 或 `icepack` 產生。可燒錄到 FPGA 的二進位格式。

### 標準 iCE40 BIN 格式

```
Offset 0       : 32 bytes  preamble (all zeros)
Offset 32      : 4 bytes  bit_count (u32, little-endian, 總位元數)
Offset 36      : N bytes  frame_data (CRAM 內容)
Offset 36+N    : 4 bytes  CRC32 (IEEE 802.3, 僅計算 frame_data)
```

**總長度** = 36 + (bit_count / 8) + 4 bytes

### 範例：HX1K 位元組數

- HX1K：6,624 frames
- 每 frame：165 bytes = 1,320 bits
- bit_count = 6,624 × 1,320 = 8,743,680 bits
- frame_data = 8,743,680 / 8 = 1,092,960 bytes
- **總長** = 36 + 1,092,960 + 4 = 1,093,000 bytes

### Frame 結構

每個 CRAM frame = 1,320 bits = 165 bytes，分為 33 個 word：

```
Word 0  : bits [0:39]    (LSB first, 40 bits)
Word 1  : bits [40:79]
Word 2  : bits [80:119]
...
Word 32 : bits [1280:1319]
```

**注意**：每個 word 實際佔 5 bytes（小於 40 bits 的空間有浪費），總計 33 × 5 = 165 bytes。

### CRAM 組織（iCE40 HX1K 為例）

| 項目 | 說明 |
|------|------|
| 總行數 | 30 |
| 總列數 | 16 Logic + 2 IO（左+右）|
| 每 Logic Tile | 7 frames × 33 words × 40 bits |
| 每 IO Tile | 3 frames × 33 words × 40 bits |
| 總 frames | 30 × (16 + 2) × 7 / 30 + ... 複雜計算 |
| 預設值 | 0（SRAM 初始化為 0）|

**CRAM 地址計算**（`CramAddrMap::resolve`）：
```
frame = tile_start_frame(tile_pos, tile_type)
      + frame_sub
word = addr.word
bit = addr.bit
```

其中 `tile_start_frame` 的計算包含左側 IO tiles 的 frame 數。

### CRC32

使用標準 IEEE 802.3 CRC32（與 ZIP/PKZIP 相同）。計算範圍僅包含 frame_data（Offset 36 到 36+N-1）。

```rust
// 驗證實作（與 icestorm icepack 100% 相容）
fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFFFFFFu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}
```

### BIN 解析（v2f-bitdecode）

`parse_bin()` 實作：
1. 驗證前 32 bytes 為零（preamble）
2. 讀取 bit_count，計算 frame 數量
3. 透過 frame 總數自動偵測裝置（HX1K: 6,624 frames, HX8K: 22,176 frames 等）
4. 驗證長度 = 36 + payload + 4
5. 讀取並驗證 CRC32
6. 重建 Cram（33 words × 40 bits per frame）

---

## 格式轉換對照表

| 轉換 | 工具 | 說明 |
|------|------|------|
| Verilog → JSON | `v2f synth` / yosys | 語法分析 + 綜合 |
| JSON → ASC | `v2f pnr` / nextpnr | 模擬退火 PNR |
| ASC → CRAM | `apply_asc_to_cram()` | 寫入 wiring bits |
| CRAM → BIN | `pack_bitstream()` | 加 preamble + CRC |
| BIN → CRAM | `parse_bin()` | 驗證並還原 |
| CRAM → tiles | `decode_cram()` | Tile iteration |
| tiles → JSON | `tiles_to_json()` | v2f-bitdecode-v2 輸出 |