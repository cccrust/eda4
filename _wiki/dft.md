# Design for Test (DFT)

## 什麼是 DFT

Design for Test（簡稱 DFT）是一套在晶片設計階段加入可測試性的設計技術。其核心目標是讓製造出來的晶片能夠透過測試程式有效率地檢測出製程缺陷，確保出貨品質。DFT 並非功能性驗證（functional verification），而是專門針對物理製造缺陷所設計的測試機制。

在現代奈米製程下，一顆 5nm 晶片可能容納數十億顆電晶體，即便良率達到 99.9999%，仍然會有數千顆缺陷。DFT 正是為了捕捉這些缺陷而存在。

## 為什麼需要測試

晶片製造過程中有許多環節可能引入缺陷：

- **短路（short）**：不該相連的導線連在一起，可能是由金屬殘留或光罩誤差造成
- **開路（open）**：導線斷裂，可能是因為蝕刻過度或金屬疲勞
- **雜質汙染（impurity）**：製程中的微粒落在晶圓表面，造成異常
- **光罩誤差（mask error）**：微影對位偏移導致圖案變形
- **氧化層缺陷**：閘極氧化層厚度不均或有針孔，導致漏電或擊穿
- **閾值電壓漂移**：製程變異導致電晶體開關特性偏離規格

一顆 5nm 晶片上有數百億顆電晶體，每顆只要有一處缺陷就可能導致晶片報廢。測試的目標就是在出貨前以合理的成本篩出這些不良品。

## 功能測試 vs 製造測試

功能測試（functional test / verification）與製造測試（manufacturing test）在本質上完全不同：

| 項目 | 功能測試 | 製造測試 |
|------|----------|----------|
| 目的 | 確認設計正確 | 確認製造無缺陷 |
| 執行時間 | 設計階段 | 生產階段 |
| 測試對象 | RTL / netlist 模擬 | 實體晶片 |
| 向量來源 | 驗證工程師撰寫 | ATPG 自動產生 |
| 覆蓋率目標 | 功能覆蓋率 | 缺陷覆蓋率（fault coverage）|

製造測試不需要驗證所有功能行為，只需要夠多的測試向量來覆蓋可能的物理缺陷。

## 故障模型（Fault Models）

為了讓 ATPG 工具能自動產生測試向量，必須先定義故障的數學模型。最常見的模型包括：

### Stuck-at 故障

最經典的模型。假設某條信號線永久性地 stuck-at-0（SA0）或 stuck-at-1（SA1）。例如一個 NAND 閘的輸出 stuck-at-0，無論輸入如何變化，輸出永遠是 0。

測試 stuck-at 故障的方法：對該節點施加與故障值相反的信號，並將該值傳播到輸出端（fault propagation / path sensitization）。

### Stuck-open 與 Stuck-on 故障

- **Stuck-open**：電晶體永遠處於截止狀態，無法導通。這會讓動態邏輯電路的輸出浮接（floating），表現類似於前一狀態被保持（sequential behavior）。
- **Stuck-on**：電晶體永遠處於導通狀態，導致電源和地之間出現直流通路，輸出電壓可能介於 VDD 和 GND 之間。

### 橋接故障（Bridging Fault）

兩條不該相連的信號線因製程缺陷而短路。可以分為 AND-bridge（強者拉弱）和 OR-bridge（弱者拉強）等模型。

### 延遲故障（Delay Fault）

- **Transition Delay Fault**：假設某節點的上升或下降轉換比預期慢。需要施加兩個連續的向量（launch 與 capture）來測試。
- **Path Delay Fault**：考慮整條路徑的累積延遲是否超過時脈週期。更真實但也更複雜。

現今量產測試主要以 stuck-at 和 transition delay 為主流，前者覆蓋 DC 缺陷，後者覆蓋 AC timing 問題。

## ATPG（Automatic Test Pattern Generation）

ATPG 是 EDA 工具的核心組件之一，負責自動產生測試向量。給定一個電路網表（netlist）和故障列表（fault list），ATPG 會計算出能夠區分好電路與故障電路的輸入向量。

### 經典演算法

