# OpenLANE 開源 ASIC 設計流程

## 什麼是 OpenLANE

OpenLANE 是一套開源的自動化 ASIC 設計流程，由 Efabless 公司發起並維護。它整合了多個開源 EDA 工具，形成一條從 RTL 程式碼到 GDSII 遮罩數據的完整工具鏈。OpenLANE 最初針對 SkyWater 130nm 製程（SKY130）進行最佳化，是目前最成熟的開源數位 ASIC 設計流程之一。

OpenLANE 的核心目標是降低 ASIC 設計的進入門檻。在過去，晶片設計需要昂貴的商業 EDA 工具授權（如 Synopsys、Cadence、Siemens EDA），數百萬美元的費用讓個人開發者與小型團隊難以參與。OpenLANE 與 SkyWater 開放式 PDK 的結合，使得任何人都能免費進行 ASIC 設計，並透過 Google 的 Open MPW 計畫將設計送到半導體代工廠製造。

OpenLANE 本身不是一個單一的工具，而是一個腳本驅動的流程控制器。它使用 Tcl/Python 腳本依序呼叫各個後端工具，並在每個階段進行檢查與驗證。使用者只需提供 Verilog RTL 原始碼和 pin 約束檔案，OpenLANE 就能自動完成從邏輯合成到實體實現的整個流程。

## 歷史背景

OpenLANE 的起源可以追溯到 2019 年，當時 Efabless 與 Google 合作，希望建立一個完全開源的 ASIC 設計生態系統。SkyWater Technology 同意開放其 130nm 製程的 PDK，這在當時是前所未有的創舉。在此之前，半導體代工廠從未公開過完整的製程設計套件。

2020 年，Google 啟動了 Open MPW（Multi-Project Wafer） shuttle 計畫，免費為開源硬體專案製造晶片。Efabless 將 Caravel SoC 做為 MPW 的參考設計，並以 OpenLANE 做為官方推薦的設計流程。第一輪 Open MPW（MPW-1）於 2020 年 11 月截止投片，此後每幾個月就開啟一輪新的 shuttle 機會。

截至 2024 年，Open MPW 已經開放至第八輪（MPW-8），數百個來自全球的開源硬體專案透過此計畫成功製造並收到實體晶片。OpenLANE 也從最初的版本迭代到 OpenLANE 2.x 版本，使用了更現代的 CHIP Alliance 流程管理框架。

2023 年，Google 與 Efabless 進一步推出 Open Source Silicon 計畫，提供更多晶片製造名額與教育資源。SkyWater 也開放了 90nm 製程（SKY90 FD-SOI），為開源 ASIC 生態開闢了新的可能性。

## OpenLANE 完整流程

OpenLANE 的標準流程從 RTL 原始碼出發，經過多個階段最終產出 GDSII 遮罩數據。以下是各個階段的詳細說明。

### 合成階段（Synthesis）

流程的起點是 Verilog RTL 程式碼。OpenLANE 使用 Yosys 進行邏輯合成，將 RTL 轉換為閘級網表（gate-level netlist）。Yosys 支援 Verilog-2005 語法，並提供多種最佳化選項。

合成過程中，Yosys 會進行：
- 語法解析與層次化展開（hierarchy flattening）
- 邏輯最佳化（constant propagation、dead logic elimination）
- 技術對映（technology mapping）到 SKY130 標準單元庫
- 輸出 Verilog 網表與統計報告

Yosys 是 eda4 的 verilog2fpga 專案中純 Rust 合成工具的參考對象。兩者都處理 Verilog 解析與邏輯合成，但 Yosys 的功能更加完整，支援絕大多數的 Verilog 建構。

### 邏輯最佳化（ABC）

合成後的網表會傳遞給 ABC 工具進行進一步的邏輯最佳化。ABC 是由 UC Berkeley 開發的邏輯合成與驗證工具，專門處理布林函數的最佳化與技術對映。

ABC 在 OpenLANE 中的工作包括：
- 邏輯簡化（logic minimization）
- 路徑重定時（retiming）
- 扇入/扇出最佳化
- 面積與延遲的權衡最佳化
- 將邏輯對映到 SKY130 的標準單元庫

### 靜態時序分析（OpenSTA）

在合成的每個關鍵節點，OpenLANE 會透過 OpenSTA 進行靜態時序分析。OpenSTA 是一個開源的靜態時序分析工具，能夠檢查設計是否符合時序約束。

