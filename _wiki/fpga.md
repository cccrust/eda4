# FPGA (Field-Programmable Gate Array)

## 什麼是 FPGA

FPGA（現場可編程閘陣列）是一種半導體元件，其核心特點在於出廠後可透過軟體重新配置硬體邏輯功能。與 CPU 或 GPU 不同，FPGA 並非執行指令序列，而是直接將電路「燒錄」在晶片上，形成專用的資料路徑。這種可重配置性讓 FPGA 在 ASIC 量產前扮演原型驗證角色，也在需要低延遲、高吞吐量的應用中成為關鍵元件。

一顆 FPGA 晶片內部包含大量可編程的邏輯區塊（Logic Blocks）與可編程的互連線路（Routing Fabric）。設計者使用硬體描述語言（Verilog、VHDL）描述電路，經過合成（Synthesis）、佈局（Place）與繞線（Route）後產生位元流（Bitstream），再載入 FPGA 完成配置。整個流程可在數分鐘到數小時內完成，遠比 ASIC 的數月到數年製造週期快速。

FPGA 的「可現場編程」特性使其在通訊、航太、國防、金融交易、機器視覺、AI 推論等領域廣泛應用。例如，5G 基地台的基頻處理、高頻交易系統的封包解析、衛星上的即時影像處理，都能看到 FPGA 的身影。

## 歷史：從 PAL 到現代 FPGA

可編程邏輯元件的歷史可追溯到 1970 年代。1978 年，Monolithic Memories 推出了 PAL（Programmable Array Logic），這是一種可一次編程的 AND-OR 陣列，用於取代小型 TTL 邏輯。隨後 AMD 與 Lattice 等公司推出 GAL（Generic Array Logic），可重複抹除編程，成為 PAL 的進化版本。

1990 年代前期，CPLD（Complex Programmable Logic Device）出現，將多個 PAL/GAL 區塊透過全域互連整合在單一晶片中。CPLD 採用 EEPROM/Flash 架構，特性為非揮發性、延遲可預測，適合控制邏輯與膠合邏輯（Glue Logic）應用。

1985 年，Xilinx 推出了 XC2064，被認為是全球第一顆 FPGA。與 CPLD 不同，XC2064 採用 SRAM 為基礎（SRAM-based）的 Look-Up Table（LUT）架構，邏輯區塊以陣列排列並由 SRAM 單元控制互連。這項創新開啟了 FPGA 產業，也奠定了 Xilinx 在市場的領導地位。

1990 年代後期到 2000 年代，FPGA 容量與效能快速成長。Xilinx Virtex 系列與 Altera Stratix 系列加入嵌入式 Block RAM（BRAM）、DSP 乘法器、PLL 時脈管理單元、高速序列收發器（SerDes），從單純的膠合邏輯元件轉變為完整的系統單晶片（SoC）平台。2010 年代，Xilinx 推出 Zynq 系列整合 ARM 處理器核心，進一步模糊了 FPGA 與處理器的界線。

## 架構

### 邏輯區塊（Logic Blocks）

現代 FPGA 的核心邏輯單元為 **Slice**（Xilinx）或 **Logic Element**（LE, Altera/Intel），每個單元包含一個或多個 LUT（Look-Up Table，查找表）與正反器（Flip-Flop, FF）。LUT 本質上是一個小型的 SRAM，以 K 個輸入位址對應 2^K 種輸出組合，可實現任何 K 輸入布林函數。常見的 LUT 大小為 4 輸入（LUT4）到 6 輸入（LUT6）。

正反器緊接在 LUT 之後，可配置為 D 型正反器或鎖存器，用於實現管線化與狀態機。進階的 Slice 還包含快速進位鏈（Carry Chain）用於算術運算、多工器、以及專用暫存器鏈。

### 互連線路（Routing Fabric）

邏輯區塊之間由可編程的互連網路連接，包括走線通道（Routing Tracks）、開關盒（Switch Box）、連接盒（Connection Box）。互連佔據晶片面積的 50% 以上，也是決定 FPGA 佈局繞線難度的主要因素。

現代的 FPGA 採用分層式互連結構：短線（Local Routing）連接相鄰邏輯區塊，長線（Global Routing）橫跨整個晶片。開關點通常由 SRAM 單元或 Flash 單元控制傳輸閘（Pass Transistor）或多工器，決定信號路徑。

### I/O 區塊（I/O Blocks）

I/O 區塊連接晶片內部邏輯與外部接腳。支援多種電壓標準（LVCMOS、LVDS、SSTL、HSTL），每根接腳可獨立配置為輸入、輸出或雙向。高速序列收發器（SerDes）的收發速率已達 58 Gbps（如 Xilinx Virtex UltraScale+）。

### 嵌入式功能區塊

