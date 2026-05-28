# Clocking、時脈樹合成與跨時脈域

## 什麼是時脈

時脈（clock）是數位電路中最重要的信號之一。它是一個週期性的方波（square wave），用來同步電路中所有循序元件（flip-flop、latch、counter、register）的操作。每個循序元件在時脈的上升沿（posedge）或下降沿（negedge）觸發資料更新，確保資料在整個電路中按照預定的節奏流動。

沒有時脈，大規模數位邏輯將無法協調運作。時脈的品質（頻率穩定度、抖動、相位雜訊）直接決定晶片的效能極限。

## 時脈基本術語

- **頻率（Frequency）**：時脈每秒的週期數，單位 Hz（MHz、GHz）
- **週期（Period）**：頻率的倒數，一個完整方波的時間長度，單位 ns、ps
- **工作週期（Duty Cycle）**：高電位持續時間佔整個週期的比例，理想方波為 50%
- **上升時間（Rise Time）**：信號從 20% 上升到 80% VDD 所需的時間
- **下降時間（Fall Time）**：信號從 80% 下降到 20% VDD 所需的時間
- **抖動（Jitter）**：時脈邊緣在時間上的隨機偏移，通常以 ps rms 或 ps peak-to-peak 表示
- **偏移（Skew）**：同一個時脈源到達不同暫存器的時間差異
- **潛伏時間（Latency）**：時脈從源頭到達目標暫存器的傳播延遲

## 時脈產生

數位系統中的時脈可以由多種電路產生：

### 晶體振盪器（Crystal Oscillator）

利用石英晶體的壓電效應產生極穩定的參考頻率。具有高 Q 值、低相位雜訊、高溫度穩定度等優點。缺點是無法整合在晶片內部（離散元件），且輸出頻率固定。

### RC 振盪器

利用電阻電容充放電產生振盪。可完全整合在晶片內部（on-chip oscillator），但頻率精度和穩定度遠不如晶體振盪器，受製程、電壓、溫度（PVT）影響較大。

iCE40 FPGA 內部包含一個 RC 振盪器，可配置為 10kHz 低功耗模式或 48MHz 高速模式，頻率誤差約 ±5–10%。

### PLL（Phase-Locked Loop，鎖相迴路）

PLL 是目前最廣泛使用的時脈產生與合成電路。它能從一個參考頻率產生更高或更低頻率的輸出，並能與參考信號保持相位同步。

**PLL 基本架構**：

```
參考頻率 (Fref)
    │
    ▼
┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│ Phase    │ ──► │ Loop     │ ──► │ VCO      │ ──► │ Frequency│ ──► 輸出
│ Detector │     │ Filter   │     │          │     │ Divider  │
└──────────┘     └──────────┘     └──────────┘     └──────────┘
    ▲                                                  │
    └────────────────── 回授分頻 ──────────────────────┘
```

- **Phase Detector（鑑相器）**：比較參考頻率與回授頻率的相位差，輸出一個與相位誤差成比例的電壓（或脈衝寬度信號）
- **Loop Filter（迴路濾波器）**：通常是低通濾波器，將鑑相器的脈衝輸出平滑為類比電壓，控制 VCO 的頻率
- **VCO（Voltage-Controlled Oscillator，壓控振盪器）**：輸出頻率由控制電壓決定的振盪器
- **回授分頻器（Feedback Divider）**：將 VCO 輸出除以 N，送回鑑相器與參考頻率比較

鎖定時：Fvco = Fref × N，可產生參考頻率的整數倍頻輸出。

### DLL（Delay-Locked Loop，延遲鎖定迴路）

DLL 與 PLL 類似，但 DLL 並非產生新的頻率，而是調整輸出信號相對於輸入的延遲量。DLL 利用可調延遲線（voltage-controlled delay line）取代 PLL 中的 VCO，鎖定時輸入與輸出的相位差為零或固定值。DLL 沒有頻率合成能力，主要用於 clock de-skew 和 DDR 記憶體介面。

## iCE40 PLL