OpenSTA 讀取合成的網表、標準單元庫的時序模型（Liberty 格式）、以及使用者定義的時序約束（SDC 格式），計算所有路徑的建立時間（setup）與保持時間（hold）餘量。

### 平面規劃（Floorplanning）

由 OpenROAD 的平面規劃器負責。這個階段決定：
- 晶片核心面積（core area）
- IO pad 的位置與排列
- 電源網路（power grid）的初步規劃
- 宏單元（macro cell，如 SRAM）的擺放位置

OpenROAD 的平面規劃器使用隨機最佳化演算法來找到合理的佈局。使用者可以透過 config.tcl 設定晶片利用率（utilization rate）、長寬比（aspect ratio）等參數。

### 單元佈局（Placement）

平面規劃完成後，OpenROAD 進行標準單元的詳細擺放。這分為兩個階段：

- 全局佈局（global placement）：使用解析式方法（analytic placement）大致確定每個單元的位置
- 詳細佈局（detailed placement）：進行局部調整，消除重疊，滿足行對齊要求

RePLace 是 OpenROAD 中使用的全局佈局引擎，而 OpenDP 負責詳細佈局。

### 時脈樹合成（Clock Tree Synthesis, CTS）

時脈信號必須以接近相同的延遲到達所有觸發器（flip-flop），這就是 CTS 的目標。OpenROAD 的 TritonCTS 工具透過插入時脈緩衝器（clock buffer）來建立平衡的時脈分配網路。

CTS 的好壞直接影響晶片的最大工作頻率。時脈偏移（clock skew）過大會導致時序違例。

### 繞線（Routing）

繞線階段將標準單元之間的邏輯連接用金屬導線實現。OpenROAD 使用 TritonRoute 進行繞線，分為：

- 全局繞線（global routing）：規劃繞線通道與大致路徑
- 詳細繞線（detailed routing）：在每個金屬層上實際繪製導線

TritonRoute 支援多層金屬繞線（SKY130 提供 5 層金屬），並處理設計規則檢查（DRC）所要求的間距、寬度等限制。

### 佈局驗證與 GDSII 輸出

流程的最終階段包括：
- 使用 Magic 進行佈局編輯與寄生參數提取（parasitic extraction）
- 使用 Netgen 進行 LVS（Layout vs. Schematic）比對，確保佈局與原始網表一致
- 使用 KLayout 進行 DRC，確保符合代工廠的設計規則
- 輸出最終的 GDSII 檔案，這是交給代工廠進行光罩製作的標準格式

## 流程中的工具

### Yosys

Yosys 是基於開源架構的 Verilog 合成工具，由 Clifford Wolf 開發。它使用 C++ 編寫，支援大量的 Verilog-2005 語法，並提供豐富的直通（pass）架構來進行各種合成轉換。

Yosys 在 OpenLANE 中的作用是將行為層級的 RTL 轉換為閘級網表。它支援多種輸出格式，包括 Verilog、EDIF、BLIF 等。Yosys 也廣泛應用於 FPGA 開發流程（如 IceStorm、nextpnr），是開源 EDA 生態中最關鍵的工具之一。

值得一提的是，Yosys 與 eda4 的純 Rust 合成工具共用相似的目標：將 Verilog 原始碼轉換為邏輯連接網路。eda4 的實作可以參考 Yosys 的架構設計，尤其是在 AST 處理與模組層次化方面的設計模式。

### ABC

ABC 是由 UC Berkeley 的 Alan Mishchenko 領導開發的邏輯合成與驗證工具。它專注於布林函數的最佳化，提供大量的命令來操作邏輯閘級網路。

在 OpenLANE 中，ABC 主要用於合成後的邏輯最佳化，但也參與技術對映的工作。ABC 使用 Liberty 格式的單元庫描述來進行時序感知的對映最佳化。

### OpenROAD

OpenROAD 是一個整合的開源數位晶片實作工具，涵蓋從平面規劃到繞線的整個後端流程。它由多個子專案組成：

- OpenROAD-app：主應用程式與 Tcl 介面
- RePLace：全局佈局引擎
- OpenDP：詳細佈局引擎
- TritonCTS：時脈樹合成
- TritonRoute：詳細繞線引擎
- PDN（Power Distribution Network）：電源網路產生器

OpenROAD 使用 Tcl 作為主要的控制介面，與 OpenLANE 的 Tcl 腳本系統緊密整合。

### OpenSTA

OpenSTA 是基於開源的靜態時序分析引擎。它讀取設計網表、標準單元庫的 Liberty 時序模型、以及 SDC 時序約束，進行全面的路徑分析。

