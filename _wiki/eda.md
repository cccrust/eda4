# EDA (Electronic Design Automation) 電子設計自動化

## 什麼是 EDA

EDA（Electronic Design Automation，電子設計自動化）是指利用電腦軟體來設計、模擬、驗證與製造電子系統的一系列工具與方法論。廣義而言，EDA 涵蓋從晶片規格定義到最終製造光罩的全流程；狹義上則常指積體電路（IC）與印刷電路板（PCB）的輔助設計工具。

EDA 的核心命題是：**用軟體來設計硬體**。工程師透過高階描述語言（如 Verilog、VHDL、SystemVerilog）描述電路行為，再由 EDA 工具自動化地將其轉換為實體佈局與製造檔。這個抽象層層堆疊的過程，與編譯器將高階語言轉換為機器碼的概念相似，但硬體設計的約束（面積、功耗、時序、良率）遠比軟體編譯複雜。

EDA 是半導體產業的關鍵支柱。沒有 EDA，就無法在合理的成本與時間內設計出現代數十億電晶體的晶片。據估計，一款先進製程 SoC 的設計成本中，EDA 工具授權與軟體工程費用可佔總開發預算的 20% 到 30%。EDA 工具的正確性直接影響晶片能否一次投片成功（first-silicon success），這在 5nm、3nm 製程中每次試產動輒數千萬美元的背景下極為重要。

EDA 與半導體製程（foundry）以及矽智財（IP）共同構成了現代積體電路的三大支柱。晶圓廠提供製程設計套件（PDK），EDA 工具根據 PDK 的規則進行設計與驗證，而 IP 供應商則提供預先設計好的功能區塊（如 USB、PCIe、CPU 核心）。三者分工協作，使得設計團隊可以專注於晶片的架構創新，而不必從電晶體層級從頭打造。

## 歷史：從 CAD 到現代 EDA

### 1960s--1970s：萌芽期與 CAD

EDA 的起源可追溯至 1960 年代的電腦輔助設計（CAD）。當時積體電路的複雜度還不高，工程師仍大量依靠手工繪製佈局（layout），用膠帶將不同層的光罩貼在一起（所謂的「膠帶輸出」）。最早的 EDA 工具是電路模擬器——1971 年 Berkeley SPICE（Simulation Program with Integrated Circuit Emphasis）的誕生標誌著電腦輔助分析時代的開始。SPICE 至今仍是業界標準的電路模擬引擎。

1970 年代中期，隨著 MOS 技術的成熟，晶片整合度快速提升，手工佈局逐漸無法應付。加州理工學院與 IBM 分別發展了早期的自動佈線（auto-routing）與佈局（placement）演算法。Carver Mead 與 Lynn Conway 在 1980 年出版的 _Introduction to VLSI Systems_ 提出了結構化設計方法論，主張用抽象化的設計規則取代繁瑣的手工繪製，這為後來的邏輯合成與高層次綜合奠定了理論基礎。

### 1980s：邏輯合成與商用 EDA 的崛起

1980 年代是 EDA 商業化的黃金時期。1983 年，GE 的工程師開發了第一套邏輯合成工具 SOCRATES，能將布林等式自動轉換為閘級網表。1986 年，Synopsys 成立並推出了 Design Compiler，這是有史以來第一款商業邏輯合成工具，徹底改變了數位設計流程——工程師不再需要手動推導閘級電路，而是用硬體描述語言（HDL）撰寫 RTL 程式碼，由工具自動合成。

同一時期，硬體描述語言本身也在標準化。1984 年 DoD 推出了 VHDL，1985 年 Gateway Design Automation 推出了 Verilog。1990 年 Verilog 成為公開標準（由 Open Verilog International 管理），VHDL 則在 1987 年成為 IEEE 1076 標準。這兩種語言至今仍是數位設計的主流。1980 年代末，Cadence Design Systems 透過一系列併購成為了實體設計（佈局與佈線）領域的主導者。

### 1990s：驗證與 DFT 時代

1990 年代，晶片設計的瓶頸從「能不能設計出來」轉向「能不能確保正確」。模擬驗證（simulation）與形式驗證（formal verification）工具快速發展。1992 年，基於 RTL 的邏輯模擬器成為標準流程。1996 年，形式驗證工具開始商業化，用數學方法證明 RTL 與閘級網表的等價性（equivalence checking）。

