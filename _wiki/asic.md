# ASIC (特定應用積體電路)

## 什麼是 ASIC

ASIC (Application-Specific Integrated Circuit，特定應用積體電路) 是針對特定用途或應用而設計的客製化晶片，與通用晶片（如 CPU、GPU、記憶體）不同。ASIC 從設計之初就為單一任務最佳化，因此在效能、功耗、面積和成本上能達到最高效率。常見的 ASIC 範例包括比特幣礦機晶片、手機基頻晶片、AI 加速器（TPU/NPU）、以及各種物聯網感測器控制晶片。

ASIC 的核心概念是在「設計階段」而非「使用階段」決定晶片的功能。一旦製造完成，其電路結構便固定無法改變，這與 FPGA（現場可程式化閘陣列）可在出廠後重新配置的特性形成對比。

## 歷史發展

### TTL 7400 系列時代 (1960s-1970s)

早期的數位系統設計採用標準 TTL（電晶體-電晶體邏輯）7400 系列晶片，如 7400 (NAND)、7404 (NOT)、7474 (D flip-flop) 等。工程師透過 PCB 上焊接數十至數百顆獨立的 SSI/MSI 封裝晶片來實現系統功能。這種做法的缺點是體積龐大、功耗高、可靠度低，但優點是設計週期短且無一次性工程費用 (NRE)。

### 閘陣列 (Gate Array) (1980s)

閘陣列是半客製化 ASIC 的早期形式。晶圓廠預先生產包含大量未連接電晶體陣列的晶圓（基片），設計工程師只需定義金屬層的連接 pattern 來實現所需的邏輯功能。這種方法的 NRE 較低（僅需少數光罩層），但邏輯利用率較差，且效能不如全客製設計。隨著製程演進，閘陣列逐漸被標準細胞庫設計取代。

### 標準細胞 (Standard Cell) (1990s)

標準細胞設計法將預先設計好的邏輯閘（AND、OR、flip-flop 等）儲存在細胞庫中，設計者使用 EDA 工具自動進行邏輯合成、佈局與繞線 (PNR)。這種方法大幅提升了設計自動化程度，讓數百萬閘級的晶片設計成為可能。標準細胞設計在面積利用率與設計週期之間取得了良好平衡，至今仍是數位 ASIC 的主流方法。

### 全客製 (Full Custom) (1990s-present)

全客製設計由設計者手動控制每個電晶體的尺寸和佈局，以達到最佳的效能、功耗和面積。這種方法主要用於類比電路、記憶體單元（SRAM、DRAM）、I/O 電路和高效能微處理器核心。全客製設計的開發時間最長、成本最高，但能實現最高的效能和最小的晶片面積。

## 全客製 vs 半客製

### 全客製設計

全客製設計從零開始手動設計每個電晶體。設計者可以自由決定電晶體的尺寸、通道長度、摻雜濃度等參數。

- **優點**：最高效能、最低功耗、最小面積、可整合類比與數位電路
- **缺點**：極長的設計時間（數月至數年）、極高的設計成本（需大量資深工程師）、難以擴展至大規模設計
- **應用場景**：CPU 核心（如 ARM Cortex 客製化）、SRAM/DRAM 巨集、類比前端、高速 SerDes

### 半客製設計

半客製設計使用預先設計和驗證的建構區塊，透過 EDA 工具自動化流程完成設計。

**閘陣列 (Gate Array / MPGA)**：
- 預先生產包含電晶體陣列的基片，僅需客製金屬層
- NRE 低，適合中等產量（1K-100K 單位）
- 邏輯利用率約 60-80%，效能比標準細胞差約 20-30%

**標準細胞 (Standard Cell)**：
- 使用預先設計的邏輯細胞庫，透過合成和 PNR 自動佈局
- 必須定義所有光罩層，NRE 較高
- 邏輯利用率約 90% 以上，設計自動化程度高
- 適合大規模數位設計（數百萬至數十億閘級）

**結構化 ASIC (Structured ASIC)**：
- 介於閘陣列和標準細胞之間的方式
- 預先製造統一的邏輯層（如 LUT-based 陣列），僅客製金屬層
- 比閘陣列更好的效能和密度，比標準細胞更低的 NRE
- 在 2000 年代末期曾曇花一現，現在已較少使用

## ASIC 設計流程