OpenSTA 會報告最差路徑（worst-case path）的建立時間與保持時間餘量，幫助設計者在流程早期發現時序問題。

### Magic

Magic 是由 UC Berkeley 開發的經典 VLSI 佈局編輯器，歷史可追溯到 1980 年代。儘管年代久遠，Magic 仍然是開源 ASIC 流程中不可或缺的工具。

在 OpenLANE 中，Magic 用於：
- 從 GDSII 讀取佈局進行可視化檢查
- 寄生參數提取（使用 magic_extract）
- 檢查標準單元的佈局完整性

### Netgen

Netgen 是開源的 LVS 工具，用於比對電路佈局與原始網表的邏輯等效性。它從 Magic 提取的電晶體級網表中重建邏輯連接，並與合成產生的閘級網表進行比對。

LVS 是晶片設計中至關重要的驗證步驟。如果 LVS 失敗，製造出來的晶片將無法正常運作。

### KLayout

KLayout 是一個高效率的 GDSII/OASIS 佈局檢視器與編輯器，由 Matthias Köfferlein 開發。在 OpenLANE 中，KLayout 主要負責 DRC 檢查。

KLayout 的 DRC 引擎使用 Ruby 腳本來描述設計規則，可以快速檢查大量的幾何違例。由於 KLayout 的效能優異，它已成為開源 EDA 流程中的標準 DRC 工具。

## SkyWater SKY130 PDK

SkyWater SKY130 是一個 130nm 的開源 PDK（Process Design Kit），由 SkyWater Technology 提供。這是史上第一個完全開源的商業半導體製程 PDK，採用 Apache 2.0 授權。

SKY130 提供：
- 數位標準單元庫（包含 combinational 與 sequential 單元）
- I/O 單元庫
- 類比元件（電晶體、電容、電阻、二極體）
- 5 層金屬互連
- MIM（Metal-Insulator-Metal）電容
- 1.8V / 3.3V 雙電壓操作

PDK 的開源不僅包含文件與模型，還包括標準單元的原始 GDSII 佈局、Spice 模型、Liberty 時序模型、以及 LVS/DRC 的 rule deck。

## Caravel SoC

Caravel 是一個基於 RISC-V 的 SoC（System-on-Chip）範本設計，由 Efabless 開發，專為 Google Open MPW shuttle 設計。

Caravel 包含：
- VexRiscv RISC-V MCU（RV32IMC）
- 多種周邊介面（SPI、UART、GPIO、I2C）
- Wishbone 匯流排架構
- 使用者專案區域（User Project Area, UPA）：提供標準化的接合介面，讓使用者可以在 MPW shuttle 中嵌入自己的設計

使用者只需將自己的 OpenLANE 設計嵌入 Caravel 的使用者專案區域，就能參與 MPW 投片。Caravel 的架構確保使用者的設計與 Caravel SoC 的其他部分不會互相干擾。

## Open MPW 計畫

Open MPW 是 Google 贊助的開源晶片製造計畫。每輪 shuttle 週期約 4-6 個月。

參與流程：
1. 設計者使用 OpenLANE 完成 RTL 到 GDSII 的流程
2. 將設計嵌入 Caravel SoC 範本
3. 提交 GDSII 到 Efabless 平台進行審查
4. Google 收集所有參與者的設計，合併成一個 multi-project wafer
5. SkyWater 進行晶圓製造（約 3-4 個月）
6. 參與者收到封裝好的晶片樣品

每一輪 Open MPW 通常支援 40-50 個設計。Google 至今已資助製造數百個開源晶片設計。

## 設定系統（config.tcl）

OpenLANE 使用 Tcl 腳本進行流程設定。主要的設定檔案是 config.tcl，其中包含以下類別的參數：

### 合成設定
- `SYNTH_STRATEGY`：合成策略（AREA、DELAY 等）
- `SYNTH_MAX_FANOUT`：最大扇出限制
- `SYNTH_BUFFERING`：是否插入緩衝器

### 平面規劃設定
- `FP_CORE_UTIL`：核心利用率
- `FP_ASPECT_RATIO`：晶片長寬比
- `FP_IO_MODE`：IO pad 模式

### 佈局設定
- `PL_TARGET_DENSITY`：目標佈局密度
- `PL_TIME_DRIVEN`：是否啟用時序驅動佈局

### CTS 設定
- `CLOCK_PERIOD`：目標時脈週期
- `CLOCK_PORT`：時脈埠名稱

### 繞線設定
- `ROUTING_STRATEGY`：繞線策略

