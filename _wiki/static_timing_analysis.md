# 靜態時序分析 (Static Timing Analysis)

## 什麼是靜態時序分析

靜態時序分析（Static Timing Analysis, STA）是數位積體電路設計中用於驗證時序正確性的關鍵方法。與傳統的動態模擬（Dynamic Simulation）不同，STA 不需要輸入測試向量，而是透過窮舉所有可能的邏輯路徑，計算每條路徑的延遲，並與給定的時序約束進行比對。

STA 的完整度是其最大優勢：動態模擬僅能檢查模擬中觸發到的路徑，而 STA 則能覆蓋每一條從起點到終點的邏輯路徑，無論該路徑是否會在實際操作中被觸發。然而，STA 不考慮電路的邏輯功能，這意味著它無法辨識「實際上不可能發生的路徑」──這是後續章節中「假路徑」概念要解決的問題。

STA 的運作建立在以下前提之上：
- 每個邏輯閘的延遲已透過單元庫（Standard Cell Library）的特性化資料（Liberty .lib）給定
- 每條互連線的延遲由 RC 參數提取（Parasitic Extraction）計算
- 時脈網路已定義明確的時脈源、頻率、相位關係

## 標準單元時序模型

STA 的計算基礎源自標準單元的時序特性。在 Liberty 格式中，每個單元的延遲透過非線性延遲模型（NLDM）以二維查找表表示。查表的兩個索引為輸入轉換時間（Input Slew）與輸出負載電容（Output Load Capacitance），對應的值為單元延遲與輸出轉換時間。

這個模型讓 STA 工具能夠根據實際電路的連線情況，計算每個單元的精確延遲。單元延遲與互連延遲疊加後，即為一條完整路徑的總延遲。

## 路徑類型

STA 將所有時序路徑分為四種類型：

### 暫存器到暫存器 (Reg-to-Reg)

最常見的路徑類型。從一個正反器的時脈輸入開始，經過該正反器的 CK→Q 延遲、其間的組合邏輯、到達下一個正反器的資料輸入（D pin）。這種路徑定義了管線中的最大操作頻率。

一條 Reg-to-Reg 路徑的延遲組成：
1. 啟動時脈路徑延遲（Launch Clock Path Delay）— 從時脈源到第一個正反器的時脈輸入
2. 正反器 CK→Q 延遲（Clock-to-Q Delay）
3. 組合邏輯延遲（Combinational Logic Delay）— 中間所有邏輯閘與互連線的延遲總和
4. 目標正反器的 setup 時間

### 暫存器到輸出 (Reg-to-Output)

從正反器的時脈輸入經過 CK→Q、組合邏輯、到達晶片的輸出接腳。此類路徑用於驗證輸出訊號相對於時脈的延遲是否符合 I/O 時序約束。

### 輸入到暫存器 (Input-to-Reg)

從晶片的輸入接腳經過組合邏輯到達正反器的資料輸入。用於驗證輸入訊號在時脈邊緣前抵達正反器的時間是否足夠。

### 輸入到輸出 (Input-to-Output)

從輸入接腳直接經過組合邏輯到達輸出接腳，不涉及任何正反器。此為純 combinational path，其延遲限制通常由 I/O 規格定義。

## Setup 與 Hold 時序

STA 中最核心的兩個檢查為 Setup 檢查與 Hold 檢查。兩者均涉及資料信號與時脈信號之間的相對時間關係。

### Setup 檢查 (Setup Timing Check)

Setup 時間定義為：在時脈的有效邊緣（Capture Edge）到來之前，資料輸入必須穩定就緒的最小時間。若資料在 setup 窗口內發生變化，正反器可能進入亞穩態（Metastability），輸出處於未知狀態。

Setup 檢查的關鍵公式：

```
Data Required Time = Capture Clock Edge + Capture Clock Path Delay - Setup Time
Data Arrival Time = Launch Clock Edge + Launch Clock Path Delay + CK-to-Q Delay + Combi Delay
Slack = Data Required Time - Data Arrival Time
```

若 Slack 為正數，表示資料在截止時間前抵達，時序滿足；若為負數，則發生 setup violation，設計可能無法在目標頻率下正常運作。

### Hold 檢查 (Hold Timing Check)

Hold 時間定義為：在時脈邊緣到來之後，資料輸入必須保持穩定的最短時間。若資料在 hold 窗口內變化，同樣會導致亞穩態。

Hold 檢查的關鍵公式：

