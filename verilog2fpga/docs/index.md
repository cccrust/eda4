# verilog2fpga 文件總覽

verilog2fpga 是純 Rust 實現的 iCE40 FPGA 工具鏈。將 Verilog 程式碼編譯為可在 Lattice iCE40 FPGA 上運行的位元流。

## 文件結構

| 檔案 | 說明 |
|------|------|
| [index.md](index.md) | 快速開始與建置說明 |
| [architecture.md](architecture.md) | 系統架構、crate 依賴關係 |
| [file_formats.md](file_formats.md) | ASC / BIN / JSON 格式詳解 |
| [pipeline.md](pipeline.md) | 端到端工具鍊流程 |
| [devices.md](devices.md) | iCE40 裝置支援矩陣 |
| [cram_layout.md](cram_layout.md) | CRAM 結構、frame layout |
| [cli_reference.md](cli_reference.md) | v2f / v2f-bitdecode CLI 完整參考 |
| [toolchain.md](toolchain.md) | 外部工具與純 Rust 後端比較 |

## 快速開始

```sh
# 建置整個專案
cargo build

# 執行完整 E2E 測試（需要 yosys/nextpnr/icestorm）
./run.sh

# 純 Rust 流程（不需要外部工具）
cargo run -p v2f-cli -- build examples/blinky/blinky.v \
    --backend pure-rust --output _out/blinky
```

## 支援的 FPGA 裝置

- **iCE40 HX1K** — 1,280 LC（實驗用途）
- **iCE40 HX4K** — 3,520 LC
- **iCE40 HX8K** — 7,680 LC（預設）
- **iCE40 LP1K** — 1,280 LC（低功耗）
- **iCE40 UP5K** — 5,280 LC（UltraPlus，含 DSP）

## 核心特性

- **純 Rust 實作**：不需要 yosys / nextpnr / icepack 即可完成完整流程
- **可選外部工具**：使用 `--backend yosys` 可调用成熟的開源工具提升品質
- **三方交叉驗證**：JSON / ASC 層使用結構等效性比對（非 binary identical）
- **完整測試覆蓋**：69+ 個 workspace tests，含 cross-validation

## 輸出格式

```
Verilog (.v) → [synth] → JSON (網表)
                        → [pnr]  → ASC (Tile 佈局 + 繞線)
                                  → [pack] → BIN (可燒錄位元流)
                                            → [bitdecode] → JSON (Tile 解析)
```

- **JSON**：`v2f synth` 輸出，描述電路邏輯結構（cells、ports）
- **ASC**：nextpnr 或 `v2f pnr` 輸出，Tile 級別文字配置
- **BIN**：`v2f pack` 或 `icepack` 輸出，可燒錄到 FPGA 的二進位格式