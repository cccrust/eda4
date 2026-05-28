# 電路分割：Kernighan-Lin 與 Fiduccia-Mattheyses 演算法

## 什麼是電路分割

電路分割（Circuit Partitioning）是電子設計自動化（EDA）中最基礎也最核心的問題之一。給定一個由邏輯閘（cells）和連線（nets）組成的電路，分割的目標是將電路劃分為多個部分（partitions），使得：

1. **切割最小化**：跨越多個部分的連線數量（稱為 cut size）最小
2. **大小平衡**：每個部分的元件數量或面積儘可能均衡
3. **其他約束**：某些設計可能需要滿足時序、功耗或特定元件群組約束

形式化地，給定一個超圖 \( H(V, E) \)，其中：
- \( V \) 為頂點集合（代表電路中的邏輯閘、暫存器、功能單元）
- \( E \) 為超邊集合（代表連接多個頂點的連線）

將 \( V \) 劃分為 \( k \) 個子集 \( P_1, P_2, ..., P_k \)，使得：
- \( \bigcup P_i = V \)（所有頂點都被分配）
- \( P_i \cap P_j = \emptyset \) for \( i \neq j \)（互不重疊）
- \( |P_i| \) 儘可能均勻（大小平衡）
- \( \sum_{e \in E} w(e) \times \text{cut}(e) \) 最小化，其中 \( \text{cut}(e) = 1 \) 若 \( e \) 的頂點分佈於多個部分，否則為 0

電路分割問題是經典的 NP-hard 問題，因此實務上依賴啟發式演算法來逼近最佳解。其中最具影響力的兩個演算法為 Kernighan-Lin（K-L）演算法（1970）和 Fiduccia-Mattheyses（F-M）演算法（1982）。

## 分割在 EDA 中的重要性

電路分割之所以是 EDA 的基石，是因為它在許多環節中都扮演關鍵角色：

### 分而治之（Divide and Conquer）

現代晶片設計包含數十億個電晶體，直接在完整設計上進行佈局與繞線在計算上是不可行的。分割提供了一種自然的分解策略：
1. 將大型電路分割為多個子電路
2. 對每個子電路獨立進行最佳化
3. 將結果合併

這種方法不僅降低了計算複雜度，也使平行處理成為可能。

### FPGA 分割

當一個設計太大而無法裝入單一 FPGA 時，必須將其分割到多個 FPGA 晶片上。這種多 FPGA 分割需要考慮：
- 每個 FPGA 的邏輯容量限制
- FPGA 之間的 I/O 腳位數量限制（通常很有限）
- 跨 FPGA 通訊的延遲

這是分割問題最直接的應用場景之一。

### 階層式佈局（Floorplanning）

在晶片實體設計中，分割被用於確定功能區塊的大致位置：
- 將相關的邏輯單元群組在一起
- 減少高頻寬區塊之間的通訊距離
- 優化晶片整體的熱分佈

### FPGA 封裝（Packing）

在 FPGA 設計流程中，分割被用於「封裝」或「聚類」步驟：
- 將 LUT 和正反器（FF）配對並組合成邏輯區塊（CLB / logic tile）
- 決定哪些 LUT+FF 對應到同一個 slice 或 CLB
- 考慮進位鏈、時脈使能等共用資源

### 時脈域分組

實際設計中不同時脈域的邏輯應盡量分開放置：
- 減少跨時脈域的走線
- 簡化時脈樹綜合
- 降低時脈偏差（clock skew）

## Kernighan-Lin 演算法

Kernighan-Lin 演算法由 Brian Kernighan 與 Shen Lin 於 1970 年發表於 The Bell System Technical Journal，論文題為 "An Efficient Heuristic Procedure for Partitioning Graphs"。這是電路分割領域最具開創性的工作之一。

### 問題模型

K-L 演算法處理由圖 \( G(V, E) \) 描述的電路，目標是將 \( V \) 劃分為兩個大小相等（或接近相等）的子集 \( A \) 和 \( B \)。