```
Data Required Time = Capture Clock Edge + Capture Clock Path Delay + Hold Time
Data Arrival Time = Launch Clock Edge + Launch Clock Path Delay + CK-to-Q Delay + Combi Delay
Slack = Data Arrival Time - Data Required Time
```

注意到 Hold 檢查的 Slack 方向與 Setup 相反。Hold violation 發生在資料太快抵達目標暫存器、在 capture edge 之後未能保持穩定。Hold violation 無法透過降低頻率解決，必須透過插入延遲緩衝器（Delay Buffer）或最佳化時鐘偏移來修復。

### 正反器時序參數

正反器的時序特性包含以下關鍵參數：
- **Setup Time (tsu)**：資料在時脈邊緣前必須穩定的最短時間
- **Hold Time (thd)**：資料在時脈邊緣後必須保持穩定的最短時間
- **Clock-to-Q Delay (tcq)**：時脈邊緣到資料輸出變化之間的延遲
- **Minimum Pulse Width**：時脈高電位或低電位的最小持續時間
- **Recovery/Removal Time**：類似於非同步重置/設定的 setup/hold 參數

## 時脈偏斜 (Clock Skew)

時脈偏斜（Clock Skew）是指時脈訊號到達不同正反器的時間差異，主要由時脈分佈網路的延遲不匹配所造成。時脈偏斜對 Setup 與 Hold 有不同的影響：

- **Useful Skew**（對 Setup 有利）：若 capture clock 的到達時間比 launch clock 晚（positive skew），Setup slack 會增加，因為資料有更多時間傳播。
- **Hold Penalty**（對 Hold 有害）：同樣的 positive skew 會減少 Hold slack，因為資料和時脈都在較晚時間到達 capture register。

時脈樹合成（Clock Tree Synthesis, CTS）的目的之一就是最小化時脈偏斜。然而，先進設計中設計師也常故意導入「有用偏斜」（Useful Skew）來幫助收斂 Setup 時序，同時須確保 Hold 時序仍然滿足。

## Slack

Slack 是 STA 中衡量路徑時序裕量的核心指標。它代表 Required Time（需求時間）與 Arrival Time（到達時間）之間的差距：

- **Positive Slack**：路徑符合時序約束，有設計裕量
- **Negative Slack**：路徑違反時序約束，必須優化
- **Worst Negative Slack (WNS)**：所有路徑中最差的 Slack 值，代表整個設計的時序瓶頸
- **Total Negative Slack (TNS)**：所有違規路徑的 Slack 之和，反映整體時序違規的嚴重程度

在 STA 報表中，WNS 與 TNS 是兩個最重要的品質指標。WNS 越接近零越好，TNS 為零代表所有路徑都滿足時序。

## 假路徑 (False Path)

假路徑（False Path）是指電路中存在、但在正常工作模式下永遠不會被啟動的邏輯路徑。由於 STA 不考慮邏輯功能，它會將這些路徑納入分析，導致設計師對時序狀況產生誤判。常見的假路徑包括：

- **測試模式路徑**：掃描鏈（Scan Chain）在測試模式下的路徑，在正常操作中不作用
- **靜態控制訊號**：如 reset 或 enable 訊號在特定狀態下的邏輯路徑
- **跨時脈域的非同步路徑**：不同時脈域之間的訊號經過同步器後，STA 不需要分析原始的不穩定路徑
- **彼此互斥的多工器路徑**：通過多工器但永遠不會同時啟動的路徑分支

設計師透過 SDC 約束中的 `set_false_path` 命令告知 STA 工具忽略這些路徑。如果假路徑未正確定義，可能導致不必要的時序違規或過度約束。

## 多週期路徑 (Multi-Cycle Path)

多週期路徑（Multi-Cycle Path）指的是經過邏輯合成設計後，該路徑的資料傳輸需要多個時脈週期才能穩定。這種路徑並非時序違規，而是故意設計的行為。常見應用包括：

- 緩慢的算術運算（如多週期乘法器）
- 時脈門控（Clock Gating）的 enable 訊號
- 某些狀態機中的控制路徑

設計師在 SDC 中使用 `set_multicycle_path` 命令指定所需週期數。例如 `set_multicycle_path -setup 2` 表示資料有兩個時脈週期的時間來傳播。

## 晶片上變異 (On-Chip Variation, OCV)

隨著製程微縮到深次微米（65nm 以下），同一顆晶片不同位置的電晶體特性也會出現顯著差異，稱為晶片上變異（On-Chip Variation, OCV）。OCV 的成因包括：