- **D-algorithm（1966，Roth）**：第一個完整的 ATPG 演算法。使用 D-calculus 符號系統（D 表示好電路為 1 故障電路為 0，D' 則相反），透過 D-frontier 的概念將故障效應傳播到輸出。
- **PODEM（1981，Goel）**：將問題轉化為分支搜尋，只對原始輸入（primary input）做決策，大幅減少回溯次數。
- **FAN（1983，Fujiwara & Shimono）**：加入 head line 與 multiple backtrace 概念，進一步減少搜尋空間。

### 現代 ATPG 流程

1. 讀入 netlist 並建立電路拓樸
2. 讀入故障列表（可由 fault simulation 或 fault collapsing 產生）
3. 依序對每個目標故障執行：
   - **Fault excitation**：在故障節點施加與故障值相反的信號
   - **Fault propagation**：沿著靈敏路徑將 D 值傳到輸出
   - **Line justification**：將所有需要的信號值回溯到原始輸入
4. 將產生的測試向量寫入測試資料檔

### 壓縮與最佳化

ATPG 產生的原始向量集通常需要經過 **static compaction**（合併相容向量）和 **dynamic compaction**（在產生過程中同時考慮多個故障），以減少測試時間和測試資料量。

## 故障覆蓋率（Fault Coverage）

故障覆蓋率是衡量測試品質的關鍵指標：

```
Fault Coverage = (檢測到的故障數) / (可檢測的故障總數)
```

- Stuck-at 覆蓋率目標：一般 > 95%，車用或航太級 > 98%
- Transition delay 覆蓋率目標：一般 > 90%
- 某些冗餘電路中的故障可能無法被檢測（redundant fault），可以從分母中扣除

計算覆蓋率需要 **fault simulation**：給定一組測試向量，模擬好電路和故障電路的輸出，確認哪些故障能被區分開來。

## 掃描鏈（Scan Chain）

掃描鏈是 DFT 中最核心的技術。基本原理是將電路中的每一個 D 型正反器（DFF）替換為 **掃描正反器（scan flop）**，然後把這些 scan flop 串聯成一個移位暫存器（shift register）。

### Scan Flop 結構

一個 scan flop 在標準 DFF 的基礎上增加了：
- **scan_in 輸入**：測試模式下的資料輸入
- **scan_enable 控制腳**：選擇正常模式或測試模式
- **內部多工器**：由 scan_enable 控制選擇 D（功能資料）或 scan_in（測試資料）

### Scan 測試流程

1. **Shift-in 階段**：將 scan_enable 設為 1，把測試向量依序從 scan_in 腳位移進入所有 scan flop
2. **Capture 階段**：將 scan_enable 設為 0，觸發一個時脈，讓組合邏輯的計算結果被 latch 進 scan flop
3. **Shift-out 階段**：將 scan_enable 設為 1，把捕獲到的響應從 scan_out 位移出來
4. **比對**：將 shift-out 的資料與預期的 golden response 進行比對

### Scan 的優缺點

**優點**：
- 每個 DFF 都可以直接被控制與觀察（controllability & observability）
- ATPG 可以將時序電路視為組合電路來處理（測試時間點固定為 capture 那一拍）
- 與標準邏輯合成流程整合度高

**缺點**：
- 增加晶片面積（每個 DFF 多一個 MUX，約 10–15%）
- 增加一組 scan_enable 全局信號（需要做 skew 管理）
- 測試時間與 scan chain 長度成正比
- 無法直接測試 timing 路徑（需搭配 at-speed test）

### 多條 Scan Chain

現代晶片使用數十到數百條平行的 scan chain，每條 chain 有自己的 scan_in 和 scan_out 腳，以縮短 shift-in 時間。Chain 的長度平衡（balancing）會影響測試時間的最佳化。

## At-Speed Scan Test

傳統 scan test 使用慢速測試時脈，只能測 stuck-at 故障。為了檢測 timing 缺陷，必須在 shift-in 完成後用正常工作頻率進行 capture（at-speed test）。

常見的 at-speed 測試方式：
- **Launch-on-Capture（LOC）**：shift-in 完成後連續給兩個功能時脈，第一個時脈產生轉換（launch），第二個時脈捕獲結果（capture）。
- **Launch-on-Shift（LOS）**：shift-in 的最後一拍同時作為 launch，緊接著一個快速 capture。需要 scan_enable 能在極短時間內切換。