- **Block RAM（BRAM）**：專用的同步記憶體區塊，可配置為單埠、雙埠、FIFO 或 ROM。容量通常為 18-36 Kbits 每區塊，總計數十 Mbits。
- **DSP Slice**：專用乘法器與累加器，用於數位訊號處理（濾波、FFT、矩陣運算）。Xilinx 的 DSP48 可執行 27x18 乘法與 48-bit 累加。
- **PLL / DLL**：鎖相迴路與延遲鎖定迴路，負責時脈產生、抖動濾除、相位調整。
- **高速 SerDes**：用於 PCIe、Ethernet、JESD204B 等高速通訊協定。
- **嵌入式處理器**：硬核心（ARM Cortex-A 系列，如 Zynq）或軟核心（MicroBlaze、Nios II）。

## 配置技術

### SRAM-based（靜態隨機存取記憶體型）

SRAM-based FPGA 使用 SRAM 單元儲存連線配置與 LUT 內容。這是最常見的技術（Xilinx/AMD、Intel/Altera 全系列）。

- 優點：使用標準 CMOS 製程，邏輯密度高，可無限次重寫，配置速度快。
- 缺點：揮發性（Volatile），斷電後配置遺失，每次上電需從外部 Flash 載入位元流。
- 安全性：位元流易被竊取，需加密或使用內建 AES 解密。

### Flash-based（快閃記憶體型）

Flash-based FPGA 將配置儲存在晶片內部的 Flash 單元中（Lattice iCE40、MachXO2/3、Microchip PolarFire）。

- 優點：非揮發性，斷電後配置保留，上電即用（Instant On），不需外部配置記憶體。
- 缺點：Flash 單元需要額外製程步驟，邏輯密度與 SRAM-based 相比略低；重寫次數有限（約 10,000 次），但對正常應用已足夠。
- 安全性：Flash 配置較難逆向，安全性優於 SRAM-based。

### Antifuse（反熔絲型）

Antifuse FPGA 是單次可編程（OTP, One-Time Programmable）。編程時對反熔絲節點施加高電壓，使其從高阻抗變為永久導通。

- 優點：極高的可靠性與抗輻射能力，適合航太與國防應用。
- 缺點：無法重寫，適合量產後固定設計的場合。
- 代表：Microchip/Microsemi 的 RTAX 系列。

### 比較

| 特性 | SRAM-based | Flash-based | Antifuse |
|------|-----------|-------------|----------|
| 揮發性 | 是 | 否 | 否 |
| 可重寫 | 無限次 | 約 10,000 次 | 單次 |
| 上電立即 | 否 | 是 | 是 |
| 外部記憶體 | 需要 | 不需要 | 不需要 |
| 邏輯密度 | 最高 | 中等 | 中等 |
| 輻射耐受 | 低 | 中等 | 高 |
| 成本 (量產) | 低 (晶片) + Flash | 中等 | 中高 |
| 代表廠商 | Xilinx, Intel | Lattice, Microchip | Microchip |

## 應用場景

### ASIC 原型驗證

ASIC 在投片（Tape-out）前，先在 FPGA 上驗證功能與效能。單顆高階 FPGA 可實現數百萬閘的設計；多顆 FPGA 組成的原型驗證平台（如 Synopsys HAPS、ProDesign S4）可模擬完整的 SoC，大幅降低晶片改版風險。

### 低量產與快速部署

對於年出貨量在數千到數萬顆的應用（航太、軍工、醫療儀器），ASIC 的非 recurring engineering（NRE）成本過高，FPGA 成為更經濟的選擇。FPGA 的可重配置性也讓現場升級（OTA）成為可能。

### 網路與 AI 加速

FPGA 在資料中心用於智慧網卡（SmartNIC）、NVMe 加速、壓縮/解壓縮、加密。微軟 Azure 採用 Altera FPGA 加速 Bing 搜尋，百度使用 Xilinx FPGA 加速語音辨識。近年來，AI 推論加速成為 FPGA 的重要市場：INT8/INT4 量化運算、稀疏矩陣處理、客製化資料路徑，FPGA 的彈性與能效比在中等批次下優於 GPU。

### 低延遲金融交易

高頻交易（HFT）對延遲極度敏感。FPGA 可實現從網路封包解析到交易決策的純硬體管線，端到端延遲低至數十奈秒，遠優於 CPU 的微秒級。NASDAQ 與其他交易所已採用 FPGA 處理訂單簿與風險檢查。

### 航太與國防

FPGA 在衛星、雷達、電子戰系統中處理高頻寬感測資料，抗輻射 FPGA（耐輻射強化型）用於太空環境。Flash-based 與 Antifuse FPGA 因其非揮發性與可靠性在此領域佔有優勢。