iCE40 HX 系列和 UP 系列 FPGA 內含一個 PLL（每個元件一個，HX8K 有兩個）。主要規格：

- 輸入頻率範圍：10MHz – 133MHz
- 輸出頻率範圍：最大 275MHz（HX 系列）
- 支援整數倍頻與分頻（利用分頻器組合產生多種頻率）
- 支援全球時脈網路輸出或通用 routing 輸出
- 可由此特流（bitstream）配置暫存器

iCE40 PLL 的典型應用場景：
- 將外部低速參考時脈倍頻為內部高速運算時脈
- 產生多個不同頻率的時脈域
- 調整時脈相位來滿足外部介面的時序需求

## 時脈分佈（Clock Distribution）

時脈信號必須送到晶片中所有使用到時脈的暫存器。由於時脈通常是最快、負載最大的信號，它的分佈網路需要特別設計。

### 時脈分佈的挑戰

- **高扇出（High Fanout）**：一顆 SoC 可能包含數十萬到數億個暫存器，每個都需要時脈
- **低偏移（Low Skew）**：所有暫存器應盡可能在相同時間接收到時脈邊緣
- **低功耗**：時脈網路在現代晶片中消耗 15–40% 的動態功耗
- **低抖動**：時脈分佈網路不應引入額外的抖動

## 時脈樹合成（Clock Tree Synthesis, CTS）

CTS 是實體設計（physical design）流程中從佈局（placement）到繞線（routing）之間的一個關鍵步驟。CTS 的目標是建立一個平衡的時脈分佈網路，將時脈從源頭（PLL 輸出或晶片腳位）送到每個暫存器的時脈輸入端，同時滿足偏移（skew）、延遲（latency）、功耗和面積的限制。

### CTS 流程

1. **時脈網路的定義**：指定哪些信號是時脈、時脈的根節點（root）在哪裡、哪些元件是 sink（暫存器）
2. **緩衝器插入（Buffer Insertion）**：在時脈路徑上插入反相器或緩衝器來驅動大量的暫存器
3. **樹狀結構建立**：利用演算法將暫存器分群，建立多層級的分佈結構
4. **偏移最佳化（Skew Optimization）**：調整緩衝器的尺寸和路徑長度，使所有 sink 的 clock arrival time 盡可能一致
5. **時序驗證（Timing Verification）**：在 CTS 完成後重新提取 RC 參數，進行靜態時序分析（STA）

### CTS 的目標

- **Skew**：在同一時脈域內，任意兩個暫存器的 clock arrival time 差異應在幾十到幾百 ps 以內
- **Latency**：時脈從 root 到 sink 的總延遲，影響設計的 timing margin
- **功耗**：時脈樹的緩衝器數量與尺寸直接影響動態功耗
- **可繞線性**：時脈樹不應過度佔用繞線資源，影響其他信號的 routing

## 時脈樹拓樸（Clock Tree Topologies）

### H-Tree

H-tree 是最傳統的平衡時脈分佈結構。以遞迴方式將晶片區域等分，並在每個階段的中心點放置緩衝器。形狀類似字母 H 的遞迴圖案。

**優點**：
- 理論上可達到極低的偏移（如果製程完全對稱）
- 結構規則，易於分析

**缺點**：
- 只對規則的晶片形狀有效
- 對非均勻的暫存器分佈適應性差
- 緩衝器數量和繞線資源消耗較大

### Fishbone（魚骨結構）

從一條主幹（spine）向兩側延伸分支（ribs），形似魚骨。適合長條形的晶片佈局。

### X-Tree

與 H-tree 類似但使用 X 形狀的分支模式，在一些實驗性設計中可提供更好的 skew 表現。

### Mesh（網格結構）

在晶片頂層使用金屬網格（grid）來分佈時脈，暫存器直接從最近的網格節點取時脈。Mesh 的 skew 極低但功耗較高，常用於高效能處理器。

### Hybrid（混合）

現代設計通常混合使用多種拓樸：頂層使用 H-tree 或 mesh，底層使用 fishbone 或局部樹狀結構。

