# 端到端工具鍊流程

verilog2fpga 將 Verilog 程式碼轉換為可在 iCE40 FPGA 上運行的位元流。以下是完整流程的詳細說明。

## 完整流程

```
Verilog (.v)
    │
    ├─── [synth] ──────────────────────────────┐
    │                                        │
    ▼                                        │
JSON (網表)                                   │
    │                                        │
    ├─── [pnr] ──────────────────────────────┤
    │                                        │
    ▼                                        │
ASC (Tile 佈局 + 繞線)                         │
    │                                        │
    ├─── [pack] ─────────────────────────────┤
    │                                        │
    ▼                                        │
BIN (可燒錄位元流) ────────────────────────────┤
                                             │
    └─── [prog] ──────────────────────────────┘
                                             │
     ▼                                       │
  FPGA (燒錄)                                 │
                                             │
    └─── [bitdecode] ◄──────────────────────┘
              (可選：將 BIN 逆向解讀為 JSON)
```

## 步驟 1：Synthesis（綜合）

**目的**：將 Verilog 電路轉換為與技術無關的網表（JSON）。

### Pure-Rust 實作（v2f-synth）

輸入：Verilog 原始碼
輸出：JSON 網表

支援的語法：
- `module` / `input` / `output` / `wire` / `reg`
- `assign`（組合邏輯）
- `always @(posedge clk)`（時序邏輯）
- `if` / `case`（組合或時序）
- 基本的算術運算（`+`, `-`）
- 比較器（`==`, `!=`, `<`, `>`）
- 多選器（`? :`）
- 參數化模組

**限制**（純 Rust）：
- 不支援三態匯流排（`tri`, `tri0`, `tri1`）
- 不支援雙向埠（`inout`）
- 不支援 `force` / `release`
- 不支援延遲（`#delay`）

### YOSYS 實作

輸入：Verilog 原始碼
輸出：JSON 網表

```sh
yosys -p "read_verilog input.v; synth -json output.json"
```

---

## 步驟 2：Place & Route（佈局佈線）

**目的**：將邏輯網表映射到實際的 FPGA Tile 資源，並計算繞線。

### Pure-Rust 實作（v2f-pnr）

使用模擬退火演算法：

1. **初始放置**：將每個 cell 隨機分配到一個 Logic Tile
2. **代價函數**：計算網路延遲 + 線長估計
3. **迭代**：反覆移動 cell 到鄰近 Tile，根據溫度參數接受次優解
4. **輸出**：ASC 文字格式

### Nextpnr 實作

```sh
nextpnr-ice40 --hx8k --json input.json --pcf constraints.pcf --asc output.asc
```

nextpnr 輸出更完整的 ASC，包含 LUT 初始化值（`"0123"` 之類的 hex 字串）。

---

## 步驟 3：Pack（打包）

**目的**：將 ASC Tile 配置轉換為標準 iCE40 bitstream 格式（BIN）。

### Pure-Rust 實作（v2f-bitstream）

流程：
1. **解析 ASC**：`parse_asc()` 將 `.logic_tile`、`.wiring` 等指令轉換為 `AscFile`
2. **初始化 CRAM**：建立一個空的 `Cram`（所有位為 0）
3. **應用 ASC**：`apply_asc_to_cram()` 將 wiring 設定寫入 CRAM
   - Logic Tile：7 個 frame × (FRAME_BITS/7) bits/frame
   - IO Tile：3 個 frame × (FRAME_BITS/3) bits/frame
4. **寫入 synckey**：放在最後一個 frame 的最高位
5. **打包 BIN**：`pack_bitstream()` 加 preamble + bit_count + CRC

**BIN 格式**：`[32 zero][bit_count: u32 LE][frame_data][CRC32]`

### Icepack 實作

```sh
icepack input.asc output.bin
```

`icepack` 實作更完整的 LUT init 編碼，將 ASC 中的 `.lut` 值轉換為 CRAM bit 位置。

### 預設裝置

`v2f pack --backend rust` 預設使用 `hx8k`。如需其他裝置，需修改程式碼或使用 `--backend icepack`。

---

## 步驟 4：Program（燒錄）

**目的**：將 BIN 燒錄到 FPGA。

### Mock 燒錄（測試用）

```sh
./target/debug/v2f prog _out/blinky.bin --driver mock
```

使用 `v2f_programmer::Ice40Programmer::program_cram_jtag()`，僅模擬行為（不燒錄到實際硬體），驗證流程正確性。

### SPI Flash 燒錄（模擬）

```sh
./target/debug/v2f prog _out/blinky.bin --driver spi
```

模擬 SPI Flash 編程流程：erase → page program → readback verify。

### 實際燒錄

```sh
openFPGALoader -b iCE40HX1K _out/blinky.bin
# 或
iceprog _out/blinky.bin
```

---

## 步驟 5（可選）：Bitstream Decode

**目的**：將 BIN 逆向解析為可讀的 JSON，驗證位元流內容。

### Pure-Rust 實作（v2f-bitdecode）

```sh
./target/debug/v2f-bitdecode _out/blinky.bin --pretty
```

流程：
1. **Parse BIN**：`parse_bin()` 驗證 preamble + bit_count + CRC，重建 `Cram`
2. **Detect Device**：由 frame 總數自動辨識裝置
3. **Decode CRAM**：`decode_cram()` 將 frame 資料映射回 tile 結構
4. **Decode Wiring**：對每個 tile 的非零 word，反向計算 bit_index
5. **Decode Synckey**：從最後一個 frame 讀取 synckey
6. **Output JSON**：產生 `v2f-bitdecode-v2` 格式

**當前解碼深度**：`decode_level: "wiring"`
- ✅ 可解：wiring bit_index（反向重建）、synckey、CRC 有效性
- ⏳ 部分：CRAM raw words（非零 words 列表）
- ❌ 未實作：LUT init 值、FF 類型、IO pad mapping

---

## Backend 組合

| Backend | Synth | PNR | Pack | 需要工具 |
|---------|-------|-----|------|---------|
| `pure-rust` | v2f-synth | v2f-pnr | v2f-bitstream | 無 |
| `yosys` | yosys | nextpnr | icepack | yosys, nextpnr-ice40, icestorm |
| `pnr-only` | yosys | v2f-pnr | v2f-bitstream | yosys |
| `auto` | yosys → pure-rust fallback | nextpnr → pure-rust fallback | icepack → rust fallback | 無（全部可選） |

### 快速比較實驗

`v2f build` 預設使用 `pure-rust`，但相同電路可用 `yosys` 重新編譯後比對結果：

```sh
# Pure-Rust
./target/debug/v2f build examples/blinky/blinky.v \
    --backend pure-rust --output _out/blinky_pure

# YOSYS（需安裝）
./target/debug/v2f build examples/blinky/blinky.v \
    --backend yosys --output _out/blinky_yosys
```