這些參數提供了靈活的流程控制空間，讓設計者可以根據專案需求調整流程行為。

## GDSII 輸出

GDSII（Graphic Design System II）是半導體工業的標準遮罩數據格式，由 Calma 公司在 1970 年代開發。儘管格式古老，它至今仍是晶圓廠接受的標準格式。

GDSII 以二元樹狀結構描述幾何圖形（多邊形、路徑、圓弧），並透過階層化單元引用來減少檔案大小。OpenLANE 的最終輸出就是 GDSII 檔案，可直接提交給晶圓廠進行光罩製作。

## 驗證步驟

### 等效性檢查（Equivalence Checking）

OpenLANE 支援使用 Synlig 與 Yosys 進行等效性檢查。Synlig 是 Yosys 的一個分支，增加了對 SystemVerilog 的支援。等效性檢查確保合成前後的設計邏輯一致。

### 靜態時序分析

如前所述，OpenSTA 在流程的多個節點進行 STA，確保設計滿足時序要求。

### 形式驗證

SymbiYosys（sby）是基於 Yosys 的開源形式驗證工具。它使用 BMC（Bounded Model Checking）與 induction 方法來驗證設計的屬性。OpenLANE 可以整合 sby 來進行正式的屬性檢查。

## 與 IceStorm 流程的比較

IceStorm 是 Lattice iCE40 FPGA 的開源工具鏈，也是 eda4 專案中 verilog2fpga 工具的參考目標。OpenLANE 與 IceStorm 有許多相似與相異之處。

### 相同點
- 都是完全開源的數位設計流程
- 都使用開源工具取代商業軟體
- 都支援 Yosys 做為合成引擎
- 都支援從 RTL 到最終產物的自動化流程
- 都有活躍的開源社群支援

### 差異點
- **目標平台**：IceStorm 針對 FPGA（iCE40 系列），OpenLANE 針對 ASIC（SKY130）
- **最終產物**：IceStorm 輸出 CRAM 位元流（bitstream）來配置 FPGA；OpenLANE 輸出 GDSII 遮罩數據來製造晶片
- **工具鏈**：IceStorm 使用 nextpnr 進行佈局繞線，OpenLANE 使用 OpenROAD
- **時序**：FPGA 的時序是固定的（查表式 LUT），ASIC 的時序需要從標準單元庫計算
- **成本**：FPGA 開發只需購買開發板；ASIC 開發需要透過 MPW shuttle 製造

### 互補性

對於一個目標為 ASIC 的設計流程（如 eda4），可以先在 IceStorm 支援的 FPGA 上進行原型驗證，再透過 OpenLANE 進行 ASIC 實現。兩者共享 Yosys 做為合成工具，因此修改 Verilog 原始碼後可以透過相同的合成流程分別導向不同的後端。

## 整合潛力

eda4 的 verilog2rust 專案可以將 Verilog 轉換為 Rust HDL 進行模擬驗證。經過模擬驗證的設計可以無縫地透過 OpenLANE 進行 ASIC 實現。

具體的整合路徑：
1. 使用 verilog2rust 將 Verilog 設計轉換為 Rust 進行功能驗證
2. 使用 verilog2fpga 將相同設計合成到 iCE40 FPGA 進行原型驗證
3. 使用 OpenLANE 將設計轉換為 GDSII 進行 ASIC 製造

這樣的流程讓設計者可以在投入昂貴的晶片製造之前，先用 FPGA 進行完整的硬體驗證。

## RTL-to-GDSII 概念

RTL-to-GDSII 是 ASIC 設計流程的總稱，描述從行為層級的暫存器傳輸級（RTL）描述到最終物理層級的 GDSII 遮罩數據的完整轉換過程。

這個過程包含三個主要的抽象層轉換：
1. **行為層（Behavioral）→ 結構層（Structural）**：RTL 合成為閘級網表
2. **結構層（Structural）→ 幾何層（Geometric）**：物理設計，包括佈局與繞線
3. **驗證層（Verification）**：確保每個階段的轉換都是正確的

OpenLANE 實現了上述所有階段的自動化轉換，是當前最成熟的開源 RTL-to-GDSII 解決方案。

## 延伸閱讀

- [Application-Specific Integrated Circuit (Wikipedia)](https://en.wikipedia.org/wiki/Application-specific_integrated_circuit)
- [GDSII (Wikipedia)](https://en.wikipedia.org/wiki/GDSII)
- [Open-source Silicon (Wikipedia)](https://en.wikipedia.org/wiki/Open-source_silicon)