## 時脈偏移（Clock Skew）

時脈偏移是指同一個時脈信號到達兩個不同暫存器的時間差。

```
Skew = Tclk2 - Tclk1
```

- **正偏移（Positive Skew）**：第二個暫存器比第一個晚收到時脈，可以放寬從第一個暫存器到第二個暫存器的 timing
- **負偏移（Negative Skew）**：第二個暫存器比第一個早收到時脈，會緊縮 timing

### 偏移的來源

- **製程變異（Process Variation）**：不同區域的電晶體特性不同
- **負載差異（Load Mismatch）**：不同路徑的扇出數不同
- **繞線長度差異（Wire Length Mismatch）**：金屬線長度不同導致 RC 延遲不同
- **溫度梯度（Temperature Gradient）**：晶片不同區域的溫度不同影響載子遷移率
- **電壓降（IR Drop）**：不同區域的電源電壓不同影響緩衝器速度

### 有用偏移（Useful Skew）

傳統 CTS 的目標是將 skew 最小化，但有意地引入特定方向的 skew（skew scheduling）可以改善 timing：

- 在 setup-critical 路徑上可以讓接收端暫存器的時脈延遲（positive skew），增加有效週期
- 在 hold-critical 路徑上則需要小心，因為 positive skew 會惡化 hold timing

## 時脈抖動（Clock Jitter）

抖動是時脈邊緣在理想時間點周圍的隨機或確定性偏移。不同於 skew（由空間差異造成，屬於靜態），jitter 是時間上的動態變化。

### 抖動的分類

- **週期抖動（Period Jitter）**：每個時脈週期長度與理想週期的偏差。直接影響 setup/hold timing margin。
- **Cycle-to-Cycle Jitter**：相鄰兩個週期長度的差異。對 DLL 和高速序列介面（如 SerDes）影響較大。
- **長期抖動（Long-Term Jitter）**：經過多個週期後累積的相位誤差。對同步協定和通訊介面重要。

### 抖動的來源

- **電源雜訊（Power Supply Noise）**：電源電壓的瞬態變化改變 VCO 頻率
- **基板雜訊（Substrate Noise）**：數位電路的切換雜訊透過基板耦合到類比 PLL 電路
- **熱雜訊（Thermal Noise）**：電阻和電晶體的熱雜訊是隨機抖動的基本來源
- **EMI / 耦合雜訊**：相鄰信號線的容性耦合

## 時脈閘控（Clock Gating）

時脈閘控是降低數位電路動態功耗最有效的方法之一。基本原理是在暫存器不需要更新資料時，將時脈關閉（或凍結），避免多餘的開關損耗。

### 實作方式

將時脈信號與一個 enable 信號 AND 起來（使用 AND gate 或 latch + AND gate）：

```verilog
// 基本 clock gating（可能有 glitch 問題）
assign gated_clk = clk & enable;

// 安全的 clock gating（使用 latch 避免 glitch）
always_latch begin
    if (!clk) enable_latched <= enable;
end
assign gated_clk = clk & enable_latched;
```

### Clock Gating 的效益

- 動態功耗降低：時脈樹中約 15–40% 的功耗來自不必要的暫存器開關，clock gating 可以大幅減少
- 面積節省：clock gating cell 比對每個暫存器加 MUX 的面積小
- 現代合成工具（Synopsys DC、Yosys）會自動推斷 clock gating

## 多時脈域與跨時脈域

### 多個時脈域

現代 SoC 中，不同的功能區塊通常使用不同的時脈頻率：
- CPU 核心：2–4 GHz
- 記憶體控制器：DDR 頻率
- 週邊介面（I2C、SPI、UART）：數十到數百 MHz
- 匯流排互連（AXI、AHB）：中間頻率

這種設計被稱為 **多時脈域（Multiple Clock Domains, MCD）**。

### 為什麼需要多時脈域