可測試性設計（DFT）也在這個時代成熟。掃描鍊插入（scan chain）、內建自我測試（BIST）與邊界掃描（JTAG/IEEE 1149.1）成為必須嵌入晶片的結構。Mentor Graphics（現為西門子 EDA）的 DFT 工具如 FastScan 與 FlexTest 主導了這個領域。

1990 年代末還見證了靜態時序分析（STA）的普及。PrimeTime（Synopsys）成為了黃金標準，任何晶片在 tape-out 前都必須通過 STA 的時序簽核（timing signoff）。

### 2000s--2010s：物理綜合與多核心

2000 年代，製程微縮進入深亞微米（deep submicron），佈線延遲超越了閘延遲成為主導。這催生了物理綜合（physical synthesis）——將邏輯合成與佈局資訊整合在一起，讓工具在綜合時就能預估走線延遲。Synopsys IC Compiler 與 Cadence Encounter（後來的 Innovus）是這個時期的代表產品。

多核心處理器的普及也改變了 EDA 工具的架構。靜態時序分析與模擬引擎大量採用平行計算。同時，功耗分析（power analysis）與低功耗設計技術（多電壓域、時脈閘控、電源關斷）成為必備功能，因為攜帶式裝置與資料中心的功耗預算越來越嚴苛。

### 設計與工具的陰陽關係

回顧 EDA 歷史，可以觀察到一種「陰陽循環」：新製程技術的出現產生設計瓶頸，推動 EDA 工具創新；創新後的工具反過來解鎖更複雜的設計，進一步推動製程前進。Mead 與 Conway 的抽象化方法解放了 1980 年代的 VLSI 設計，卻也讓工具複雜度急速攀升；邏輯合成降低了設計人力成本，但驗證與分析工具的需求隨之爆發。每一個世代的 EDA 突破都是在解決前一代設計方法論所創造的複雜度問題。

這個陰陽關係在 2020 年代依然持續。AI/ML 輔助設計、3D-IC、RISC-V 開源生態正在催生新一輪的 EDA 工具革命，而這些工具又將進一步降低晶片設計的門檻。

## EDA 流程

一個典型的數位 IC 設計流程包含以下階段，每個階段都有對應的 EDA 工具：

### RTL 設計

設計工程師使用硬體描述語言（Verilog、VHDL、SystemVerilog）撰寫暫存器傳輸層級（Register-Transfer Level, RTL）的電路描述。RTL 描述的是資料如何在暫存器之間流動，以及在每個時脈週期中進行的運算。

RTL 設計通常伴隨著程式碼檢查（linting）與語法驗證。工具如 SpyGlass（Synopsys）或 Verilator 的 lint 模式可以在早期發現綜合陷阱與時序問題。

### 模擬與驗證

RTL 撰寫完成後，需要使用測試平台（testbench）進行功能模擬（functional simulation），以確認設計符合規格。模擬器分為事件驅動（event-driven）與時脈週期精確（cycle-accurate）兩類。商業工具如 VCS（Synopsys）、Xcelium（Cadence）、Questa（Siemens），開源工具如 Verilator、Icarus Verilog、GHDL。

驗證工程師還會使用 UVM（Universal Verification Methodology）來建構可重複使用的驗證環境，並使用覆蓋率導向的隨機驗證技術來發現邊界案例。形式驗證（formal verification）則用數學證明的方式檢查斷言（assertion）是否在所有可能狀態下都成立。

### 邏輯合成

邏輯合成將 RTL 程式碼轉換為閘級網表（gate-level netlist），並選擇標準單元（standard cells）來實現邏輯功能。合成工具需要考慮面積、速度與功耗的取捨。輸入是 RTL 與製程庫（liberty file），輸出是閘級網表與設計約束文件（SDC）。

合成過程包含三個步驟：轉譯（translation）將 RTL 轉為未優化的布林邏輯；邏輯最佳化（logic optimization）化簡與重構電路；技術映射（technology mapping）將邏輯映射到製程庫中的實際單元。

### 實體設計（佈局與佈線）

Place & Route（P&R）將閘級網表轉換為晶片的實體幾何佈局。佈局（placement）決定每個標準單元的位置，目標是最小化總走線長度且不違反面積約束。佈線（routing）則在單元之間建立實際的金屬連線，必須滿足設計規則檢查（DRC）的要求。