ASIC 設計流程從規格定義到晶片量產，通常需要 12-24 個月，依設計複雜度而異。

### 1. 規格定義 (Specification)

定義晶片的功能、效能目標（時脈頻率、功耗預算）、介面協定、封裝形式、製程節點和成本目標。通常以 PDF 文件或設計意圖文件呈現。

### 2. 架構設計 (Architecture)

將規格轉化為系統架構，包括模塊分割、資料路徑設計、控制邏輯、記憶體層級、匯流排架構（如 AXI、Wishbone）。此階段使用 SystemC 或 Python 進行快速模型驗證。

### 3. RTL 設計 (Register-Transfer Level)

使用硬體描述語言（Verilog、SystemVerilog、VHDL）撰寫暫存器傳輸層級的邏輯描述。RTL 程式碼為後續合成流程的輸入。

### 4. 功能驗證 (Functional Verification)

透過模擬（simulation）和形式驗證（formal verification）確保 RTL 正確性。現代驗證方法包括：
- 動態模擬：使用 testbench 和 UVM (Universal Verification Methodology)
- 覆蓋率分析：碼涵蓋率、條件涵蓋率、翻轉涵蓋率
- 形式驗證：數學證明設計滿足斷言 (SVA)
- 模擬加速：使用硬體模擬器（emulator）或 FPGA 原型驗證

### 5. 邏輯合成 (Logic Synthesis)

使用合成工具（如 Synopsys Design Compiler、Cadence Genus）將 RTL 轉換為閘級網表 (gate-level netlist)。合成工具根據設計師設定的約束（時序、面積、功耗）從標準細胞庫中選擇合適的邏輯閘。

### 6. DFT 插入 (Design for Test)

在設計中插入測試結構以確保製造後的良率：
- 掃描鏈 (scan chain)：將 flip-flop 串接成移位暫存器，用於測試邏輯電路
- ATPG (Automatic Test Pattern Generation)：自動生成測試向量
- JTAG / IEEE 1149.1：邊界掃描測試
- MBIST (Memory Built-In Self-Test)：嵌入式記憶體自我測試

### 7. 平面規劃 (Floorplanning)

決定晶片中各模塊的實體位置，包括：
- I/O pad 佈局
- 巨集區塊 (macro) 放置
- 電源供應網絡 (PDN) 設計
- 晶片面積估算
- 封裝基板規劃

### 8. 佈局與繞線 (Place & Route, PNR)

使用 EDA 工具（如 Cadence Innovus、Synopsys ICC2）進行：
- 標準細胞佈局 (placement)：將邏輯閘放置在晶片上的最佳位置
- 時鐘樹合成 (clock tree synthesis, CTS)：建立低歪斜的時鐘分佈網路
- 繞線 (routing)：連接所有標準細胞和巨集區塊

### 9. 靜態時序分析 (Static Timing Analysis, STA)

在不依賴輸入向量的情況下，分析所有可能的信號路徑的時序：
- 檢查 setup/hold 時間是否滿足
- 分析時鐘歪斜 (clock skew) 和抖動 (jitter)
- 跨時域分析 (clock domain crossing, CDC)
- 使用 Synopsys PrimeTime、Cadence Tempus

### 10. 功耗分析 (Power Analysis)

- 動態功耗：因電容充放電產生的功耗
- 靜態功耗：漏電流導致的功耗
- 紅外線降分析 (IR drop)：檢查電源網絡電壓降
- 電遷移分析 (electromigration)

### 11. 物理驗證 (Physical Verification)

- DRC (Design Rule Checking)：檢查設計是否符合晶圓廠的製造規則
- LVS (Layout vs Schematic)：確保佈局與原始電路圖一致
- 天線效應檢查 (antenna rule)
- DFM (Design for Manufacturing)：提升良率的設計調整

### 12. 簽核 (Signoff)

在設計正式交付晶圓廠前完成所有簽核檢查：
- 時序簽核 (timing signoff)
- 功耗簽核 (power signoff)
- 物理簽核 (physical signoff)
- 訊號完整性簽核 (signal integrity signoff)

### 13. 光罩製作與晶圓製造 (Tapeout & Fabrication)

將最終 GDSII 檔案交付晶圓廠：
- 光罩製作 (mask making)：電子束直寫製作光罩組
- 晶圓製造 (wafer fabrication)：透過微影、蝕刻、沉積等製程生產晶圓
- 先進製程（7nm/5nm/3nm）需使用 EUV 微影技術