## Boundary Scan（JTAG）

### IEEE 1149.1 標準

邊界掃描（Boundary Scan）是 IEEE 1149.1 標準定義的測試技術，最初是為了解決 PCB 板上晶片間互連的測試問題。它在每個晶片的 I/O 腳旁加入 boundary scan cell，串聯成環繞晶片邊界的移位暫存器。

### JTAG 信號介面

標準 JTAG 需要 5 個信號（部分實作只用 4 個，省略 TRST）：
- **TCK**：測試時脈（Test Clock）
- **TMS**：測試模式選擇（Test Mode Select）
- **TDI**：測試資料輸入（Test Data In）
- **TDO**：測試資料輸出（Test Data Out）
- **TRST**：測試重置（Test Reset，可選）

### TAP 控制器狀態機

JTAG 的核心是一個 16 狀態的有限狀態機，稱為 **TAP controller**。主要狀態包括：

- **Test-Logic-Reset**：初始狀態，TAP 處於 idle
- **Run-Test/Idle**：等待指令執行
- **Shift-DR** 與 **Shift-IR**：將資料或指令位移進暫存器
- **Capture-DR** 與 **Capture-IR**：將平行資料載入移位暫存器
- **Update-DR** 與 **Update-IR**：將移位暫存器的值鎖存到平行輸出

透過 TMS 在 TCK 的上升沿控制 TAP 在不同狀態之間切換。

### JTAG 指令

標準指令集包括：
- **BYPASS**：將 TDI 直接連到 TDO（透過 1-bit bypass register），用來跳過不參與測試的晶片
- **SAMPLE/PRELOAD**：在不干擾正常功能的情況下，捕捉 I/O 腳的狀態
- **EXTEST**：測試晶片間互連，將 boundary scan cell 輸出的測試值驅動到 I/O 腳上

延伸的公開指令還包括 **INTEST**（測試晶片內部邏輯）、**IDCODE**（讀取晶片識別碼）、**USERCODE** 等。

## BIST（Built-In Self-Test）

內建自我測試（BIST）是將測試產生器和響應分析器整合在晶片內部，讓晶片可以「自我測試」而不需要外部測試設備提供向量。

### BIST 架構

- **TPG（Test Pattern Generator）**：通常使用 LFSR（線性反饋移位暫存器）產生偽隨機測試向量
- **ORA（Output Response Analyzer）**：通常使用 MISR（多重輸入簽名暫存器）將輸出壓縮成一個簽名（signature）
- **BIST Controller**：負責啟動、執行、結束 BIST，並報告 pass/fail 結果

### BIST 的優點

- 降低對昂貴 ATE（自動測試設備）的依賴
- 支援 on-chip 測試（可在系統運作中進行啟動或定期測試）
- 可以測試 ATE 難以涵蓋的高速內部節點

### BIST 的缺點

- 晶片面積與功耗開銷
- LFSR 產生的偽隨機向量覆蓋率有限，可能需要額外的 deterministic 測試點
- 診斷能力較差（只知道 pass/fail，不容易定位故障位置）

## MBIST（Memory BIST）

記憶體陣列在現代 SoC 中佔據大部分面積，且 SRAM 的缺陷模式相當特殊（如位元單元失效、行列解碼器故障、感測放大器偏移等），因此有專門的 Memory BIST。

### MBIST 演算法

- **March C-**：經典的 March 演算法，包含 6 個 March element，可以檢測 stuck-at、transition、coupling 等多種 SRAM 缺陷
- **March C+**：March C- 的改良版，加入對某些複雜耦合故障的檢測能力
- **Checkerboard**：棋盤格寫入後讀回，主要檢測相鄰位元之間的短路

### MBIST 實作

典型的 MBIST 控制器包含：
- 位址產生器（可以產生遞增、遞減、跳躍等序列）
- 資料產生器（產生 March element 要求的資料樣式）
- 比較邏輯（將讀出值與預期值比對）
- 診斷暫存器（記錄第一個 fail 的位址）

## 測試壓縮（Test Compression）