- **製程變異**：閘極長度、氧化層厚度、摻雜濃度在晶片上的空間變化
- **電壓降（IR Drop）**：電源分佈網路造成的局部電壓差異
- **溫度梯度**：晶片不同區域的溫度不同，影響載子遷移率

STA 透過 OCV derating 來處理此問題：在分析 setup 時，對 launch path 使用 slow corner 延遲（縮放係數 > 1），對 capture path 使用 fast corner 延遲（縮放係數 < 1），模擬最差情況。對於 hold 分析則相反。

## CRPR (Clock Reconvergence Pessimism Removal)

在 OCV 分析中，launch clock path 與 capture clock path 往往會經過一段共同的時脈網路（例如從同一個 PLL 到時脈樹分岔點）。如果對這段共同路徑同時應用 Launch 與 Capture 的 OCV derating，會導入不必要的悲觀（Pessimism），因為實際電路上這段路徑的變異是共用的。

CRPR（Clock Reconvergence Pessimism Removal）又稱為 CPPR（Clock Path Pessimism Removal），其做法是計算共同路徑的延遲差異（Launch 與 Capture 在該段的 derated 延遲差），並將這個差值從 Slack 中減去，恢復到實際更準確的時序預測。

## 跨時脈域分析 (Clock Domain Crossing, CDC)

在現代 SoC 設計中，單一晶片常含有多個非同步時脈域（Asynchronous Clock Domains）。不同時脈域之間的訊號傳遞必須經由同步器（Synchronizer）處理，否則會產生亞穩態（Metastability）問題。

STA 處理跨時脈域的方式如下：

- **同步路徑（Synchronized Paths）**：經過同步器（典型為兩級正反器）的路徑，STA 可分析其時序（通常為 multicycle 設定）
- **非同步假路徑（Asynchronous False Paths）**：未經同步的路徑，在 SDC 中設定為 false path
- **CDC 驗證工具**：除 STA 之外，還需要專用的 CDC 驗證工具（如 RealIntent CDC、Synopsys SpyGlass CDC），分析同步器結構是否正確、是否有 combinational path 繞過同步器等

跨時脈域的錯誤是晶片功能失效的首要原因之一，因此 CDC 分析已成為時序簽核流程中不可或缺的一環。

## 時脈閘控檢查 (Clock Gating Checks)

時脈閘控（Clock Gating）是降低動態功耗的常用技術，透過在時脈路徑上加入閘控邏輯（如 AND 或 latch-based 閘控），在不需要時關閉特定模組的時脈。

時脈閘控的 STA 需要特別檢查：

- **Clock Gating Setup/Hold**：閘控訊號（enable）相對於時脈邊緣必須滿足特定的建立/保持時間，否則時脈脈衝可能被截斷（Glitch）或丟失
- **閘控單元時序**：閘控單元（如 integrated clock gating cell, ICG）本身的時序特性需要準確建模
- **Gate-Level 時脈路徑**：經過閘控後的時脈路徑需要使用 generated clock 正確定義

## 非同步時序檢查

除了標準的 setup/hold 檢查外，STA 還需要處理以下非同步時序檢查：

### Recovery 與 Removal

類似於 DFF 的 setup/hold，但針對非同步控制訊號：
- **Recovery Time**：非同步重置（Reset）或設定（Set）在時脈有效邊緣前必須解除穩定的最短時間，類似 setup
- **Removal Time**：非同步重置或設定在時脈有效邊緣後必須保持解除穩定的最短時間，類似 hold

### 資料至資料檢查 (Data-to-Data Check)

某些設計需要檢查兩個資料訊號之間的相對時序（不涉及時脈），例如匯流排的 turn-around 時間、記憶體位址與寫入致能訊號之間的 timing。

## 時序報告解讀

STA 工具會產生詳細的時序報告，以 Synopsys PrimeTime 格式為例，一份典型的 setup 路徑報告包含以下資訊：

```
Startpoint: U1_reg/Q (rising edge)
Endpoint:   U2_reg/D (rising edge)
Path Group: clk
Path Type:  max

  Point                                              Incr     Path
  -------------------------------------------------  ------   ------
  clock clk (rise edge)                              0.00     0.00
  clock network delay (propagated)                   0.50     0.50
  U1_reg/CP (DFF_X1)                                 0.00     0.50 r
  U1_reg/Q (DFF_X1)                                  0.35     0.85 r
  U3/ZN (NAND2_X1)                                   0.12     0.97 r
  U4/ZN (NOR2_X1)                                    0.18     1.15 f
  U5/ZN (AOI21_X1)                                   0.22     1.37 r
  U2_reg/D (DFF_X1)                                  0.00     1.37 r
  data arrival time                                           1.37

  clock clk (rise edge)                              2.00     2.00
  clock network delay (propagated)                   0.40     2.40
  U2_reg/CP (DFF_X1)                                 0.00     2.40 r
  library setup time                                 -0.15     2.25
  data required time                                          2.25
  ----------------------------------------------------------------
  data required time                                          2.25
  data arrival time                                           1.37
  slack (MET)                                                  0.88
```