### 14. 封裝與測試 (Packaging & Test)

- 晶圓測試 (wafer sort / CP test)：在切割前測試每個晶粒
- 封裝 (packaging)：QFP、BGA、Flip-chip、Chiplet 等封裝方式
- 最終測試 (final test / FT)：封裝後進行完整功能測試和特性調整

## ASIC 與 FPGA 的關鍵差異

| 特性 | ASIC | FPGA |
|------|------|------|
| 效能 | 高（可達 5-10 GHz） | 中低（通常 100-500 MHz） |
| 功耗 | 低（無程式化開銷） | 較高（靜態功耗 + 程式開銷） |
| 單位成本（大量） | 很低（<$1） | 較高（$10-$10,000+） |
| NRE | 極高（$1M-$50M+） | 幾乎為零 |
| 設計週期 | 12-24 個月 | 1-6 個月 |
| 設計修改 | 不可（需重新 tapeout） | 可（重新燒錄） |
| 驗證難度 | 極高 | 中等 |
| 工具成本 | 極高（$100K-$1M+/年） | 便宜或免費（Vivado/Quartus） |
| 適合產量 | >1K-100K 單位 | <1K-100K 單位 |

### 產量交叉點

FPGA 在低產量時具有成本優勢（無 NRE），而 ASIC 在高產量時因較低的單位成本勝出。交叉點通常在 1,000 到 100,000 單位之間，具體取決於晶片複雜度和製程節點：

- 簡單設計（如感測器介面）：交叉點約 1K-10K 單位
- 中等複雜度（如微控制器）：交叉點約 10K-50K 單位
- 複雜設計（如 AI 加速器）：交叉點約 50K-100K+ 單位

## 光罩成本

光罩組 (mask set) 是 ASIC 生產中最大的 NRE 成本項目。每個光罩需要電子束直寫設備數小時至數十小時的寫入時間，且需使用高精密的石英玻璃基板。

| 製程節點 | 光罩層數 | 估計光罩成本 |
|----------|----------|-------------|
| 180nm | 20-30 | $50K-$100K |
| 130nm | 30-40 | $100K-$300K |
| 90nm | 40-50 | $300K-$500K |
| 65nm | 50-60 | $500K-$1M |
| 45nm | 60-70 | $1M-$2M |
| 28nm | 50-60 | $2M-$3M |
| 16nm/14nm | 60-80 | $3M-$6M |
| 7nm | 80-90 | $5M-$15M |
| 5nm | 90-100 | $10M-$30M+ |
| 3nm | 100+ | $20M-$50M+ |

若採用多專案晶圓 (Multi-Project Wafer, MPW) 方案（如 MOSIS、Europractice），可與其他設計共享光罩成本，但晶片面積和產量受到限制。

## 設計方法論

### 全客製設計方法

- **電晶體層級設計**：手動繪製電路圖和佈局，使用 Virtuoso (Cadence) 或 Custom Compiler (Synopsys)
- **類比電路設計**：需考慮製程變異、溫度效應、匹配特性
- **記憶體設計**：使用記憶體編譯器或手動設計 SRAM/ROM 巨集
- **客製數位電路**：用於高效能路徑（如 ALU、乘法器）以達到最高速度

### 標準細胞設計方法

- **RTL 驅動流程**：從 Verilog/VHDL 開始，透過自動化工具完成後續步驟
- **細胞庫**：使用晶圓廠提供的標準細胞庫（包含數百至數千種邏輯細胞）
- **設計約束**：透過 SDC (Synopsys Design Constraints) 檔案定義時序和功耗目標
- **自動化佈局繞線**：EDA 工具負責 placement 和 routing

### 閘陣列設計方法

- **基片預先製造**：晶圓廠備有庫存的預製基片
- **金屬客製化**：僅需定義 2-5 層金屬光罩
- **優點**：NRE 低、設計週期短（數週至數月）
- **缺點**：邏輯密度低、效能差

## EDA 工具的角色

ASIC 設計若無 EDA (Electronic Design Automation) 工具幾乎不可能完成。三大 EDA 供應商主導市場：

### Synopsys
- Synthesis：Design Compiler (DC)、Fusion Compiler
- PNR：IC Compiler II (ICC2)
- STA：PrimeTime
- DFT：DFTMAX、TetraMAX
- Formal Verification：Formality、VC Formal
- Custom Design：Custom Compiler