現代 P&R 工具採用增量式最佳化——在佈局、時脈樹合成（clock tree synthesis）、佈線與時序最佳化之間反覆迭代。商業工具如 Innovus（Cadence）與 IC Compiler II（Synopsys）是這個領域的霸主。

### 靜態時序分析

STA 獨立驗證電路是否能在指定的時脈頻率下正確運作。它不需要輸入向量，而是計算每一條路徑的延遲，並檢查是否滿足建立時間（setup）與保持時間（hold）的要求。STA 是晶片簽核的最後關卡之一。

### 實體驗證與簽核

包含設計規則檢查（DRC）、電路圖比對（LVS）、天線效應檢查、IR drop 分析、電遷移（electromigration）檢查。通過這些檢查後，設計才能進行光罩製造。這個階段的工具如 Calibre（Siemens）與 ICV（Synopsys）。

### 位元流生成與燒錄

對於 FPGA 設計，P&R 之後需要產生位元流（bitstream）檔案，這個檔案包含了 FPGA 內部可程式邏輯區塊與互連的配置資料。將位元流燒錄到 FPGA 晶片中，就完成了從抽象 HDL 到實體電路的轉換。

## EDA 主要領域

### 邏輯合成

邏輯合成是將高階硬體描述轉換為最佳化閘級網表的核心技術。研究範圍包括布林代數化簡、多層邏輯最佳化、時序驅動的合成、低功耗合成。代表學者有 Robert Brayton（UC Berkeley）與其開發的 SIS/ABC 邏輯合成系統。

### 實體設計

實體設計處理晶片的幾何佈局與互連。經典問題包括：標準單元佈局（如何安排單元位置以最小化面積與走線）、全域佈線與細部佈線（如何分配走線通道並解決衝突）、時脈樹合成（如何分佈時脈訊號以最小化偏移）。這個領域大量使用組合最佳化與圖論演算法。

### 功能驗證

驗證佔據現代晶片設計 50%--70% 的工程人力。動態模擬、覆蓋率驅動的隨機驗證、形式驗證（模型檢查、等價性檢查）、硬體加速驗證（emulation）都是此領域的核心技術。斷言式驗證（SVA, SystemVerilog Assertions）是連接設計與驗證的重要橋樑。

### 可測試性設計 (DFT)

DFT 確保晶片在製造後可以被有效地測試。掃描鍊設計、ATPG（自動測試向量生成）、BIST、邊界掃描都是 DFT 工程師的工作。測試成本與測試品質（缺陷覆蓋率）之間的權衡是 DFT 的核心課題。

### 自動測試設備 (ATE)

ATE 是用來測試已封裝晶片的硬體設備。測試工程師使用 ATPG 產生的測試向量，在 ATE 上對每一顆晶片施加激勵並量測輸出。ATE 的通道數、時脈速度與精度決定了晶片的測試產能。

## 開源 EDA 與商業 EDA 的對比

### 商業 EDA 三巨頭

全球 EDA 市場由三家公司主導：Synopsys、Cadence Design Systems、以及 Siemens EDA（前 Mentor Graphics）。三家公司合計佔據約 75% 的市場份額。

Synopsys 是邏輯合成與靜態時序分析的龍頭，Design Compiler 與 PrimeTime 分別是這兩個領域的黃金標準。Cadence 在實體設計（Virtuoso、Innovus）與客製化 IC 設計領域佔有優勢。Siemens EDA 則以 Calibre 實體驗證平台聞名，在製造檢驗（DRC/LVS）領域幾乎無可匹敵。

商業 EDA 的授權費用極為昂貴，一套完整流程的年費可達數十萬到數百萬美元。這使得先進晶片設計的門檻極高，新創公司與中小型團隊難以負擔。

### 開源 EDA 的興起

開源 EDA 在過去十年取得了長足進步。最重要的里程碑是 Claire Wolf 開發的 Yosys——一套用於 Verilog 的開源邏輯合成框架。Yosys 支援從 Verilog 2005 到多種後端（包括 ASIC 標準單元與 FPGA）的合成流程。

在 FPGA 領域，Yosys 搭配 nextpnr（開源 P&R 工具）與 Project IceStorm（Lattice iCE40 位元流文件）形成了一套完整的開源 FPGA 工具鏈。這套工具鏈支援 Lattice iCE40、ECP5 以及部分 Xilinx 7-series 裝置。SymbiFlow 專案（後更名為 F4PGA）致力於將這套開源工具鏈擴展到更多 FPGA 架構。