圖中的每條邊 \( (u, v) \) 有權重 \( w(u, v) \)（對於未加權圖，權重為 1）。演算法的目標是最小化連接 \( A \) 與 \( B \) 的邊的總權重，即 cut size。

### 增益計算

K-L 演算法的核心概念是**增益**（gain）。對每個頂點 \( v \)，定義：

- \( I(v) \) = 與 \( v \) 在同一分割內的其他頂點之間的邊權重總和（internal cost）
- \( E(v) \) = 與 \( v \) 在不同分割內的頂點之間的邊權重總和（external cost）
- \( D(v) = E(v) - I(v) \)

\( D(v) \) 衡量的是，如果將 \( v \) 移動到另一側，cut size 的變化量（正值表示移動後 cut size 減少）。

當交換頂點 \( a \in A \) 和 \( b \in B \) 時，交換的增益為：

\[
\text{gain}(a, b) = D(a) + D(b) - 2 \times w(a, b)
\]

減去 \( 2 \times w(a, b) \) 的原因是：如果 \( a \) 和 \( b \) 之間有邊相連，這條邊在交換後會從跨分割變為同分割，需要額外扣除一次。

### 演算法流程

K-L 演算法的執行分為多個「pass」，每個 pass 又由多步組成：

```
procedure KL_Partition(G):
    A, B = initial_partition(G)    // 隨機或啟發式初始分割

    repeat:
        // 鎖定所有頂點（未鎖定）
        unlock_all_vertices()
        best_gains = [0]
        swaps = []

        // 嘗試所有可能的交換
        for i = 1 to |V|/2:
            // 找未鎖定的 a ∈ A, b ∈ B 使 gain(a,b) 最大
            (a, b) = find_best_swap(unlocked_A, unlocked_B)
            if no pair found: break

            // 記錄本次交換
            swaps[i] = (a, b)
            best_gains[i] = best_gains[i-1] + gain(a, b)

            // 執行交換（即使增益為負也要交換）
            swap(a, b)
            lock(a)
            lock(b)

            // 更新所有與 a, b 相鄰頂點的 D 值
            update_D_values(neighbors(a) ∪ neighbors(b))

        // 在最佳增益處停止
        k = argmax(best_gains)     // 使累積增益最大的步數
        if best_gains[k] <= 0: break  // 無改善則結束

        // 回退到第 k 步之後的狀態
        undo_swaps_from(k+1 to |V|/2)
        A, B = current_partition

    until no improvement
    return (A, B)
```

### 關鍵特性

1. **爬山策略**：K-L 在單步中可能接受增益為負的交換（短期變差），以獲得更大的長期累積增益。這使它能逃離局部最小值。

2. **鎖定機制**：一旦頂點被交換，就被鎖定在目前的分割中，防止在同一個 pass 中被再次交換。這保證了收斂性。

3. **多 pass 疊代**：一個 pass 中的最佳中間狀態被保留，然後在此狀態上開始新的 pass，直到無法改善。

4. **等大小限制**：K-L 要求 \( |A| = |B| \)（或最多差 1），無法處理非平衡分割問題。這是它最大的侷限。

### 時間複雜度

K-L 演算法在最樸素的實作中：

- 每次尋找最佳交換需要 \( O(|A| \times |B|) = O(n^2) \) 時間
- 每個 pass 有 \( O(n) \) 步交換
- 總複雜度為 \( O(n^3) \) 每個 pass

透過使用優先佇列（priority queue）來維護增益值，可以將復雜度降至 \( O(n^2 \log n) \)。

### 局限

1. **僅支援 2-way 分割**：原始 K-L 只能處理二分。要處理 k-way 分割，需要遞迴應用（每次將一個部分再分半），但這會導致非最優的層級決策。

2. **等大小約束**：無法處理大小不等的分割要求。