### Cadence
- Synthesis：Genus
- PNR：Innovus
- STA：Tempus
- Simulation：Xcelium
- Formal Verification：JasperGold
- Custom Design：Virtuoso
- PCB 設計：Allegro

### Siemens EDA (前 Mentor Graphics)
- Synthesis：Precision Synthesis
- Simulation：ModelSim、Questa
- DFT：Tessent
- Formal Verification：Questa Formal
- Calibre：DRC/LVS/LFD 簽核工具

### EDA 工具成本

完整 ASIC 設計工具鏈的授權費用通常每年 $100K-$1M+，對小公司和新創公司構成重大進入障礙。這也是開放原始碼 EDA 工具（如 OpenLANE）受到關注的原因。

## 開放原始碼 ASIC

### OpenLANE / OpenROAD

OpenLANE 是由 eFabless 開發的開放原始碼 ASIC 設計流程，基於 OpenROAD 專案。它提供從 RTL 到 GDSII 的自動化流程：

- Synthesis：Yosys（開放原始碼合成工具）
- STA：OpenSTA (OpenTimer)
- PNR：OpenROAD（包含 RePlAce、TritonCTS、FastRoute）
- DRC/LVS：Magic、Netgen、KLayout
- 支援 SkyWater 130nm PDK

### SkyWater 130nm

SkyWater Technology 提供的 130nm 商用製程 PDK (Process Design Kit) 在 2020 年完全開放原始碼，是第一套開放原始碼的商用 PDK。包含標準細胞庫、I/O 庫、SRAM 編譯器、類比元件等。

### Caravel 晶片框架

Caravel 是 eFabless 提供的開放式晶片框架，包含管理晶片（caravel management SoC）和使用者專案區（user project area）。使用者可以在專案區嵌入自己的設計，透過 Caravel 的 GPIO、SPI、UART 等介面與外部通訊。

### Efabless chipIgnite

chipIgnite 是 Efabless 提供的低成本 ASIC 製造計畫，每次 Tapeout 費用約 $10,000，使用 SkyWater 130nm 製程。包含 MPW 生產、封裝和測試，讓個人和小團隊也能製造自己的 ASIC。

### Google 免費 ASIC 計畫

Google 與 Efabless、SkyWater 合作，提供免費的 ASIC 製造機會（Open MPW Shuttle Program），每次選擇約 40-50 個開源專案免費生產晶片。

## ASIC 設計工作量

相較於 FPGA 設計，ASIC 設計需要投入更多的時間和資源：

### 驗證 (Verification)
- ASIC 需要 UVM 驗證環境、formal verification、coverage closure
- FPGA 設計通常只需基本模擬和板上測試
- ASIC 驗證約佔總設計時間的 50-70%

### 實體設計 (Physical Design)
- ASIC 需要 floorplanning、CTS、PDN 設計、IR drop 分析
- FPGA 設計由工具自動完成佈局繞線
- ASIC 團隊需要專精的 CAD/實體設計工程師

### 基礎設施
- ASIC 需要版本控制、資料庫管理、運算叢集
- 需要與晶圓廠的 PDK 和工具相容的 IT 環境
- 通常需要 $100K+ 的 EDA 工具授權和伺服器硬體

### 時間成本
- 簡單 ASIC（<1M 閘級）：12-18 個月
- 中等 ASIC（1M-100M 閘級）：18-24 個月
- 複雜 ASIC（>100M 閘級）：24-36 個月
- FPGA 專案通常只需以上時間的 1/3 到 1/2

## ASIC 與 FPGA 在 eda4 專案中的關聯

eda4 專案主要開發的是純 Rust 實作的 FPGA 工具鏈（verilog2fpga），包含合成 (v2f-synth)、佈局繞線 (v2f-pnr) 和位元流封裝 (v2f-bitstream)。然而，了解 ASIC 對於完整理解 FPGA 技術有以下助益：

1. **FPGA 原型驗證 (FPGA Prototyping)**：在 ASIC 設計流程中，FPGA 常被用作 RTL 驗證的原型平台。在量產前，設計團隊會將 RTL 燒錄到多顆 FPGA 上進行硬體加速驗證，速度快於軟體模擬數百至數千倍。