開源 EDA 的優勢在於：無授權成本、可自由修改與客製化、適合教育與學術研究、支援 CI/CD 自動化。劣勢則包括：對先進製程（7nm 以下）的支援不足、缺乏完整的簽核等效性認證、社群支援的可靠性不如商業供應商。

### 混合生態

當代許多設計團隊採用混合策略：在設計前端使用開源工具（Yosys 進行合成、Verilator 進行模擬），後端的時序簽核與實體驗證則使用商業工具。這種方案在成本與可靠性之間取得平衡。

## Yosys / nextpnr / IceStorm 生態系統

### Yosys

Yosys 由 Claire Wolf（現為 Google 工程師）於 2012 年開始開發，是目前最成熟的開源 Verilog 合成工具。Yosys 的架構基於一組內部表示（RTLIL 與正式化後的正式電路表示），並透過一系列 pass（處理步驟）來進行分析、轉換與最佳化。

Yosys 支援 Verilog 2005（部分支援 SystemVerilog），可以輸出 EDIF、BLIF、Verilog 網表等多種格式。其模組化設計讓使用者可以自行撰寫 pass 來擴充功能。Yosys 也被廣泛應用於學術研究——硬體安全、形式驗證、新興架構探索等領域都有人使用 Yosys 作為實驗平台。

Yosys 的經典應用流程為：`read_verilog` 讀取設計 → `hierarchy` 建立模組層級 → `proc` 處理程序區塊 → `opt` 邏輯最佳化 → `techmap` 技術映射 → `write_verilog` 輸出網表。

### nextpnr

nextpnr 是由 Claire Wolf、David Shah 等人開發的開源 FPGA P&R 工具。與 Yosys 一樣，nextpnr 採用模組化架構，支援多種 FPGA 架構（iCE40、ECP5、Gemini、 Nexus）。nextpnr 使用模擬退火（simulated annealing）演算法進行佈局，並採用基於衝突的佈線演算法。

nextpnr 的架構分為晶片資料庫層、演算法層與架構特定的後端層。這種設計使得支援新的 FPGA 家族只需實作架構描述與對應的後端程式碼，大幅降低了移植成本。

### Project IceStorm

Project IceStorm 是 2015 年啟動的反向工程專案，目標是完整理解 Lattice iCE40 FPGA 系列的位元流格式與晶片架構。當時 Lattice 的官方工具鏈（iCEcube2）是閉源的，僅支援 Linux 與 Windows，且授權限制嚴格。

IceStorm 專案透過逆向工程 iCE40 的 CRAM（配置 RAM）映射，建立了開源的位元流文件與工具鏈——包括 `icepack`（位元流打包）、`iceprog`（燒錄）、以及 Python API。IceStorm 的成功不僅催生了 Yosys 的 iCE40 支援與 nextpnr，更重要的是證明了 FPGA 逆向工程的可行性，並啟發了後續一系列開源 FPGA 專案。

### 生態影響

Yosys + nextpnr + IceStorm 的三層架構形成了一個完整的開源 FPGA 開發流程：Verilog → Yosys 合成（.json）→ nextpnr 佈局佈線（.asc）→ icepack 位元流打包（.bin）→ iceprog 燒錄。這個流程完全開源、跨平台，且不需要任何商業授權。

這條工具鏈的存在直接催生了多個開源硬體專案：PicoRV32（RISC-V CPU）、Serv（位元序列 RISC-V）、以及大量基於 iCE40 的開放式 FPGA 開發板（如 iCEBreaker、TinyFPGA、ULX3S）。它極大地降低了 FPGA 開發的進入門檻，讓個人開發者、教育機構與新創公司也能夠使用 FPGA。

## eda4 的定位

### 專案概覽

eda4 是一個以 Rust 語言為基礎的 EDA 單體倉庫（monorepo），包含兩個主要子專案：verilog2fpga 與 verilog2rust。

**verilog2fpga** 是一套純 Rust 實現的 FPGA 工具鏈，涵蓋從 Verilog 合成到位元流燒錄的完整流程。它包含 v2f-synth（Verilog 合成）、v2f-pnr（佈局佈線）、v2f-bitstream（ASC 轉 BIN 位元流打包）與 v2f-programmer（燒錄器）等核心 crate。verilog2fpga 支援 Lattice iCE40 家族（HX1K、HX4K、HX8K、LP1K、UP5K），並提供整合了 clap 的命令列界面。