3. **圖模型限制**：K-L 處理的是圖（graph），即每條邊只連接兩個頂點。但在電路中，一個連線（net）通常連接多個終端，構成超邊（hyperedge）。直接將超邊展開為多條邊會丟失資訊。

4. **初始分割敏感**：隨機初始分割可能導致收斂到不同的局部最佳解。

## Fiduccia-Mattheyses 演算法

Fiduccia-Mattheyses 演算法由 Charles Fiduccia 與 Robert Mattheyses 於 1982 年提出，論文題為 "A Linear-Time Heuristic for Improving Network Partitions"。F-M 演算法是對 K-L 演算法的重大改進，解決了 K-L 的多個根本性限制。

### 核心改進

F-M 演算法對 K-L 做出了四項關鍵改進：

#### 1. 單位元移動（Single Cell Move）

K-L 每次交換一對頂點（同時移動兩個頂點）；F-M 每次只移動一個頂點。這帶來了以下優勢：

- **更靈活的平衡控制**：因為每次只動一個頂點，可以即時調整分割大小
- **更細粒度的搜尋**：單頂點移動的搜索空間更大，不易錯過最佳中間狀態
- **更容易處理不平衡分割**：可以設定大小上限，移動時不超過即可

#### 2. 桶列資料結構（Bucket List）

這是 F-M 最著名的貢獻。桶列是一種用於實現增益排序的資料結構：

```
增益範圍: [-max_gain, +max_gain]
桶陣列: [bucket[-max_gain], bucket[-max_gain+1], ..., bucket[+max_gain]]

每個桶是一個雙向鏈表，儲存具有該增益值的頂點
```

桶列的運作方式：
- 每個頂點存儲在其對應增益值的桶中
- 需要選取最佳增益頂點時：從最高增益的桶向最低掃描，取第一個非空桶中的任一頂點
- 當頂點增益變化時：從舊桶中移除，插入新桶

這種結構使選擇最佳頂點和更新增益的操作均為 \( O(1) \) 攤銷時間。

#### 3. 超圖直接處理（Hypergraph Support）

F-M 直接處理超圖（hypergraph），無需將超邊展開為圖：

```
對每個超邊（net）e，定義：
  - 若 e 的所有頂點都在同一部分：e 不是 cut
  - 若 e 的頂點分布在多個部分中：e 是 cut
```

超邊的處理方式改變了增益計算，增加了 F-M 在電路分割中的準確性。

#### 4. 更低的時間複雜度

F-M 的每次 pass 時間複雜度為 \( O(|E|) \)（與超邊數量線性相關），相較 K-L 的 \( O(n^2 \log n) \) 有顯著改進，使其能處理更大規模的電路。

### 演算法流程

```
procedure FM_Partition(H):
    A, B = initial_partition(H)

    repeat:
        unlock_all_vertices()
        best_gains = [0]
        moves = []
        gain_buckets = build_buckets(A, B)

        // 平衡約束：允許的最大大小差異
        balance = max(|A|, |B|) - min(|A|, |B|) ≤ max_imbalance

        for i = 1 to |V|:
            // 選取未鎖定且滿足平衡約束的增益最高的頂點
            v = select_best_vertex(gain_buckets, balance)
            if v is None: break

            // 記錄此移動
            moves[i] = v
            best_gains[i] = best_gains[i-1] + gain(v)

            // 執行移動
            move(v, A, B)
            lock(v)

            // 更新增益
            for each net e connected to v:
                for each vertex u in e:
                    if not locked(u):
                        update_gain(u, gain_buckets)
                        update_bucket(u, gain_buckets)

        // 選取並回退到最佳增益位置
        k = argmax(best_gains)
        if best_gains[k] <= 0: break
        undo_moves_from(k+1 to |V|)

    until no improvement
    return (A, B)
```

### 增益計算的細節

F-M 的增益計算是整個演算法最精妙的部分。給定一個超邊（net）和一個要移動的頂點：