1. **功耗最佳化**：低效能區塊使用低頻時脈，不需要與 CPU 同頻
2. **IP 重複使用**：不同來源的 IP 核心可能使用不同的時脈
3. **外部介面要求**：外部晶片（DDR、Ethernet、USB）都有各自的時脈要求
4. **電磁干擾（EMI）控制**：分散頻譜可以降低 EMI 峰值

## 跨時脈域問題

當一個信號從一個時脈域 A 傳送到另一個非同步時脈域 B 時，會出現以下問題：

### 亞穩態（Metastability）

亞穩態是跨時脈域最根本的問題。當一個 flip-flop 的資料輸入在其 setup/hold window 內發生變化時，輸出會進入一個中間狀態（既不是 0 也不是 1），這個狀態可能持續任意長的時間。

亞穩態的特性：
- 輸出電壓徘徊在 VIL 和 VIH 之間
- 穩定的時間是不確定的（理論上無限長）
- 最終會收斂到 0 或 1，但收斂前的延遲服從指數分佈
- 收斂後的結果可能是 0 或 1（不一定是輸入值）

如果亞穩態的輸出在收斂之前被下游邏輯取樣，可能導致整個電路進入錯誤狀態。

### 資料一致性（Data Coherency）

當多個位元的匯流排跨時脈域時，不同位元可能在不同時間被取樣，導致接收端看到錯誤的組合（例如從 3'b011 變為 3'b111 或 3'b001）。

### 資料遺失（Data Loss）

當發送端更新的速度比接收端取樣的速度快時，某些資料值可能永遠不會被接收端看到。

## 跨時脈域解決方案

### 2-Flop Synchronizer（雙正反器同步器）

最簡單的單一位元同步電路：

```verilog
module sync_2ff (
    input  logic clk_b,
    input  logic data_in,
    output logic data_out
);
    logic sync_ff1, sync_ff2;
    always_ff @(posedge clk_b) begin
        sync_ff1 <= data_in;
        sync_ff2 <= sync_ff1;
    end
    assign data_out = sync_ff2;
endmodule
```

第一級 flip-flop 允許亞穩態存在，第二級 flip-flop 在一個完整的時脈週期後取樣，此時第一級輸出已有極高機率收斂。

### MTBF（Mean Time Between Failure）

MTBF 是衡量同步器可靠性的指標，計算公式（近似）：

```
MTBF = exp(tr / τ) / (W × fc × fd)
```

- tr：同步器的解析時間（通常為一個時脈週期）
- τ：flip-flop 的亞穩態時間常數（製程相關）
- W：亞穩態窗口寬度（setup + hold time）
- fc：接收端時脈頻率
- fd：資料變化頻率

MTBF 值越大越好。對於 2-flop synchronizer，適當的設計下 MTBF 可達數百萬年以上。

### FIFO（First-In First-Out）同步器

用於多位元資料傳輸，特別是連續串流的資料：

```verilog
module async_fifo #(
    parameter WIDTH = 8,
    parameter DEPTH = 16
) (
    input  logic             clk_w, rst_w, wr_en,
    input  logic [WIDTH-1:0] wr_data,
    output logic             full,
    input  logic             clk_r, rst_r, rd_en,
    output logic [WIDTH-1:0] rd_data,
    output logic             empty
);
    // 使用 grey code 同步讀寫指標
    // 雙埠 SRAM 儲存資料
    // 指標比較產生 full/empty 信號
endmodule
```

FIFO 的關鍵在於使用 grey code（葛雷碼）將讀寫指標跨時脈域同步，因為 grey code 相鄰值只差一位元，不會出現多位元同時變化的問題。

### 握手協定（Handshake）

使用 request/acknowledge 協定進行資料傳輸：

1. 發送端將資料放到匯流排上，然後拉高 request
2. 接收端取樣到 request 後，鎖存資料，拉高 acknowledge
3. 發送端看到 acknowledge 後，拉低 request
4. 接收端看到 request 變低後，拉低 acknowledge
5. 一個交易完成

優點：簡單可靠，不需要 FIFO 的儲存空間。
缺點：throughput 受限於來回握手延遲。

### MUX Recirculation