**verilog2rust** 則是一條不同的路徑——它將 Verilog 設計轉換為 Rust 程式碼，並利用 Rust 執行時的訊號模擬引擎來運行硬體模擬。verilog2rust 包含完整的 Verilog 詞法分析器、解析器、AST 與程式碼生成器，以及基於事件驅動的模擬執行環境（rhdl 模組）。使用者可以將 Verilog 設計轉譯為 Rust 原始碼，然後用標準的 rustc 進行編譯與執行。

### 處理流程

verilog2fpga 的默認流程為純 Rust 路徑：`.v` 檔案 → v2f-synth（輸出 JSON 網表）→ v2f-pnr（輸出 ASC 佈局）→ v2f-bitstream（輸出 BIN 位元流）→ v2f-programmer（燒錄至 FPGA）。也支援 `--backend yosys` 使用外部 Yosys/nextpnr/icepack，以及 `--backend pnr-only` 僅使用外部合成、內部 P&R。

verilog2rust 的流程為：讀取 `.v` 或 `.rhdl` 檔案 → 詞法分析與語法解析 → 程式碼生成（Rust 原始碼）→ rustc 編譯 → 執行。模擬引擎提供 Signal、Gate trait 等抽象，支援事件驅動的電路模擬。

### 專案理念

eda4 的核心理念包含以下幾點：

第一，**純 Rust 實現**。整個工具鏈不需要依賴 Yosys、nextpnr 或任何外部工具（雖然也支援外部後端）。這意味著只要 Rust 編譯器能跑的平台，eda4 就能跑——包括 macOS、Linux、Windows，以及 Wasm 等非傳統平台。對於一個傳統上綁定 Linux 的領域來說，這是顯著的跨平台優勢。

第二，**模組化架構**。eda4 被劃分為多個獨立 crate（v2f-core、v2f-synth、v2f-pnr、v2f-bitstream、v2f-programmer、v2f-rust 等），各自負責 EDA 流程中的一個環節。這種設計允許開發者獨立使用或替換某個階段，也便於對特定階段進行測試與最佳化。

第三，**教育與實驗友好**。由於完全開源且無需商業授權，eda4 非常適合用於 EDA 領域的教學與研究。學生可以透過閱讀原始碼了解合成演算法、模擬退火佈局策略、位元流打包等技術細節，而無需破解商業工具的封閉介面。

## 為什麼 Rust 適合 EDA

### 記憶體安全

EDA 工具處理的資料結構通常非常龐大——數百萬閘的電路網表、數十億條時序路徑、TB 級的模擬波形。在 C/C++ 的傳統實踐中，緩衝區溢位與釋放後使用等記憶體錯誤是 EDA 工具當機的首要原因。Rust 的所有權系統在編譯期保證了記憶體安全，消除了這一大類錯誤。

### 零成本抽象

EDA 演算法通常需要緊湊的記憶體佈局與高效的資料存取。Rust 的泛型、迭代替器（iterator）與模式匹配在提供高階表達力的同時不需要運行時開銷，這與 EDA 工具對效能的要求高度契合。

### 平行計算

EDA 工作負載本質上具有大量的平行性——獨立的時序路徑可以平行分析，不同的模擬測試向量可以平行執行，多個佈局候選者可以平行評估。Rust 的 Send/Sync trait 系統與 Rayon 等資料平行函式庫讓撰寫安全的平行程式碼變得相對容易，而不會有資料競爭的風險。

### 無垃圾回收

EDA 工具中常見的大規模資料結構（如圖結構的網表、四叉樹空間索引）對垃圾回收器的表現不太友善。Rust 不需要 GC，所有資源的生命週期在編譯期確定，這保證了可預測的延遲與穩定的峰值效能。

### FFI 與生態整合

Rust 可以透過 C FFI 無縫呼叫既有的 C/C++ EDA 函式庫（如 ABC 邏輯合成系統、SPICE 模擬器核心），這讓團隊可以逐步替換而不是一次性重寫整個工具鏈。

## FPGA 工具鏈的民主化

### 封閉時代

在 2015 年之前，FPGA 開發幾乎完全依賴於供應商提供的封閉工具鏈——Xilinx ISE/Vivado、Altera Quartus、Lattice iCEcube2。這些工具不僅是閉源的，而且通常只能運行在特定 Linux 發行版上，命令行界面不統一，自動化整合困難。更關鍵的是，位元流格式被視為商業機密，第三方工具完全無法介入。