定義 \( F(e) \) = net \( e \) 中被移動後會脫離 cut 的頂點數量

對於一個超邊 \( e \)，其對頂點 \( v \) 的移動貢獻的增益計算如下：

```
若 v 從 A 移到 B：
  對 e 中的每個頂點 u（u ≠ v）：
    若 u 仍在 A 中 → e 仍然是 cut（無貢獻）
    若 u 在 B 中 → 移動後可能改變 e 的 cut 狀態
```

具體的 F-M 增益公式：

\[
\text{gain}(v) = \sum_{e \in \text{nets}(v)} F(e, v)
\]

其中：
- 若移動 \( v \) 後，某個超邊 \( e \) 從 cut 變為非 cut，則貢獻 +1（淨改善）
- 若移動 \( v \) 後，某個超邊 \( e \) 從非 cut 變為 cut，則貢獻 -1（淨惡化）
- 否則貢獻 0

更精確的計算是透過追蹤每個超邊的「分佈狀態」：
- 若移動 \( v \) 後，\( e \) 中的所有頂點都在同一側 → 邊 \( e \) 不再是 cut，gain +1（稱為「移除」）
- 若移動 \( v \) 前，\( e \) 中的頂點全在同一側，但移動後分散在兩側 → 邊 \( e \) 成為 cut，gain -1（稱為「新增」）
- 其他情況：gain 0

### 平衡約束處理

F-M 透過簡單的檢查來維護分割平衡：

```
balance_ok(v, A, B):
    if v ∈ A and moving v to B:
        new_A_size = |A| - 1
        new_B_size = |B| + 1
        return |new_A_size - new_B_size| ≤ max_imbalance
    if v ∈ B and moving v to A:
        new_A_size = |A| + 1
        new_B_size = |B| - 1
        return |new_A_size - new_B_size| ≤ max_imbalance
```

在選擇要移動的頂點時，只考慮那些移動後不會破壞平衡約束的頂點。這通常在桶列掃描中整合：從最高增益桶開始掃描，找到第一個通過平衡檢查的頂點。

### 與 K-L 的比較

| 特性 | K-L | F-M |
|------|-----|-----|
| **移動單位** | 交換一對頂點 | 單個頂點 |
| **平衡控制** | 等大小嚴格 | 可配置寬鬆度 |
| **圖形模型** | 圖（graph） | 超圖（hypergraph） |
| **資料結構** | 無特殊結構 | 桶列（bucket list） |
| **每 pass 複雜度** | O(n² log n) | O(\|E\|) |
| **最優頂點選取** | O(n²) 搜尋 | O(1) 桶查找 |
| **增益更新** | 重新計算 D(v) | O(degree(v)) 局部更新 |
| **應用限制** | 2-way 等分 | 可擴展至 k-way |

## 超圖與圖的區別

理解超圖（hypergraph）與圖（graph）的區別對於掌握電路分割至關重要。

### 圖（Graph）

在圖中，每條邊（edge）恰好連接兩個頂點：
- \( e = (u, v) \)
- 適用於社交網路、道路網路等

### 超圖（Hypergraph）

在超圖中，每條超邊（hyperedge）可以連接任意數量的頂點：
- \( e = \{v_1, v_2, ..., v_k\} \)
- 非常適合表示電路中的連線（net）：一個 net 連接一個驅動器和多個接收器

### 為什麼超圖很重要

在電路中，一個連線（net）通常連接多個終端。例如，一個時脈訊號可能驅動數百個正反器。如果使用圖來表示：
- 需要將一個超邊展開為多個邊：\( (v_1, v_2), (v_1, v_3), ..., (v_1, v_k) \)
- 這會建立 k-1 條邊，而不是一條超邊
- 在分割時，這些邊被賦予 1/(k-1) 的權重，試圖模擬原始超邊的行為

但這種展開在切割計算上並不精確。超邊的切割判斷是二元的：
- 若所有頂點在同一部分：未切割
- 若頂點分佈在多部分：已切割

