# 工具鍊：外部工具 vs 純 Rust 後端

verilog2fpga 的核心設計理念是**雙軌並行**：支援完全不依賴外部工具的純 Rust 流程，同時允許用戶在需要時切换到成熟的開源工具（yosys、nextpnr、icestorm）以獲得更好的合成品質。

## 工具對照表

| 步驟 | 純 Rust | 外部工具 |
|------|---------|---------|
| **綜合** | `v2f-synth` | yosys |
| **佈局佈線** | `v2f-pnr`（模擬退火）| nextpnr-ice40 |
| **打包** | `v2f-bitstream` | `icepack`（icestorm）|
| **燒錄** | `v2f-programmer`（mock）| openFPGALoader / iceprog |

## 外部工具

### YOSYS

開源的 Verilog 合成工具，支援SystemVerilog 和精確的 ASIC 風格優化。

**安裝（macOS）：**
```sh
brew install yosys
```

**在 verilog2fpga 中的角色：**
- 將 Verilog 電路轉換為 JSON 網表
- 支援完整 Verilog-2005 語法（含 `generate`、`for`、`function` 等）
- 可處理三態匯流排、雙向埠等高階結構
- 輸出與 `v2f-synth` 相同的 JSON 格式，可交叉比對

### Nextpnr-ice40

開源的 FPGA PNR 工具，支援 iCE40 全系列。

**安裝（macOS）：**
```sh
brew install nextpnr-ice40
# 或從源碼編譯（支援最新裝置）
brew tap siliconwitchery/oss-fpga
brew install --HEAD siliconwitchery/oss-fpga/nextpnr-ice40
```

**在 verilog2fpga 中的角色：**
- 將 JSON 網表轉換為 ASC Tile 配置
- 支援基於 timing-driven 的 PNR
- 輸出 `.asc` 文字格式，包含完整的 LUT init、FF、routing 配置

**注意**：nextpnr 輸出的 ASC 可能包含 `v2f-pnr` 無法處理的項目（例如 `multiply driven net` 錯誤）。此時 `v2f build --backend auto` 會自動跳过 PNR 而使用 pure-rust。

### icestorm（icepack）

Project IceStorm 的工具鏈，包括 `icepack`（打包）和 `iceprog`（燒錄）。

**安裝（macOS）：**
```sh
brew install icestorm
```

**在 verilog2fpga 中的角色：**
- `icepack`：將 ASC 打包為 BIN（比 `v2f-bitstream` 更完整，處理 LUT init）
- `iceprog`：燒錄到實際 FPGA 硬體

## Pure-Rust 後端的限制

### v2f-synth

| 功能 | 支援 | 備註 |
|------|------|------|
| 基本語法（module、input、output）| ✅ | |
| `reg` + 時序邏輯（`always @(posedge clk)`）| ✅ | |
| `assign` + 組合邏輯 | ✅ | |
| `if` / `case` | ✅ | |
| 算術運算（`+`, `-`）| ✅ | |
| 比較器、 多選器 | ✅ | |
| 三態匯流排（`tri`）| ❌ | 會被忽略 |
| 雙向埠（`inout`）| ❌ | 會被忽略 |
| 延遲（`#1`）| ❌ | 被忽略 |
| `generate` 區塊 | ❌ | 不支援 |
| `function` / `task` | ❌ | 不支援 |
| 參數化模組（`#(parameter P=8)`）| ⏳ | 部分支援 |

### v2f-pnr

| 功能 | 支援 | 備註 |
|------|------|------|
| 模擬退火 PNR | ✅ | |
| Logic + IO Tile 配置 | ✅ | |
| 隨機放置 + 迭代優化 | ✅ | |
| Timing-driven PNR | ❌ | 僅考慮線長 |
| 完整的 LUT routing | ⏳ | 只處理基本互連 |
| BRAM / DSP 放置 | ❌ | 尚未實作 |

### v2f-bitstream

| 功能 | 支援 | 備註 |
|------|------|------|
| Wiring bits 編碼 | ✅ | |
| ASC → CRAM 轉換 | ✅ | |
| CRC32（IEEE 802.3）| ✅ | 與 icestorm 完全相容 |
| 標準 BIN 格式 | ✅ | 與 icepack 完全相容 |
| LUT init 值編碼 | ❌ | 需要 icestorm bit mapping |
| FF 配置編碼 | ❌ | 需要 icestorm bit mapping |
| Carry chain 編碼 | ❌ | 需要 icestorm bit mapping |

## 選擇建議

**使用 Pure-Rust 的時機：**
- 學習或實驗 FPGA 電路設計
- 在沒有安裝外部工具的環境中工作
- 想要離線也能跑完整流程
- 想要測試電路的基本功能（不追求最佳化）

**使用 yosys / nextpnr 的時機：**
- 設計複雜電路，需要更好的綜合品質
- 需要精確的 timing 分析
- 設計包含 BRAM / DSP 的電路
- 需要處理三態匯流排、雙向埠等進階結構
- 需要與商用工具比對驗證

## 交叉驗證測試

`test_cross.sh` 執行三方比對：

```
JSON 層：v2f-synth vs yosys
  → 比對 cell 數量、port 結構（結構等效性，非 binary identical）

ASC 層：v2f-pnr vs nextpnr
  → 比對 Tile 數量、使用的 Tile 位置
  → 自動跳过（SKIP）當 nextpnr 報錯時

BIN 層：v2f-bitstream 內部一致性
  → CRC 驗證
  → Preamble 驗證
  → Deterministic 驗證（相同輸入產生相同輸出）
```

> **注意**：由於 `icepack` 的 BIN 格式與 `v2f-bitstream` 的輸出在 CRAM 位元編排上存在差異（主要在 LUT init 值的映射），BIN 層不做與 icepack 的 binary 比對。只驗證 `v2f-bitstream` 自身的輸出是否一致。