隨著晶片規模持續成長，測試資料量變得極其龐大，測試壓縮技術應運而生。

### XOR 壓縮與 MISR

多條 scan chain 的輸出可以透過 XOR 網路壓縮成較少的觀測腳位。**MISR（Multiple-Input Signature Register）** 是更進階的壓縮方式，將多個串流壓縮成一個 n-bit 簽名，測試結束後與 golden signature 比對即可判斷好壞。

### X-Masking

在壓縮過程中，scan chain 輸出中可能出現未知值（X，例如來自未初始化的記憶體），這些 X 值會破壞簽名的正確性。X-masking 技術透過 mask 邏輯將含有 X 的 chain 遮罩掉，避免影響壓縮結果。

### EDT（Embedded Deterministic Test）

由 Mentor Graphics（現 Siemens EDA）提出的技術，在晶片內加入解壓縮邏輯（decompressor），讓 ATE 只需要提供少量的測試資料，晶片內部再將其擴展為完整的 scan chain 資料。輸入端使用 LFSR 加 phase shifter，輸出端使用 MISR 壓縮。

## iCE40 的 DFT 與 JTAG

Lattice iCE40 FPGA 系列的 DFT 相關資訊大多是專有（proprietary）且未公開的，使用者無法直接存取內部的 DFT 架構。不過，晶片上的 JTAG 介面是完全開放給使用者使用的。

### iCE40 JTAG 指令

iCE40 支援的 JTAG 指令（部分）：
- **BYPASS**：標準 IEEE 1149.1 BYPASS 指令
- **SAMPLE**：標準 SAMPLE/PRELOAD 指令
- **EXTEST**：標準 EXTEST 指令
- **USERCODE** / **IDCODE**：讀取裝置識別資訊
- **USR0 / USR1**：使用者自定義指令，用來觸發 FPGA 內部的使用者邏輯

### eda4 中的 JTAG 實作

在 eda4 專案的 `v2f-programmer` crate 中，實作了一個完整的 **JTAG TAP 狀態機**，支援以下功能：

1. **TAP 狀態機**：實作 IEEE 1149.1 定義的 16 狀態 TAP controller
2. **SVF（Serial Vector Format）播放器**：可以執行 SVF 指令來控制 JTAG 狀態序列
3. **iCE40 程式設計**：透過 JTAG 將位元流（bitstream）載入 FPGA 的 SRAM 配置記憶體
4. **SPI 介面（備用方案）**：在 FTDI 模式下也可以透過 SPI 對 iCE40 進行配置

### 在 eda4 中使用 JTAG

```
v2f program --device hx8k --jtag _out/blinky.bin
```

這條指令會：
1. 透過 FTDI（或模擬的）JTAG 介面連接到 iCE40
2. 切換 TAP 狀態機到 Shift-IR，載入 CFG_IN 指令
3. 切換到 Shift-DR，將 bitstream 位移入晶片
4. 完成後觸發 CRAM 初始化

## 測試點插入（Test Point Insertion, TPI）

當電路中的某些節點可控制性（controllability）或可觀察性（observability）太低，導致 ATPG 覆蓋率無法達標時，可以在設計中插入測試點（test point）來改善。

### 控制點（Control Point）

在難以設定的節點上加入一個 AND 或 OR 閘，透過測試模式下的額外控制信號強制節點為 0 或 1：

- **AND-type control point**：測試模式下可以強制輸出為 0（control 信號為 0 時，AND 閘輸出固定為 0）
- **OR-type control point**：測試模式下可以強制輸出為 1（control 信號為 1 時，OR 閘輸出固定為 1）

### 觀察點（Observation Point）

在難以觀測的節點上拉出一條測試線，透過一個 flip-flop 暫存該節點的值，再連接到 scan chain 上。這讓 ATPG 不需要透過長路徑傳播故障效應到原始輸出。

### 插入策略

- SCOAP（Sandia Controllability/Observability Analysis Program）是經典的測試度量分析工具，計算每個節點的 controllability 和 observability 權重
- TPI 會增加面積和 routing 資源，需要在覆蓋率和成本之間取得平衡

## IDDQ 測試