報告中的關鍵欄位：
- **Incr**：該段延遲的增量值
- **Path**：累積的總延遲
- **r/f**：表示上升或下降邊緣

Data Required Time 與 Data Arrival Time 的差距即為 Slack。Slack 為正表示滿足時序（MET），為負表示違規（VIOLATED）。

## 時序收斂策略

當 STA 報告出現負 Slack 時，設計師可採取以下策略進行時序收斂：

### 合成階段優化

- **邏輯重構（Logic Restructuring）**：改寫 RTL，減少邏輯深度
- **管線化（Retiming / Pipelining）**：在長 combinational path 中加入正反器
- **運算元排序（Operand Ordering）**：改變加法器或多工器的結構

### 實體實現階段優化

- **單元尺寸調整（Cell Sizing）**：將路徑上的單元從 X1 換成 X2/X4/X8
- **VT 交換（VT Swapping）**：將 HVT 單元換成 LVT（速度較快但漏電大）
- **緩衝器插入（Buffer Insertion）**：在長走線中插入 repeater 以改善延遲
- **實體最佳化（Physical Optimization）**：搬移單元位置以縮短關鍵路徑的繞線長度
- **時脈偏斜調整（Skew Tuning）**：引入 useful skew 改善 setup slack

### 約束調整

- **降低頻率**：作為最後手段，放寬時脈週期
- **多週期路徑標註**：確認某些路徑是否應設定為 multicycle
- **假路徑標註**：確認是否有需要設為 false path 的路徑

## 最差情況 vs. 最佳情況分析

根據時序檢查的特性，STA 需要在不同的製程角進行分析：

### Setup 分析 (慢角 / Worst-Case)

Setup 檢查需要最悲觀的傳播延遲假設，因此在以下條件進行：
- **慢製程（Slow Process, SS）**：電晶體開關最慢
- **低電壓（Low Voltage）**：驅動電流最小
- **高溫（High Temperature）**：載子遷移率最低

這些條件的組合產生了最大的資料路徑延遲與最小的 capture 窗口。

### Hold 分析 (快角 / Best-Case)

Hold 檢查需要最快的傳播延遲假設，因此在以下條件進行：
- **快製程（Fast Process, FF）**：電晶體開關最快
- **高電壓（High Voltage）**：驅動電流最大
- **低溫（Low Temperature）**：載子遷移率最高

這些條件讓資料最快到達目標暫存器，最容易違反 hold 時間。

### 多重角分析

在實際的晶片簽核流程中，STA 工具需要在所有相關的 PVT 角進行 setup 與 hold 分析，通常包含：

- SS / 0.9V / 125°C (最差 setup)
- SS / 0.9V / -40°C (低溫最差)
- TT / 1.0V / 25°C (典型)
- FF / 1.1V / -40°C (最快 hold)
- FF / 1.1V / 125°C (高溫最快)

## SDC 約束檔

Synopsys Design Constraints（SDC）是業界標準的時序約束格式，為 ASCII 文字檔，包含以下主要指令：

### 時脈定義

```
create_clock -name clk -period 10.0 [get_ports clk]
create_generated_clock -name clk_div2 -source [get_ports clk] -divide_by 2 [get_pins U1/Q]
```

`-period` 指定時脈週期（單位為奈秒），`-waveform` 可指定 duty cycle。

### I/O 延遲

```
set_input_delay -clock clk -max 2.5 [get_ports data_in]
set_output_delay -clock clk -min 0.5 [get_ports data_out]
```

輸入延遲代表外部晶片從時脈邊緣到資料抵達晶片引腳的時間；輸出延遲代表外部晶片從時脈邊緣到需要收到穩定資料的時間。

### 假路徑與多週期路徑

```
set_false_path -from [get_clocks test_clk]
set_multicycle_path -setup 2 -from [get_pins U1/Q] -to [get_pins U2/D]
```

### 時脈不透明度 (Clock Uncertainty)

```
set_clock_uncertainty -setup 0.1 [get_clocks clk]
set_clock_uncertainty -hold 0.05 [get_clocks clk]
```

