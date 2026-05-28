# verilog2fpga

純 Rust 實現的 iCE40 FPGA 工具鏈。將 Verilog 程式碼編譯為可燒錄到 Lattice iCE40 FPGA 的位元流。

## 支援的裝置

| 代號 | 全名 | 邏輯单元 | 備註 |
|------|------|-----------|------|
| `hx1k` | HX1K | 1,280 | |
| `hx4k` | HX4K | 3,520 | |
| `hx8k` | HX8K | 7,680 | 預設值 |
| `lp1k` | LP1K | 1,280 | 低功耗版本 |
| `up5k` | UP5K | 5,280 | UltraPlus |

## 工具：`v2f`

主程式 CLI，統一入口。

```sh
cargo run -p v2f-cli -- <subcommand> [options]
# 或直接執行編譯後的 binary
./target/debug/v2f <subcommand> [options]
```

### 子命令

#### `v2f build` — 完整流程

```
v2f build <input.v> [options]
```

將 Verilog 編譯為可直接燒錄的 `.bin`。

| 參數 | 說明 | 預設值 |
|------|------|--------|
| `--device <代號>` | 指定 FPGA 裝置 | `hx8k` |
| `--top <name>` | 頂層模組名稱 | 自動推斷 |
| `--output <prefix>` | 輸出檔案前綴 | `output` |
| `--backend <backend>` | 使用哪個後端 | `auto` |
| `--lang <lang>` | 輸入語言（`verilog` 或 `rust`） | `verilog` |

**Backend 選項：**

| 值 | 說明 |
|----|------|
| `auto` | 嘗試 yosys+nextpnr，否則 fallback pure-rust |
| `pure-rust` | 全程純 Rust（不需要外部工具） |
| `yosys` | 使用 yosys 做綜合、nextpnr 做 PNR、icepack 打包 |
| `pnr-only` | yosys 綜合 + pure-rust PNR |

**輸出：**
- `<prefix>.json` — 綜合後網表（cells、ports、connections）
- `<prefix>.asc` — 佈局佈線結果（Tile 配置 + 繞線）
- `<prefix>.bin` — 可燒錄的位元流

**範例：**

```sh
# 純 Rust 後端編譯（不需要安裝 yosys/nextpnr）
./target/debug/v2f build examples/blinky/blinky.v --backend pure-rust --output _out/blinky

# 使用外部工具（需先安裝 yosys、nextpnr、icestorm）
./target/debug/v2f build examples/blinky/blinky.v --backend yosys --output _out/blinky_yosys
```

#### `v2f synth` — 只做綜合

```
v2f synth <input.v> [options]
```

將 Verilog 轉換為 JSON 網表。

#### `v2f pnr` — 只做佈局佈線

```
v2f pnr <input.json> [options]
```

將網表執行 Place & Route，輸出 ASC。

#### `v2f pack` — 只打包

```
v2f pack <input.asc> --output output.bin [--backend rust|icepack]
```

將 ASC 打包為 BIN。`--backend rust` 使用純 Rust 實作，`--backend icepack` 使用 icestorm 的 `icepack` 工具。

#### `v2f prog` — 燒錄

```
v2f prog <input.bin> --driver <driver>
```

燒錄位元流到 FPGA。

| Driver | 說明 |
|--------|------|
| `mock` | 模擬 JTAG 燒錄（不需實際硬體） |
| `spi` | 模擬 SPI Flash 燒錄 |
| `auto` | 嘗試 openFPGALoader 或 iceprog |

#### `v2f list-devices` — 列出支援的裝置

#### `v2f check` — 檢查工具鍊

檢查外部工具（yosys、nextpnr、icepack）是否已安裝，並顯示 pure-rust 模組狀態。

---

## 工具：`v2f-bitdecode`

iCE40 位元流解碼器。將 `.bin` 反向解析為可讀的 JSON。

```sh
./target/debug/v2f-bitdecode <input.bin> [options]
```

| 參數 | 說明 |
|------|------|
| `-o, --output <file>` | 輸出 JSON 檔案（預設：stdout） |
| `-p, --pretty` | 格式化 JSON 輸出 |

**輸出格式（`v2f-bitdecode-v2`）：**

```json
{
  "format": "v2f-bitdecode-v2",
  "device": "hx8k",
  "decode_level": "wiring",
  "synckey": "0x12345678",
  "crc_valid": true,
  "num_tiles": 2100,
  "summary": {
    "logic_tiles_total": 1960,
    "logic_tiles_used": 1,
    "io_tiles_total": 140,
    "io_tiles_used": 0
  },
  "tiles": {
    "logic_2_1": {
      "type": "logic",
      "col": 2,
      "row": 1,
      "used": true,
      "non_zero_words": [
        {"frame_within_tile": 0, "word": 0, "value_hex": "0x1000000001", "bits_set": 2}
      ],
      "wiring": [
        {"bit_index": 0}
      ]
    }
  }
}
```