IDDQ 測試（靜態電源電流測試）是一種不需向量就能檢測某些缺陷的補充測試方法。

原理：CMOS 電路在靜態（無切換）時幾乎不消耗電流（僅有漏電）。如果某個 defect 造成電源到地的直流通路（如 bridging fault 或 stuck-on fault），靜態電流會異常升高。

### 優點
- 對某些 ATPG 難以檢測的缺陷非常敏感（如 bridging fault、gate-oxide short）
- 可以在 wafer sort 階段快速篩出 gross defect

### 缺點
- 隨著製程微縮，漏電流急劇上升，背景電流（background leakage）與故障電流的 SNR 越來越低
- 16nm以下的先進製程中 IDDQ 的鑑別力大幅下降
- 測試時間較長（需要等電源穩定）

現今先進製程中 IDDQ 已從量產測試的主流退居為可靠度監控和缺陷分析的工具。

## Wafer Sort 與 Final Test

量產測試通常分為兩個階段：

### Wafer Sort（晶圓測試 / CP, Chip Probing）

- 在晶圓尚未切割時進行
- 使用探針卡（probe card）接觸晶粒的 pad
- 篩出明顯缺陷的晶粒（gross defect），避免封裝成本浪費
- 測試項目：DC 參數、functional test、scan test
- 環境溫度：常溫

### Final Test（最終測試 / FT）

- 封裝完成後進行
- 所有晶粒腳位均可接觸（封裝後不需要探針限制）
- 更完整的測試項目：at-speed、全溫度範圍（冷、常、熱）
- 分級（binning）：根據測試結果將晶片分為不同速度或電壓等級

### 測試成本經濟學

- **測試成本佔晶片總成本的 5–30%**，視晶片複雜度和產量而定
- 測試時間越長，成本越高（ATE 每小時收費數百到數千美元）
- 測試壓縮和 BIST 的主要動機就是減少 ATE 測試時間
- **Known-Good Die（KGD）** 的概念：只有通過測試的晶粒才能進入封裝或 MCM/SiP

## DFT 在 FPGA 中的特殊性

FPGA 與 ASIC 在 DFT 方面有本質差異：

- **可程式化性**：FPGA 的使用者邏輯是在出廠後才配置的，因此 FPGA 本身和它實現的邏輯需要分開測試
- **FPGA 晶片測試**：由原廠（Lattice、Xilinx、Intel）在出廠前完成，使用者無法接觸內部的 DFT 結構
- **使用者邏輯測試**：FPGA 設計可以（而且應該）像 ASIC 一樣加入 scan chain 和 BIST，但因為 FPGA 的 routing 資源有限，加上額外的 DFT 邏輯會佔用 LUT 和 flip-flop
- **Configuration Memory 測試**：FPGA 的 SRAM 配置記憶體是缺陷敏感區塊，原廠已有內建的記憶體測試機制

iCE40 屬於低功耗低成本 FPGA 系列，內部沒有完整的 DFT 基礎架構提供給使用者。使用者需要自行在 RTL 中插入 scan 或 BIST 邏輯（如果需要的話）。

## 總結

DFT 是現代晶片設計中不可或缺的一環。從基礎的 stuck-at 故障模型和 scan chain，到進階的 at-speed test、BIST、JTAG boundary scan，每項技術都在解決製造測試的特定面向。在 eda4 專案中，JTAG 介面是與 DFT 最直接相關的元件，透過 `v2f-programmer` crate 實作的 TAP 狀態機，可以對 iCE40 FPGA 進行程式設計和基本的測試操作。

## 參考文獻

- M. L. Bushnell, V. D. Agrawal, *Essentials of Electronic Testing for Digital, Memory and Mixed-Signal VLSI Circuits*, Springer, 2000
- IEEE Std 1149.1-2013, *IEEE Standard for Test Access Port and Boundary-Scan Architecture*
- Lattice Semiconductor, *iCE40 Programming and Configuration Technical Note* (FPGA-TN-02002)
- N. K. Jha, S. Gupta, *Testing of Digital Systems*, Cambridge University Press, 2003
- E. J. McCluskey, *Logic Design Principles with Emphasis on Testable Semicustom Circuits*, Prentice Hall, 1986
