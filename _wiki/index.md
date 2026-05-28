# eda4 Wiki Index

> 本 wiki 收錄 eda4 計劃相關的專有名詞說明，涵蓋 EDA 工具鏈、FPGA 架構、硬體描述語言、數位電路模擬、自訂晶片設計與核心演算法等領域。

## 入門概念

| 詞條 | 說明 |
|------|------|
| [EDA](eda.md) | 電子設計自動化 — 用軟體設計硬體的學科 |
| [FPGA](fpga.md) | 現場可程式化閘陣列 — 可重構硬體平台 |
| [ASIC](asic.md) | 應用專屬積體電路 — 全定製晶片設計 |
| [CMOS / MOSFET](cmos.md) | 互補式金氧半導體 — 數位電路的電晶體基礎 |
| [Verilog HDL](verilog.md) | 硬體描述語言 — 數位電路設計的 lingua franca |
| [RTL](rtl.md) | 暫存器傳輸層級 — 數位設計的事實標準抽象層 |

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
| [Synthesis / Techmap](synthesis.md) | 邏輯合成 — 將 HDL 轉換為閘級電路 |
| [Netlist / Yosys-JSON](netlist.md) | 網表 — 電路的圖形化中間表示 |
| [Place & Route](pnr.md) | 佈局與繞線 — 將邏輯放置到實體位置並連接 |
| [Standard Cell](standard_cell.md) | 標準單元 — ASIC 自動化設計的建構方塊 |
| [Floorplanning / PDN](floorplanning.md) | 晶片平面規劃與電源分佈網絡 |
| [Static Timing Analysis](static_timing_analysis.md) | 靜態時序分析 — 時序驗證的黃金標準 |
| [Clock Tree / CDC](clocking.md) | 時脈樹合成與跨時脈域設計 |
| [JTAG / SPI](jtag_spi.md) | FPGA 燒錄通訊協定 |
| [Design for Test](dft.md) | 可測試性設計 — 掃描鏈、邊界掃描、BIST |

## 自訂晶片設計

| 詞條 | 說明 |
|------|------|
| [OpenLANE / Open-Source ASIC 流程](openlane.md) | 從 RTL 到 GDSII 的開源 ASIC 設計流程 |
| [SPICE Simulation](spice.md) | 電晶體層級電路模擬 |
| [Formal Verification](formal_verification.md) | 形式驗證 — 等價性檢查與模型檢驗 |
| [RISC-V](riscv.md) | 開放指令集架構 |

## 核心演算法

| 詞條 | 說明 |
|------|------|
| [Technology Mapping](technology_mapping.md) | 將邏輯網表綁定至特定工藝庫單元 |
| [Optimal Tree Covering](optimal_tree_covering.md) | 以動態規劃進行動態樹覆蓋 |
| [Quine-McCluskey](quine_mccluskey.md) | 二層邏輯最小化的精確演算法 |
| [Espresso](espresso.md) | 啟發式邏輯最小化器 |
| [BDD](bdd.md) | 二元決策圖 — 標準布爾表示 |
| [ABC](abc.md) | 開放源碼邏輯合成與驗證系統 |
| [Simulated Annealing](simulated_annealing.md) | 模擬退火 — 佈局最佳化演算法 |
| [A* Routing](a_star_routing.md) | A* 路徑搜尋 — 繞線核心演算法 |
| [Maze Routing / Lee's Algorithm](maze_routing.md) | 基於波前的迷宮繞線演算法 |
| [PathFinder](pathfinder.md) | 協商式擁塞繞線 |
| [Global Routing](global_routing.md) | 全域繞線 — 資源分配與拥塞控制 |
| [Detail Routing](detailed_routing.md) | 詳細繞線 — 指定確切金屬軌跡 |
| [Partitioning (K-L / FM)](partitioning.md) | 電路分割 — Kernighan-Lin 與 Fiduccia-Mattheyses |
| [SAT Solver (DPLL / CDCL)](sat_solver.md) | 布爾可滿足性求解器 |
| [ATPG (D-Algorithm / PODEM / FAN)](atpg.md) | 自動測試向量生成 |
| [Retiming (Leiserson-Saxe)](retiming.md) | 穿越暫存器移動以優化時序 |

## Verilog2Rust 生態

| 詞條 | 說明 |
|------|------|
| [Verilog2Rust / ruHDL / rhdl](verilog2rust.md) | Verilog→Rust 翻譯器與模擬執行環境 |
| [Event-Driven Simulation](event_simulation.md) | 事件驅動數位電路模擬原理 |
| [Rust HDL DSL / fpga! macro](hdl_dsl.md) | Rust 嵌入式硬體描述領域專屬語言 |