2. **共同概念**：合成、時序分析、邏輯最佳化等技術在 ASIC 和 FPGA 設計中共通。v2f-synth 中的技術映射 (technology mapping) 概念源自 ASIC 的標準細胞映射。

3. **PNR 演算法**：ASIC 和 FPGA 的 PNR 共享類似的模擬退火 (simulated annealing) 和時序驅動繞線演算法。v2f-pnr 中的演算法可視為 ASIC PNR 演算法的簡化版本。

4. **合成技術**：從 Verilog 解析、AST 建立、邏輯最佳化到網表輸出的流程在兩者間相似。EDA 原理不因目標平台而改變。

5. **設計約束**：SDC 格式的時序約束、多時域設計、非同步處理等概念在兩者間通用。

## 現代趨勢

### 小晶片 (Chiplets)

將大型 SoC 分割為多個較小的晶片（chiplet），透過先進封裝技術（如 2.5D/3D 封裝、CoWoS、InFO）連接。優點包括：
- 更高的良率（小晶片良率高於大晶片）
- 混合製程節點（I/O 用成熟製程，核心用先進製程）
- 設計可重用性（不同產品可共享相同 chiplet）
- 典型範例：AMD Ryzen/EPYC（CCD + IOD）、Apple M1 Ultra（UltraFusion）

### 領域特定加速器 (Domain-Specific Accelerators)

隨著摩爾定律放緩和 Dennard scaling 終結，通用處理器效能提升趨緩。領域特定 ASIC 成為更高效能的替代方案：
- Google TPU：專為 TensorFlow 最佳化的矩陣運算加速器
- NPU (Neural Processing Unit)：手機 SoC 中的 AI 推理引擎
- DPU (Data Processing Unit)：資料中心網路和儲存加速
- 比特幣 ASIC：SHA-256 雜湊運算專用晶片

### 開放原始碼 ASIC 生態系

- PDK：SkyWater 130nm、IHP 130nm BiCMOS、GlobalFoundries 180nm
- EDA：Yosys、NextPNR、OpenROAD、KLayout、Magic
- 設計平台：Efabless ChipIgnite、Google Shuttle、Zero-to-ASIC 課程
- RISC-V 生態系：PULP、CORE-V、VexRiscv、SERV

### AI 驅動的 EDA

機器學習正在改變 EDA 工具：
- 基於 ML 的佈局最佳化（如 Google's RL-based floorplanning）
- 時序預測和功耗預測
- 自動化設計空間探索
- 熱點檢測和 DRC 修復

## 關鍵指標

### 閘級數 (Gate Count)
- 衡量晶片邏輯規模的基本單位
- 現代 ASIC 從數萬閘（微控制器）到數十億閘（GPU/TPU）
- 通常以等效二輸入 NAND 閘數計算

### 時脈頻率 (Frequency)
- 決定數位電路的運算速度
- 取決於製程節點、邏輯深度和電源電壓
- 先進製程（5nm）可達 5-10 GHz（但通常因功耗限制在 2-4 GHz）

### 功耗 (Power)
- 動態功耗：P = αCV²f（活動因子 x 負載電容 x 電壓平方 x 頻率）
- 靜態功耗：漏電流引起的功耗，在先進製程中佔比增加
- 功耗牆 (power wall) 是現代設計的主要限制因素

### 良率 (Yield)
- 良率 = (良品晶粒數) / (總晶粒數)
- 受缺陷密度、晶片面積、製程成熟度影響
- 良率模型：Murphy 模型、Poisson 模型
- 大晶片面積直接降低良率，因此 chiplet 策略具吸引力

### 晶片面積 (Die Area)
- 以 mm² 為單位，直接影響成本和良率
- 面積利用率：標準細胞約 70-90%，全客製約 90%+
- 成本約與面積成反比（相同製程條件下）

## 延伸閱讀

- [ASIC - Application-Specific Integrated Circuit (Wikipedia)](https://en.wikipedia.org/wiki/Application-specific_integrated_circuit)
- [Integrated Circuit (Wikipedia)](https://en.wikipedia.org/wiki/Integrated_circuit)
- [Full Custom IC Design (Wikipedia)](https://en.wikipedia.org/wiki/Full_custom)
- [Standard Cell (Wikipedia)](https://en.wikipedia.org/wiki/Standard_cell)
- [Tape-out (Wikipedia)](https://en.wikipedia.org/wiki/Tape-out)