用於估計時脈抖動（Jitter）與其他不確定性。

## STA 與動態模擬的比較

| 特性 | STA | 動態模擬 |
|------|-----|---------|
| 覆蓋率 | 窮舉所有路徑 | 僅限測試向量觸及的路徑 |
| 輸入需求 | 時脈定義 + 約束 | 測試向量序列 |
| 執行速度 | 快（分鐘小時） | 慢（模擬器，取決於向量長度） |
| 功能驗證 | 不考慮功能 | 可驗證邏輯正確性 |
| 假路徑處理 | 需人工標註 | 自動排除（因無測試向量觸發） |
| 時序解析度 | 精確到最差情況路徑 | 取決於輸入向量是否觸發最差情況 |
| 功耗相關 | 無 | 可做功耗分析 |

兩者在晶片設計驗證中互補而非互斥：STA 確保時序收斂，動態模擬確保功能正確。完整的驗證流程兩者都需要。

## 時序簽核 (Timing Signoff)

時序簽核是晶片設計流程中最後的 STA 階段，也是決定設計是否可提交量產的關鍵關卡。簽核流程包含以下步驟：

1. **提取最終的 RC 參數**：在完成繞線後，從 GDSII 佈局中提取所有互連線的寄生電阻與電容，生成 SPEF（Standard Parasitic Exchange Format）檔案
2. **載入簽核級單元庫**：使用最精確的 Liberty 模型，包含完整 PVT 角的 timing 與 power 資料
3. **執行全角 STA**：在所有 PVT 角執行 setup 與 hold 分析
4. **檢查假路徑定義**：確保所有 false path 與 multi-cycle path 正確無誤
5. **收斂所有違規**：修復所有負 Slack 路徑（透過單元尺寸調整、緩衝器插入、邏輯重構等）
6. **生成簽核報告**：包含 WNS、TNS、時序路徑報告、以及 worst-case 路徑列表

時序簽核通過後，設計才能進入光罩製作（Tapeout）階段。此時如果仍有未修復的時序違規，晶片回來後可能無法在目標頻率下正常運作，導致功能失效或良率下降。

## FPGA 時序分析

FPGA 的時序分析概念與 ASIC STA 高度相似，但存在以下差異：

### FPGA 架構的影響

- **LUT 與互連延遲**：FPGA 的邏輯單元（LUT + FF）與可編程互連網路有固定的延遲模型，這些延遲由 FPGA 廠商提供（而非 ASIC 的標準單元庫）
- **互連路徑的預測性低**：相同設計放置在不同位置可能產生截然不同的時序結果
- **廠商工具整合 STA**：Xilinx Vivado、Intel Quartus 以及 Lattice iCEcube2 都內建完整的 STA 引擎，自動分析設計中的所有路徑

### iCE40 與 eda4

Lattice iCE40 FPGA 的時序分析由 nextpnr 等工具支援。nextpnr 可進行時序驅動的佈局繞線（Timing-Driven PNR），在模擬退火過程中將時序資訊作為成本函數的一環，引導佈局繞線演算法優先優化關鍵路徑。

eda4 專案中的 v2f-pnr 模組目前實現了模擬退火基礎佈局繞線，但尚未整合時序分析。未來若加入以下功能，將可實現完整的純 Rust FPGA 時序驅動 PNR：

1. **時脈定義**：從 Verilog 中提取時脈資訊，定義 launch/capture 關係
2. **單元延遲模型**：建立 iCE40 邏輯單元（LUT、FF、Carry 等）的延遲模型
3. **互連延遲估算**：基於 Manhattan 距離或更精確的繞線模型估算走線延遲
4. **路徑列舉與分析**：從網表中列舉所有 Reg-to-Reg 路徑，計算 Slack
5. **時序驅動的成本函數**：在 PNR 模擬退火中，加入 WNS/TNS 作為優化目標

在 v2f-synth 層級，合成的品質（邏輯層數、扇出分佈）也會直接影響最終時序結果。透過與 v2f-pnr 的整合，未來 eda4 可實現完整的 FPGA 設計閉環流程，包括時序分析與優化。

## 結語

STA 是數位積體電路設計中最關鍵的驗證技術之一。無論是 ASIC 還是 FPGA，時序收斂都是設計能否成功運作的核心條件。edc4 專案雖然以 FPGA 為目標平台，但其自動化佈局繞線引擎的發展路徑必然需要納入 STA 能力，才能真正實現在純 Rust 生態中完成完整的 FPGA 設計流程。