將資料多工器與暫存器組成回授環路，在跨時脈域時保持資料穩定性。常用於控制信號的非同步傳送。

### Grey Code（葛雷碼）

Grey code 的特性是相鄰兩個數值只有一個位元不同，非常適合用於跨時脈域的計數器同步：

```verilog
// Binary to Grey
assign grey = (binary >> 1) ^ binary;

// Grey to Binary
integer i;
always_comb begin
    binary[WIDTH-1] = grey[WIDTH-1];
    for (i = WIDTH-2; i >= 0; i--)
        binary[i] = grey[i] ^ binary[i+1];
end
```

常見應用：非同步 FIFO 中的讀寫指標同步、非同步計數器。

## 時脈域交叉檢查（CDC Verification）

靜態檢查工具（如 Synopsys SpyGlass CDC、Cadence JasperCDC）會分析設計中所有跨時脈域的路徑，並檢查：

1. 每個跨時脈域信號是否有合適的同步器
2. 同步器的類型是否正確（2-flop、FIFO、handshake）
3. 是否存在 unreachable 的同步路徑
4. 是否存在 combinational 信號直接跨時脈域（禁止！）
5. 重新收斂分歧（reconvergence）的問題

## iCE40 的時脈資源

### 全球時脈網路（Global Clock Network）

iCE40 系列提供多條全球時脈網路（global clock networks），可以將時脈信號以低偏移送到晶片中的所有邏輯區塊：

- iCE40 HX 系列：每顆裝置有 4 條 global clock 網路（GBUFx）
- 全球時脈輸入腳位：專用時脈腳位（GBINx）可以直接驅動 global clock 網路
- PLL 輸出也可以連接到 global clock 網路

### 內部振盪器

- **HFOSC**：高速振盪器，預設輸出 48MHz（可配置為 12MHz、24MHz、48MHz）
- **LFOSC**：低速振盪器，輸出約 10kHz，適合低功耗定時用途

### 時脈使用建議

- 設計中推導出的正反器應該使用同一個時脈域，避免不必要的跨時脈域同步
- 如果需要多個頻率，使用 PLL 產生第二個時脈，而非透過組合邏輯分頻（會產生 glitch）
- iCE40 的 PLL 輸出可以直接連接到 global clock 網路，不必經過一般 routing 資源

## eda4 中的時脈處理

在 eda4 專案中，verilog2fpga 與 verilog2rust 對時脈的處理原則如下：

1. **單一時脈域假設**：所有的範例設計（blinky、adder）都使用單一個時脈，所有正反器在同一時脈的 posedge 觸發
2. **DFF 推斷**：從 Verilog 的 `always_ff @(posedge clk)` 或 `always @(posedge clk)` 推斷出 DFF，並自動連接到時脈網路
3. **同步重置**：支援同步重置（synchronous reset），在時脈邊緣判斷 reset 信號
4. **PLL 例化**：在進階使用中可以例化 iCE40 的 PLL primitive（SB_PLL40_CORE 或 SB_PLL40_PAD）來產生第二個時脈
5. **時脈樹**：在純粹的 EDA 流程中，時脈樹的建構由 Yosys/nextpnr（或 v2f-pnr 的模擬退火演算法）處理

## 參考文獻

- J. Bhasker, R. Chadha, *Static Timing Analysis for Nanometer Designs: A Practical Approach*, Springer, 2009
- R. J. Baker, *CMOS: Circuit Design, Layout, and Simulation*, 4th Edition, Wiley, 2019
- Lattice Semiconductor, *iCE40 LP/HX Family Data Sheet* (DS1040)
- Lattice Semiconductor, *iCE40 Programming and Configuration Technical Note* (FPGA-TN-02002)
- C. E. Cummings, "Clock Domain Crossing (CDC) Design & Verification Techniques", SNUG 2008
- Synopsys, *PrimeTime User Guide: Clock Tree Synthesis Analysis*
- N. H. E. Weste, D. Harris, *CMOS VLSI Design: A Circuits and Systems Perspective*, 4th Edition, Addison-Wesley, 2011