**解碼深度（`decode_level`）：**

| 等級 | 說明 |
|------|------|
| `raw` | 僅顯示原始 CRAM frame words |
| `wiring` | 還原 wiring 連線位址（目前實作至此） |
| `lut` | 可解出 LUT init 值（LUT init 編碼尚未實作） |

---

## 資料格式說明

### `.json` — 網表（Synthesis 輸出）

由 `v2f synth`（純 Rust）或 yosys 產生。描述電路的邏輯結構：

```json
{
  "modules": {
    "top": {
      "ports": { "clk": {"direction": "input", "bits": [0]}, "led": {...} },
      "cells": { "counter": {"type": "FDRE", "port_map": {...}} }
    }
  }
}
```

### `.asc` — 佈局佈線結果（PNR 輸出）

由 `v2f pnr` 或 nextpnr 產生。純文字格式，描述 Tile 級別的設定：

```verilog
## 最小 ASC 檔案
.module top
.io_tile 0 0
  .pad 0 clk
  .pad 1 led
.logic_tile 1 1
  .lut 0 0 0 0 0 "0123"
  .wiring 0 0 0 128
.synckey 0x12345678
```

常用指令：
- `.io_tile <col> <row>` — IO Tile 設定
- `.logic_tile <col> <row>` — Logic Tile 設定
- `.lut <output> <in0> <in1> <in2> <in3> <hex>` — LUT 設定
- `.wiring <bit_index> <col> <row> <value>` — 繞線設定
- `.synckey <hex>` — 同步金鑰

### `.bin` — 位元流（最終輸出）

由 `v2f pack` 產生。二進位格式，可燒錄到 FPGA：

```
[32 bytes: 0x00 預載空白]
[4 bytes: bit_count (LE, 總位元數)]
[N bytes: CRAM frames]
[4 bytes: CRC32]
```

每個 Frame = 165 bytes = 1320 bits = 33 words × 40 bits。

---

## 工具鍊流程圖

```
Verilog (.v)
    │
    ▼ [synth]
JSON (網表)
    │
    ▼ [pnr]
ASC (Tile 佈局 + 繞線)
    │
    ▼ [pack]
BIN (可燒錄位元流)
    │
    ▼ [bitdecode]
JSON (Tile 解析結果) ← v2f-bitdecode
```

| 步驟 | 純 Rust | 外部工具 |
|------|---------|---------|
| synth | ✅ v2f-synth | yosys |
| pnr | ✅ v2f-pnr | nextpnr-ice40 |
| pack | ✅ v2f-bitstream | icepack |
| prog | ✅ mock JTAG/SPI | openFPGALoader, iceprog |

---

## 目錄結構

```
verilog2fpga/
├── v2f-cli/          主 CLI 入口（build/synth/pnr/pack/prog）
├── v2f-core/          共享：Device enum、Config、V2fError
├── v2f-synth/         純 Rust Verilog 綜合 → JSON 網表
├── v2f-pnr/           純 Rust Place & Route → ASC
├── v2f-bitstream/     純 Rust CRAM 管理、打包
├── v2f-bitdecode/     位元流解碼器 BIN → JSON（v2f-bitdecode）
├── v2f-programmer/    JTAG / SPI Flash 燒錄（mock + 實際）
├── v2f-db/            iCE40 裝置資料庫（Tile 位置、CRAM 定址）
├── v2f-viz/           網表/ASC 視覺化工具
├── examples/          範例電路
│   ├── blinky/        LED 閃爍（.v + .pcf）
│   └── adder/         加法器（.v）
└── run.sh             完整流程 script
```

---

## 依賴工具（可選）

以下工具用於 `--backend yosys` 模式。若未安裝，會自動 fallback 到 pure-rust。

```sh
# macOS
brew install yosys icestorm nextpnr-ice40 openfpgaloader

# Linux (以 Debian/Ubuntu 為例)
# apt install yosys nextpnr-ice40 icestorm openfpgaloader
```

---

## 快速開始

```sh
# 建置
cargo build

# 完整流程（純 Rust，不需要外部工具）
./run.sh

# 或手動執行
cargo run -p v2f-cli -- build examples/blinky/blinky.v --backend pure-rust --output _out/blinky

# 查看輸出
ls -la _out/

# 燒錄（模擬）
./target/debug/v2f prog _out/blinky.bin --driver mock

# 檢查工縣鏈狀態
./target/debug/v2f check
```

---

## 測試

```sh
./test.sh          # cargo build && cargo test
./test_cross.sh    # 交叉驗證（v0.8 cross-validation）
./run.sh           # 完整 E2E pipeline
```