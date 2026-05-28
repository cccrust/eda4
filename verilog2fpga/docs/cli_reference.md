# CLI 完整參考

## v2f — 主工具

```sh
cargo run -p v2f-cli -- [subcommand] [options]
# 或
./target/debug/v2f [subcommand] [options]
```

### 全域選項

無（所有選項都在各子命令中）。

---

### `v2f build` — 完整流程

從 Verilog 到可燒錄 BIN，一次完成。

```sh
v2f build <input.v> [options]
```

**必要參數：**
| 參數 | 說明 |
|------|------|
| `input.v` | 輸入的 Verilog 檔案路徑 |

**選項：**
| 參數 | 說明 | 預設值 |
|------|------|--------|
| `--device <id>` | 指定 FPGA 裝置（hx1k/hx4k/hx8k/lp1k/up5k）| `hx8k` |
| `--top <name>` | 頂層模組名稱 | 自動推斷 |
| `--output <prefix>` | 輸出檔案前綴 | `output` |
| `--backend <backend>` | 後端策略（見下方） | `auto` |
| `--lang <lang>` | 輸入語言（`verilog` 或 `rust`）| `verilog` |
| `--pcf <file>` | 約束檔案（nextpnr 格式，暫未實作）| — |

**後端選項：**

| 值 | 說明 |
|----|------|
| `auto` | 嘗試 yosys+nextpnr+icepack，失敗則 fallback pure-rust |
| `pure-rust` | 全程純 Rust（不需要外部工具）|
| `yosys` | yosys 綜合 → nextpnr PNR → icepack 打包 |
| `pnr-only` | yosys 綜合 → v2f-pnr（pure）→ v2f-bitstream |

**輸出檔案：**
- `<prefix>.json` — 網表
- `<prefix>.asc` — Tile 配置（如果 PNR 成功）
- `<prefix>.bin` — 可燒錄位元流

**範例：**

```sh
# 純 Rust 流程
./target/debug/v2f build examples/blinky/blinky.v \
    --backend pure-rust --output _out/blinky

# YOSYS 流程
./target/debug/v2f build examples/blinky/blinky.v \
    --backend yosys --device hx8k --top blinky \
    --output _out/blinky_yosys
```

---

### `v2f synth` — 邏輯綜合

只做綜合，輸出網表。

```sh
v2f synth <input.v> [options]
```

| 參數 | 說明 | 預設值 |
|------|------|--------|
| `input.v` | 輸入 Verilog 檔案 | — |
| `--output <file>` | 輸出 JSON 檔案 | `output.json` |
| `--top <name>` | 頂層模組名稱 | 自動推斷 |
| `--backend` | `rust` 或 `yosys` / `auto` | `auto` |

---

### `v2f pnr` — 佈局佈線

只做 PNR，輸出 ASC。

```sh
v2f pnr <input.json> [options]
```

| 參數 | 說明 | 預設值 |
|------|------|--------|
| `input.json` | 輸入網表（來自 synth）| — |
| `--output <file>` | 輸出 ASC 檔案 | `output.asc` |
| `--device <id>` | 指定 FPGA 裝置 | `hx8k` |
| `--backend` | `pure-rust` / `yosys` / `auto` | `auto` |

---

### `v2f pack` — 位元流打包

將 ASC 打包為 BIN。

```sh
v2f pack <input.asc> [options]
```

| 參數 | 說明 | 預設值 |
|------|------|--------|
| `input.asc` | 輸入 ASC 檔案 | — |
| `--output <file>` | 輸出 BIN 檔案 | `output.bin` |
| `--backend` | `rust`（純 Rust）或 `icepack`（外部工具）| `auto` |

**注意**：目前 `--backend rust` 預設使用 `hx8k`（即使 ASC 中有其他裝置）。如需精確控制，請使用 `--backend icepack`。

---

### `v2f prog` — 燒錄

燒錄 BIN 到 FPGA。

```sh
v2f prog <input.bin> [options]
```

| 參數 | 說明 | 預設值 |
|------|------|--------|
| `input.bin` | 輸入位元流檔案 | — |
| `--driver` | 燒錄驅動（見下方）| `auto` |

**驅動選項：**

| 值 | 說明 |
|----|------|
| `auto` | 嘗試 openFPGALoader，失敗則 iceprog |
| `mock` | 模擬 JTAG 燒錄（不回寫到硬體）|
| `spi` | 模擬 SPI Flash 燒錄（不回寫）|
| 其他 | 嘗試當作實際工具執行 |

---

### `v2f list-devices` — 列出支援的裝置

```sh
./target/debug/v2f list-devices
```

輸出：
```
支援的 iCE40 裝置:
  hx1k
  hx4k
  hx8k
  lp1k
  up5k
```

---

### `v2f check` — 檢查工具鍊狀態

```sh
./target/debug/v2f check
```

輸出：
```
  ✓ yosys: 已安裝
  ✓ nextpnr-ice40: 已安裝
  ✓ icepack: 已安裝
  ✗ openFPGALoader/iceprog: 未安裝
  ✓ v2f-synth (pure Rust): 已啟用
  ✓ v2f-pnr (pure Rust): 已啟用
  ✓ v2f-bitstream (pure Rust): 已啟用
  ✓ v2f-bitdecode (pure Rust): 已啟用
```

---

## v2f-bitdecode — 位元流解碼器

將 BIN 逆向解析為可讀的 JSON。

```sh
./target/debug/v2f-bitdecode <input.bin> [options]
# 或
cargo run -p v2f-bitdecode -- <input.bin> [options]
```

### 必要參數

| 參數 | 說明 |
|------|------|
| `input.bin` | 輸入位元流檔案 |

### 選項

| 參數 | 說明 | 預設值 |
|------|------|--------|
| `-o, --output <file>` | 輸出 JSON 檔案 | stdout |
| `-p, --pretty` | 格式化 JSON 輸出（4 空格縮排）| 緊湊單行 |

### 輸出格式（v2f-bitdecode-v2）

```json
{
  "format": "v2f-bitdecode-v2",
  "device": "hx8k",
  "decode_level": "wiring",
  "created_by": "v2f-bitdecode v0.9",
  "bin_source": "/path/to/input.bin",
  "crc_valid": true,
  "synckey": "0x12345678",
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

### 欄位說明

| 欄位 | 說明 |
|------|------|
| `device` | 自動偵測的 FPGA 裝置 |
| `decode_level` | 解碼深度：`wiring` = wiring 反向解碼完成 |
| `crc_valid` | CRC32 校驗是否通過 |
| `synckey` | 從最後一個 frame 讀出的 synckey（hex 字串）|
| `num_tiles` | 總 tile 數量（IO + Logic）|
| `tiles.*.used` | 該 tile 是否有設定（non_zero_words 非空）|
| `tiles.*.non_zero_words` | 每個 non-zero word 的 frame/word/value |
| `tiles.*.wiring` | 解碼出的 wiring bit_index 列表 |

### 範例

```sh
# 基本用法
./target/debug/v2f-bitdecode _out/blinky.bin

# 格式化輸出 + 寫入檔案
./target/debug/v2f-bitdecode _out/blinky.bin -p -o _out/blinky_decoded.json

# 結合 pack + decode
./target/debug/v2f pack examples/minimal.asc --output /tmp/t.bin --backend rust
./target/debug/v2f-bitdecode /tmp/t.bin -p | head -30
```