## FPGA vs ASIC vs CPU vs GPU

| 特性 | FPGA | ASIC | CPU | GPU |
|------|------|------|-----|-----|
| 硬體靈活性 | 可重配置 | 固定 | 軟體可程式 | 軟體可程式 |
| 開發時間 | 數週-數月 | 數月-數年 | 數週-數月 | 數週-數月 |
| NRE 成本 | 低 | 數百萬-數千萬美元 | 低 | 低 |
| 單位成本 (大量) | 中等 | 最低 | 視架構 | 視架構 |
| 效能 (專用任務) | 高 | 最高 | 低 | 中高 |
| 功耗效率 | 高 | 最高 | 低 | 中等 |
| 延遲 | 極低 (ns) | 極低 (ns) | 高 (us) | 中高 (us) |
| 並行度 | 高 (資料流) | 高 (資料流) | 低 (指令流) | 極高 (SIMT) |

FPGA 的獨特價值在於「硬體效能 + 軟體靈活性」的平衡點。對於需要高吞吐量、低延遲，但需求量不足以攤提 ASIC NRE 成本的應用，FPGA 是最佳選擇。

## FPGA 供應商

### Xilinx / AMD

Xilinx 由 Ross Freeman 與 Bernard Vonderschmitt 於 1984 年創立，是 FPGA 產業的開創者。產品線從低成本的 Spartan 系列到高效能的 Virtex 系列，再到整合 ARM 的 Zynq 與 Versal ACAP。2022 年 AMD 以 498 億美元收購 Xilinx，現以 AMD Adaptive Computing 事業群營運。

### Intel / Altera

Altera 曾是 Xilinx 的主要競爭對手，產品包括 Cyclone（低功耗）、Arria（中階）、Stratix（高效能）。2015 年 Intel 以 167 億美元收購 Altera，將 FPGA 整合至 Xeon 處理器封裝（Xeon+FPGA）。目前產品線以 Agilex 系列為主，採用 Intel 7 製程。

### Lattice Semiconductor

Lattice 專注於低功耗、小尺寸的 FPGA，避開與 Xilinx/Intel 在高階市場的直接競爭。主要產品線：

- **iCE40 Ultra/UltraLite/HX/LP**：超低功耗 FPGA，封裝小至 2.5x2.5 mm，待機功耗低至 100 uA。廣泛用於消費性電子、物聯網、行動裝置。
- **MachXO2/3/5**：非揮發性 FPGA，專為膠合邏輯、控制匯流排、系統管理設計。
- **ECP5**：中階 FPGA，支援 SERDES 與 DDR3 記憶體介面，是 open source 工具鏈（PRJC project）的重點目標。
- **CrossLink / Nexus**：用於 MIPI 橋接與視訊處理。

### Microchip / Microsemi

Microsemi（現為 Microchip 子公司）是抗輻射 FPGA 的主要供應商。PolarFire 系列採用 Flash-based 技術，強調低功耗與高安全性，適合航太、國防、邊緣運算。

### 中國廠商

隨著美中科技脫鉤，中國 FPGA 廠商快速崛起。**Gowin（高雲半導體）** 提供小尺寸低功耗 FPGA，與 Lattice iCE40 直接競爭。**Efinix（益福科技）** 以 Quantum 架構（可程式單元可同時作為邏輯與繞線使用）差異化切入市場。**AGM（蘇州國芯）** 與 **Anlogic（安路科技）** 也有完整的產品線。

## iCE40 系列

### 產品定位

Lattice iCE40 系列是針對超低功耗與小型封裝設計的 FPGA 產品線。典型應用包括感測器集線器（Sensor Hub）、LED 控制、觸控辨識、開機安全驗證、消費性電子中的膠合邏輯替代。

相較於 Xilinx Spartan 或 Intel Cyclone，iCE40 的邏輯容量較小（128 到 7,680 個 LUT）、無內建 SerDes 或 DDR PHY，但功耗極低且封裝極小。其待機功耗可低至 100 uA，適合電池供電裝置。

### 支援的變體

eda4 專案支援 iCE40 五個主要變體：

| 變體 | LUT 數量 | BRAM | PLL | 封裝 |
|------|----------|------|-----|------|
| iCE40-HX1K | 1,280 | 64 Kbit | 1 | 100-pin VQFN / 256-ball caBGA |
| iCE40-HX4K | 3,520 | 80 Kbit | 2 | 同上 |
| iCE40-HX8K | 7,680 | 128 Kbit | 2 | 同上 |
| iCE40-LP1K | 1,280 | 64 Kbit | 1 | 同 HX 系列但功耗更低 |
| iCE40-UP5K | 5,280 | 120 Kbit | 2 | 32-pin QFN / 36-ball WLCSP (2.5x2.5 mm) |

