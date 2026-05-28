# 系統架構

verilog2fpga 是一個 Rust workspace，包含 9 個 crate。以下是各 crate 的職責與依賴關係。

## Crate 依賴圖

```
v2f-core          ← 共享：Device、Config、V2fError（無底層依賴）
v2f-db            ← 依賴 v2f-core（iCE40 裝置資料庫、CRAM 定址）
     ↓
v2f-synth         ← 依賴 v2f-core（Verilog → JSON 網表）
v2f-bitstream     ← 依賴 v2f-db（CRAM 管理、frame、打包）
     ↓                  ↓
v2f-pnr           ← 依賴 v2f-core、v2f-db、v2f-synth（PNR → ASC）
v2f-cli           ← 依賴幾乎全部（統一 CLI）
     ↓
v2f-bitdecode     ← 依賴 v2f-bitstream、v2f-db（BIN → JSON 解碼）
v2f-programmer    ← 依賴 v2f-core（JTAG / SPI 燒錄）
v2f-viz           ← 依賴 v2f-synth 的 JSON（網表視覺化）
```

## 各 crate 職責

### v2f-core
共享基礎設施。包含：
- `Device` enum：HX1K / HX4K / HX8K / LP1K / UP5K
- `Config`：全域設定
- `V2fError` / `V2fResult`：錯誤處理

### v2f-db
iCE40 裝置資料庫。包含：
- `CramAddrMap`：Tile 位置 → CRAM frame 定址的映射
- `Ice40Device`：各裝置的 frame 數量、行列數等靜態參數
- `TileType` / `TilePos`：Tile 拓撲類型與座標

### v2f-synth
純 Rust Verilog 合成。將 `.v` 檔案解析為 JSON 網表。支援：
- Module / input / output / reg / wire / assign
- always 區塊（時序邏輯）
- 基本的算術運算

**輸出格式**：`{ "modules": { "top": { "ports": {...}, "cells": {...} } } }`

### v2f-pnr
純 Rust Place & Route。使用模擬退火演算法將網表映射到實際 Tile。

**輸入**：JSON 網表
**輸出**：ASC 文字格式（`.logic_tile`、`.io_tile`、`.wiring` 等）

### v2f-bitstream
CRAM 管理與位元流打包。包含：
- `Cram`：模擬 FPGA 內部配置 RAM（frame 陣列）
- `Frame`：1320-bit frame（33 words × 40 bits）
- `pack_bitstream()`：CRAM → 標準 iCE40 BIN 格式
- `apply_asc_to_cram()`：ASC → CRAM 轉換（寫入 wiring bits）

**BIN 格式**：`[32 zero][bit_count: u32 LE][frames][CRC32]`

### v2f-bitdecode
位元流解碼器（本次 v0.9 新增）。將 BIN 反向解析為 JSON：
- `parse_bin()`：BIN → CRAM + device + CRC 驗證
- `decode_cram()`：CRAM → tile iteration + wiring 反向解碼
- `decode_synckey()`：從最後一個 frame 讀取 synckey

**輸出格式**：`v2f-bitdecode-v2`（含 decode_level、synckey、wiring 陣列）

### v2f-programmer
燒錄工具鏈：
- JTAG 狀態機（`JtagStateMachine`）
- SPI Flash 模擬（`SpiFlash`）
- 選項：`mock`（測試用）和 `ftdi`（實際硬體）

### v2f-cli
統一 CLI 入口。實作 9 個子命令：
`build` / `synth` / `pnr` / `pack` / `prog` / `list-devices` / `check`

### v2f-viz
基於 `eframe` 的 GUI 工具。可視化 JSON 網表和 ASC 檔案。

## 雙軌後端設計

工具鏈支援兩條獨立的編譯路徑：

**Pure-Rust 路徑**（不需要外部工具）：
```
Verilog → v2f-synth → v2f-pnr → v2f-bitstream → BIN
```

**外部工具路徑**（使用成熟開源工具）：
```
Verilog → yosys → nextpnr-ice40 → icepack → BIN
```

兩條路徑的輸出都遵循相同的格式約定（ASC 的子集相容性、BIN 的標準格式），因此可以交叉比對驗證正確性。

## 測試架構

- **單元測試**：各 crate 內的 `#[cfg(test)]` 模組
- **整合測試**：`v2f-*/tests/*.rs`
- **交叉驗證**：`test_cross.sh`
  - JSON 層：比對 `v2f-synth` 與 yosys 輸出（結構等效，非 binary）
  - ASC 層：比對 `v2f-pnr` 與 nextpnr 輸出
  - BIN 層：內部一致性驗證（CRC + preamble + deterministic）
  - 由於 `icepack` 格式與 `v2f-bitstream` 不相容，BIN 層不做 binary identical 比對