使用圖展開時，一個部分切割的超邊可能會被計為部分切割（取決於哪幾對頂點被分開），這會引入不準確性。

F-M 直接對超圖操作，正確地處理了這種「全部或無」的切割邏輯，因此對電路分割更有效。

## 應用於 FPGA 設計

### FPGA 封裝（Packing）

在 FPGA 設計流程中，分割演算法被應用於「封裝」步驟：

1. **輸入**：技術映射後的網表（LUT + FF 為基本單元）
2. **目標**：將 LUT 和 FF 分組到 CLB 的 slice 中
3. **約束**：
   - 每個 slice 有固定數量的 LUT 和 FF（例如 iCE40 每個 tile 一個 LUT + 一個 FF）
   - 同一個 slice 中的元件可以共用 carry chain、clock enable、set/reset
   - LUT 的輸入埠數量有限（4-LUT 或 5-LUT）

4. **分割策略**：
   - 將 LUT 和 FF 視為頂點
   - 將 nets 視為超邊
   - 使用 K-L 或 F-M 進行聚類（clustering）

### eda4 的 v2f-pnr 與分割

在 eda4 專案的 v2f-pnr 中，目前**沒有**獨立的封裝（packing）步驟：

- 預設假設：每個邏輯單元直接映射到一個 iCE40 邏輯磚塊（logic tile）
- iCE40 的每個 logic tile 包含一個 LUT4 + 一個正反器 + 進位邏輯
- 對於簡單設計（如 blinky、adder），這種一對一映射是足夠的
- 對於更複雜的設計，可能需要封裝多個基本邏輯單元到一個 tile 中

未來如果要加入封裝步驟，F-M 演算法將是首選的候選方案，因為：
- 它直接處理超圖（適合電路網表）
- 線性時間複雜度（適合大規模設計）
- 靈活的平衡約束（可配置每個 tile 的利用率）

### 與大型 FPGA 的對比

在 Xilinx 或 Intel 的大型 FPGA 架構中，分割與封裝的重要性更為突出：

- **Xilinx 7 系列**：每個 CLB 包含 2 個 slice，每個 slice 包含 4 個 LUT + 8 個 FF
- **Intel Stratix 10**：每個 ALM 包含 8 個 LUT + 16 個 FF

這意味著對於這些架構，必須將大量的 LUT 與 FF 封裝到有限數量的 CLB/ALM 中——這本質上就是一個大規模的分割問題。

## 多層次分割

處理超大規模電路時，直接應用 K-L 或 F-M 仍可能太慢。為此，學術界提出了多層次分割（Multilevel Partitioning）方法。

### 核心思想

多層次分割包含三個階段：

```
1. 粗化（Coarsening）：
   將相鄰的頂點合併，建立較小的「粗化圖」
   重複此步驟，直到圖足夠小

2. 初始分割（Initial Partitioning）：
   在最粗的圖上進行分割（使用 K-L 或 F-M）
   由於頂點數量很少，此步驟很快

3. 反粗化（Uncoarsening / Refinement）：
   逐步將粗化圖展開回原圖
   在每個層級對分割進行細化（refinement）
   細化同樣使用 K-L 或 F-M
```

### hMetis

hMetis 是最著名的多層次超圖分割工具，由 University of Minnesota 的 George Karypis 等人開發：

- 專門為超圖設計（支援電路網表）
- 使用多層次框架
- 頂點合併策略：基於超邊連接的匹配演算法
- 細化階段：使用 F-M 演算法
- 可處理數百萬頂點的超圖
- 在分割品質和執行時間上均優於直接 F-M

### MLpart

MLpart 是 VTR（Verilog to Routing）工具鏈中的多層次分割工具：
- 基於 hMetis 的思想
- 與 VPR 的封裝步驟整合
- 支援 FPGA 特定的約束（如 carry chain 約束）

## 其他分割演算法