這種封閉生態導致了幾個問題。教育機構難以在課堂中深入講解 FPGA 的底層原理，因為位元流格式不透明。新創公司進入 FPGA 加速領域的門檻極高，因為工具鏈的授權費用與綁定效應使得創新成本高昂。硬體安全研究人員也難以分析 FPGA 位元流的安全性。

### 開源革命

Project IceStorm 在 2015 年的突破具有象徵意義——它證明了 FPGA 位元流可以透過逆向工程被公開理解。隨後 Yosys 加入對 iCE40 的支援，nextpnr 的開發填補了開源 P&R 的空白，到 2018 年左右，一條完整的開源 FPGA 工具鏈已經可用。

這個生態的影響遠超出技術層面。開源工具鏈讓 FPGA 開發從「少數供應商壟斷」轉向「社群協作創新」。目前開源 FPGA 工具鏈已支援 Lattice iCE40、ECP5、Nexus 等家族，並且對 Xilinx 7-series 的支援正在透過 Project X-Ray 與 SymbiFlow/F4PGA 持續推進。

### eda4 的貢獻

eda4 在這個開源運動中的角色是提供一個純 Rust 的替代實現。不同於 Yosys 的 C++ 實作與 nextpnr 的混合架構，eda4 從頭到尾使用同一種語言——這降低了新貢獻者參與的認知負擔，也讓跨元件的重構與重組更加直接。

對於 verilog2rust 路線，eda4 提出了一個有趣的觀點：硬體設計不僅可以透過傳統的 EDA 流程轉換為位元流，也可以轉換為一般用途的程式語言（Rust）並在 CPU 上執行模擬。這模糊了硬體與軟體的邊界，呼應了硬體/軟體協同設計（HW/SW co-design）的長期趨勢。

### 民主化的願景

FPGA 工具鏈的民主化最終目標是讓更多人能夠參與數位系統設計。就像開源軟體讓任何人都能成為開發者一樣，開源 EDA 工具鏈讓硬體設計不再被昂貴的專有工具所限制。這對於新興領域——如開放式硬體加速器設計、邊緣 AI 推論晶片、客製化 IoT 處理器——具有重要的推動作用。

## 參考文獻

- Chen, W. K. (Ed.). (2003). _Electronic Design Automation_. Prentice Hall. 這本經典的 EDA 教科書系統性地介紹了從邏輯綜合到實體設計的各個領域，是了解 EDA 理論基礎的重要參考。
- Wolf, W. (2012). _Modern VLSI Design: System-on-Chip_ (4th ed.). Prentice Hall. Wolf 的書從系統層級的角度討論 VLSI 設計方法論，強調設計抽象與工具自動化之間的關係。
- Wolf, C., & Glaser, J. (2015). "Yosys -- A Free Verilog Synthesis Suite." _Proceedings of the 21st Austrian Workshop on Microelectronics (Austrochip)_. Yosys 的原始論文，介紹了其架構與能力。
- Wolf, C., Shah, D., et al. (2018). "nextpnr: A Framework for Portable FPGA Place and Route." _Proceedings of the 8th International Workshop on FPGAs_. nextpnr 的架構設計論文。
- Kranenburg, T., & Wolf, C. (2015). "Project IceStorm: Reverse Engineering the Lattice iCE40 Bitstream." _Proceedings of the 2015 ACM/SIGDA International Symposium on FPGAs_. IceStorm 專案的逆向工程方法。

## 延伸閱讀

- [EDA - Electronic Design Automation (Wikipedia)](https://en.wikipedia.org/wiki/Electronic_design_automation)
- [VLSI (Wikipedia)](https://en.wikipedia.org/wiki/Very_Large_Scale_Integration)
- [System-on-a-chip (Wikipedia)](https://en.wikipedia.org/wiki/System-on-a-chip)
- [Yosys Open Source Suite (Wikipedia)](https://en.wikipedia.org/wiki/Yosys_(software))
- [nextpnr (Wikipedia)](https://en.wikipedia.org/wiki/nextpnr)
- LaLlement, J., & Al. (2020). "The Democratization of FPGA Development." _Communications of the ACM_, 63(5), 44--46. 討論開源 FPGA 工具鏈對產業與教育的影響。