HX（High Performance）系列強調更高的效能與更多 I/O；LP（Low Power）系列最佳化靜態功耗；UP（Ultra Plus）則是最小封裝的選擇。

## 開源 FPGA 工具鏈

### IceStorm

IceStorm 是由 Claire Wolf 發起的 iCE40 逆向工程專案，目標是建立完整的開放原始碼 FPGA 工具鏈。IceStorm 包含：

- **icepack / iceunpack**：位元流打包與解包工具。
- **icebox**：iCE40 資料庫與 CRAM 位址映射。
- **icebram**：BRAM 初始化工具。
- **icetime**：靜態時序分析。

IceStorm 的逆向工程為後續的 Yosys + nextpnr 提供了基礎。

### Yosys

Yosys 是 Claire Wolf 開發的開源 Verilog 合成工具。它從 Verilog RTL 開始，經過一連串 passes（編譯、最佳化、技術映射）輸出 netlist。對於 iCE40 目標，Yosys 使用 `synth_ice40` 指令將電路映射至 iCE40 的邏輯單元（SB_LUT、SB_DFF）。

Yosys 支援完整的 Synthesizable Verilog 子集，並可輸出 BLIF、EDIF、JSON 等格式供下游工具使用。

### nextpnr

nextpnr 是與 Yosys 搭配的開源佈局繞線工具。它支援 iCE40（`--hx1k`、`--hx4k`、`--hx8k`、`--lp1k`、`--up5k`）、ECP5、以及部分 Nexus 系列。nextpnr 採用模擬退火（Simulated Annealing）演算法進行佈局，再以 A*-based 繞線器完成繞線。

nextpnr 可輸出 ASCII 格式的 PnR 結果（.ASC），再由 IceStorm 工具的 icepack 轉換為最終的 .bin 位元流。

### 生態系與專案

- **SymbiFlow** / **F4PGA**：旨在建立 FPGA 架構無關的開源工具鏈，支援 Xilinx 7-Series（透過 Project X-Ray）、Lattice ECP5 等。
- **PRJC Project**：延伸 Yosys + nextpnr 支援 ECP5。
- **Apio**：高層級的開源 FPGA 開發工具，封裝了 Yosys、nextpnr、IceStorm 的安裝與使用流程。
- **Amaranth / nMigen**：Python-based 的 HDL 框架，可產生 Verilog 再由 Yosys 合成。

## iCE40 與 eda4 專案

### 定位

eda4 是使用 Rust 語言開發的純開源 EDA 工具鏈，專注於 Lattice iCE40 FPGA。不同於依賴 Yosys 或 nextpnr 的既有工具鏈，eda4 從合成、佈局繞線到位元流打包皆以 Rust 實作，提供完整的端到端（End-to-End）純 Rust 解決方案。

### 子專案架構

eda4 採用 monorepo 結構，包含四條核心管線：

- **verilog2fpga**：Verilog 合成器，解析 Verilog 程式碼產生內部 netlist。合成結果為 JSON 格式的邏輯網路，可直接傳遞給 PnR 階段。
- **v2f-pnr**：純 Rust 實作的模擬退火佈局繞線器，接受 JSON netlist 與晶片描述（來自 v2f-db），輸出 ASCII 格式的 .asc 檔案。
- **v2f-bitstream**：位元流打包器，將 .asc 轉換為 iCE40 的 .bin 位元流，可直接載入至 iCE40 晶片。
- **v2f-programmer**：透過 SPI/JTAG 介面將 .bin 燒錄至 iCE40 的程式設計器。支援開源的 openFPGALoader 以及 FTDI-based 燒錄。

此外，eda4 也包含 v2f-rust（提供 `fpga!` 巨集，以 Rust 語法描述硬體）與 v2f-viz（視覺化工具，用於檢視合成與 PnR 結果）。

### 優勢

- **純 Rust 實作**：無需安裝 Yosys、nextpnr、IceStorm 等外部工具。開發者可用 `cargo build` 完成完整的合成、佈局、繞線、打包流程。
- **除錯便利**：所有階段皆以 Rust 原生型別與結構表示，可在同一程序中進行交叉驗證與除錯。
- **驗證導向**：配合 verilog2rust 可將 Verilog 轉換為 Rust 模擬模型，進行邏輯驗證。
- **現代軟體工程**：採用 Rust 的型別系統與所有權模型，減少 C/C++ 工具鏈中常見的記憶體安全問題。

### 貢獻

eda4 專案歡迎貢獻者參與合成器最佳化、新增 iCE40 變體支援、PnR 演算法改進、以及測試案例擴充。開發方向包括全形式化驗證支援、更詳盡的時序分析、以及對 Lattice ECP5 系列的工具鏈擴展。
