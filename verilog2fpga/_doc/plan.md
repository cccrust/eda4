# verilog2fpga — Rust 開源 FPGA 工具鏈

## 緣起

根據 `eda/_doc/opensource.md` 所述，方案 A 的工作流為：

```
Verilog → Yosys (綜合) → Netlist → nextpnr (佈線) → ASCII → icepack → Bitstream
```

本計畫以 Rust 打造一套**純 CLI 工具鏈**，完整支援上述流程，並逐步將各環節
用 Rust 原生實作，降低對外部 C++ 工具（Yosys、nextpnr）的依賴。

---

## 架構總覽

```
verilog2fpga/          # Cargo workspace
├── v2f-cli/           #   CLI 入口 — 串接整個流程
├── v2f-synth/         #   邏輯綜合 (Yosys 包裝 / 純 Rust 綜合器)
├── v2f-pnr/           #   佈局佈線 (nextpnr 包裝 / 純 Rust P&R)
├── v2f-bitstream/     #   位元流打包 (icepack 純 Rust 替代)
├── v2f-programmer/    #   燒錄器 (OpenFPGALoader 包裝 / FTDI 直通)
├── v2f-core/          #   共用型別、網表表示、設定檔
├── v2f-db/            #   裝置資料庫 (iCE40 拓撲、LUT、IO 腳位)
└── v2f-rust/          #   Verilog → Rust (rhdl) 橋接至本工具鏈
```

### Crate 依賴圖 (由上往下)

```
v2f-cli
  ├── v2f-synth
  │     └── v2f-core
  ├── v2f-pnr
  │     └── v2f-core
  ├── v2f-bitstream
  │     └── v2f-db
  ├── v2f-programmer
  └── v2f-rust
        └── v2f-core
```

---

## 各 Crate 規劃

### 1. `v2f-core` — 共用核心

- `Netlist` 資料結構（線路網表、Cell、Port、Net）
- `Device` trait（描述 FPGA 架構介面）
- `Config` 設定載入（TOML 格式）

### 2. `v2f-synth` — 邏輯綜合

**階段一**：包裝 Yosys（subprocess 呼叫）

```
v2f-synth input.v
  → 呼叫 yosys -p "synth_ice40 -json output.json"
  → 輸出 JSON 格式網表
```

**階段二**：純 Rust 綜合器（精簡版）
- Verilog parser（參考 `verilog4` 既有實作）
- AST → 邏輯閘對應 (LUT, DFF, Carry)
- Tech mapping（對 iCE40 上下文的邏輯單元）

### 3. `v2f-pnr` — 佈局佈線

**階段一**：包裝 nextpnr（subprocess 呼叫）

```
v2f-pnr --json netlist.json --pcf constraints.pcf
  → 呼叫 nextpnr-ice40 --hx8k --json netlist.json --asc output.asc
  → 輸出 ASC 格式（ASCII 配置）
```

**階段二**：純 Rust 佈局佈線（精簡晶片專用）
- 使用 `v2f-db` 提供的裝置拓撲
- Simulated Annealing 放置演算法
- A* / PathFinder 繞線演算法

### 4. `v2f-bitstream` — 位元流打包

**純 Rust 實作**（無外部依賴）

- 解析 ASC → 內部 CRAM（Configuration RAM）表示
- 使用 `v2f-db` 的 iCE40 定址資訊
- 輸出 `.bin` 位元流（= `icepack` 功能）

```
ASC 格式架構：
  .io_tile  row col ioBank IO腳位設定
  .logic_tile row col 邏輯單元(LUT/FF/Carry)
  .connect   row col net 連線矩陣設定
  .syndkey  32位元 同步金鑰
```

### 5. `v2f-programmer` — 燒錄器

**階段一**：包裝 OpenFPGALoader（subprocess 呼叫）

```
v2f-programmer --write bitstream.bin
  → openFPGALoader -b ice40_generic bitstream.bin
```

**階段二**：純 Rust FTDI/JTAG 驅動（使用 `libftd2xx` 或 `hidapi`）

### 6. `v2f-cli` — 主程式入口

```sh
# 全部自動化 (呼叫外部工具)
v2f-cli build input.v --device hx8k --pcf constraints.pcf
# 逐步執行
v2f-cli synth input.v -o netlist.json
v2f-cli pnr netlist.json -o output.asc
v2f-cli pack output.asc -o bitstream.bin
v2f-cli prog bitstream.bin

# 純 Rust 模式（不依賴外部工具，精簡功能）
v2f-cli build --pure-rust input.v --device hx1k
```

### 7. `v2f-db` — 裝置資料庫

以 Lattice iCE40 為初始目標：

| 裝置 | LUT | BRAM | PLL | 封裝 |
|------|-----|------|-----|------|
| iCE40UP5K | 5280 | 4 | 1 | SG48, UWG30 |
| iCE40HX8K | 7680 | 4 | 2 | CT256, BG121 |
| iCE40HX4K | 3520 | 2 | 1 | BG121, TQ144 |
| iCE40LP1K | 1280 | 0 | 0 | VQ100 |

資料庫包含：
- 邏輯區塊拓撲（行列數）
- IO 腳位對應（Package pinout）
- CRAM 位址對應（Frame/Word 層級）
- 連線資源（Routing track 長度與切換點）

### 8. `v2f-rust` — Verilog → Rust (rhdl) 橋接

將 `verilog2rust` 的輸出整合至此工具鏈：
- 接受 rhdl 格式的 Rust 電路描述
- 輸出標準 Verilog（提供給 `v2f-synth`）或直接生成網表

---

## 開發階段

### Phase 1 — CLI 整合（外部工具呼叫）

- `v2f-core` 共用型別
- `v2f-cli` 框架（clap CLI）
- `v2f-synth` + `v2f-pnr` + `v2f-bitstream` 階段一包裝
- `v2f-programmer` 階段一包裝
- 完整 E2E 測試：合成 iCE40 範例電路

### Phase 2 — 純 Rust 位元流產生

- `v2f-db` iCE40 CRAM 資料庫
- `v2f-bitstream` 純 Rust 實作
- 驗證：比對 `icepack` 輸出

### Phase 3 — 純 Rust 綜合器（精簡）

- Verilog 子集解析
- iCE40 tech mapping
- 驗證：與 Yosys 輸出比對

### Phase 4 — 純 Rust P&R（精簡）

- Simulated Annealing 放置
- PathFinder 繞線（iCE40 特定資源圖）
- 驗證：與 nextpnr 輸出比對

### Phase 5 — 純 Rust 燒錄器

- FTDI/JTAG 驅動
- SPI flash 程式化

---

## 專案慣例

- 使用 **2024 edition**（與 `sql4`、`btree` 等 crate 一致）
- 原始碼註解使用 **繁體中文**
- 每個 crate 各自有 `test.sh`（build + test）
- `#![allow(dead_code)]` 置於 `v2f-core/src/lib.rs`

---

## 參考資源

- [Project IceStorm](https://github.com/YosysHQ/icestorm) — iCE40 逆向工程 (Verilog → ASC → bitstream)
- [Yosys](https://github.com/YosysHQ/yosys) — 開源 RTL 綜合
- [nextpnr](https://github.com/YosysHQ/nextpnr) — 開源 P&R
- [OpenFPGALoader](https://github.com/trabucayre/openFPGALoader) — 通用 FPGA 燒錄器
- [F4PGA (SymbiFlow)](https://f4pga.org/) — 全開源 FPGA 工具鏈聯盟
- [prjoxide](https://github.com/gatecat/prjoxide) — Lattice Oxide (iCE40 UltraPlus 新架構)
