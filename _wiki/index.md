# eda4 Wiki Index

> 本 wiki 收錄 eda4 計劃相關的專有名詞說明，涵蓋 EDA 工具鏈、FPGA 架構、硬體描述語言、數位電路模擬等領域。

## 入門概念

| 詞條 | 說明 |
|------|------|
| [EDA](eda.md) | 電子設計自動化 — 用軟體設計硬體的學科 |
| [FPGA](fpga.md) | 現場可程式化閘陣列 — 可重構硬體平台 |
| [Verilog HDL](verilog.md) | 硬體描述語言 — 數位電路設計的 lingua franca |

## FPGA 架構

| 詞條 | 說明 |
|------|------|
| [iCE40 Architecture](ice40.md) | Lattice iCE40 FPGA 家族，本計劃目標平台 |
| [ASC Format / Tile](asc_tile.md) | 佈局結果的 ASCII 表示法與 FPGA 磚塊架構 |
| [CRAM / Frame / Bitstream](cram_bitstream.md) | FPGA 組態記憶體與位元流格式 |
| [Logic Cell (LUT+FF+Carry)](logic_cell.md) | iCE40 邏輯單元 — LUT、正反器、進位鍵 |

## EDA 工具鏈

| 詞條 | 說明 |
|------|------|
| [Synthesis](synthesis.md) | 邏輯合成 — 將 HDL 轉換為閘級電路 |
| [Netlist / Yosys-JSON](netlist.md) | 網表 — 電路的圖形化中間表示 |
| [Place & Route](pnr.md) | 佈局與繞線 — 將邏輯放置到實體位置並連接 |
| [Simulated Annealing](simulated_annealing.md) | 模擬退火 — 佈局最佳化演算法 |
| [A* Routing](a_star_routing.md) | A* 路徑搜尋 — 繞線核心演算法 |
| [JTAG / SPI](jtag_spi.md) | FPGA 燒錄通訊協定 |

## Verilog2Rust 生態

| 詞條 | 說明 |
|------|------|
| [Verilog2Rust / ruHDL / rhdl](verilog2rust.md) | Verilog→Rust 翻譯器與模擬執行環境 |
| [Event-Driven Simulation](event_simulation.md) | 事件驅動數位電路模擬原理 |
| [Rust HDL DSL / fpga! macro](hdl_dsl.md) | Rust 嵌入式硬體描述領域專屬語言 |