### 頻譜分割（Spectral Partitioning）

頻譜方法基於圖拉普拉斯矩陣的特徵向量來進行分割：

1. 建構圖的拉普拉斯矩陣 \( L = D - A \)
2. 計算第二小特徵值對應的特徵向量（Fiedler vector）
3. 根據特徵向量的符號或閾值進行分割

優點：
- 有堅實的數學基礎（可證明是最佳化問題的連續鬆弛解）
- 可產生高品質的分割結果

缺點：
- 不支援超圖
- 計算特徵分解的代價高（O(n³)）
- 需要額外步驟將連續解離散化

### 隨機最佳化分割

使用遺傳演算法、模擬退火等通用最佳化方法進行分割：

優點：
- 靈活：可整合任意約束條件
- 易於實作

缺點：
- 收斂慢
- 缺乏針對分割問題的專用最佳化

### 流導向分割（Flow-Based Partitioning）

基於最大流最小割定理（Max-Flow Min-Cut Theorem）：

1. 在圖上執行最大流計算
2. 最小割對應於最小化切割的最佳分割
3. 適用於 small 2-way 分割

優點：
- 可找到精確最佳解（對 2-way 分割）
- 理論基礎完善

缺點：
- 擴展性差無法處理大規模電路
- 不支援大小平衡約束
- 不支援 k-way 分割

## 如何選擇分割演算法

在實際專案中選擇分割演算法時需要考慮以下因素：

### 設計規模

- **小型設計（< 1000 單元）**：K-L 或 F-M 均足夠
- **中型設計（1000 ~ 100000 單元）**：F-M 是首選
- **大型設計（> 100000 單元）**：多層次分割（hMetis / MLpart）

### 分割類型

- **2-way 等分**：K-L 或 F-M 均可
- **2-way 不等分**：F-M（內建支援）
- **k-way 分割**：F-M 的遞迴組合或 hMetis

### 架構約束

- **簡單的容量限制**：F-M 可滿足
- **複雜的資源共用**：需要自訂增益函數
- **階層式架構**：多層次分割

### eda4 的建議

對於 eda4 專案的 v2f-pnr，如果未來引入封裝步驟：

1. 從 F-M 演算法開始實作（約 200-300 行程式碼）
2. 使用桶列資料結構管理增益
3. 支援超圖輸入（直接從 `PnrNetlist` 轉換）
4. 平衡約束設定為每個 tile 的 LUT/FF 容量
5. 若未來要處理更大設計，考慮引入多層次框架

這樣的實作可以在不顯著增加複雜度的前提下，顯著提升 v2f-pnr 對大型 FPGA 設計的處理能力。

## 參考文獻

- Kernighan, B. W., & Lin, S. (1970). An Efficient Heuristic Procedure for Partitioning Graphs. The Bell System Technical Journal, 49(2), 291–307.
- Fiduccia, C. M., & Mattheyses, R. M. (1982). A Linear-Time Heuristic for Improving Network Partitions. DAC '82, 175–181.
- Karypis, G., & Kumar, V. (1998). A Fast and High Quality Multilevel Scheme for Partitioning Irregular Graphs. SIAM Journal on Scientific Computing, 20(1), 359–392.
- Alpert, C. J., & Kahng, A. B. (1995). Recent Directions in Netlist Partitioning: A Survey. Integration, 19(1-2), 1–81.
- Hagen, L., & Kahng, A. B. (1992). New Spectral Methods for Ratio Cut Partitioning and Clustering. IEEE TCAD, 11(9), 1074–1085.
- [Place & Route (PNR) 佈局與繞線](pnr.md) — 分割結果的下一步：佈局與繞線
- [模擬退火](simulated_annealing.md) — 另一種佈局最佳化方法，可供對比
- [iCE40 架構](ice40.md) — 理解 FPGA 架構對分割策略的影響
- [網表 / Yosys-JSON](netlist.md) — 分割操作的輸入輸出格式
