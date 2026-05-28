# 電路分割：Kernighan-Lin 與 Fiduccia-Mattheyses 演算法

## 什麼是電路分割

電路分割（Circuit Partitioning）是 EDA 中最基礎也最核心的問題之一。給定一個由邏輯閘（cells）和連線（nets）組成的電路，分割的目標是將電路劃分為多個部分，使得跨部分的連線數量（cut size）最小化，同時各部分的大小保持平衡。

形式化地，給定超圖 \( H(V, E) \)，將 \( V \) 劃分為 \( k \) 個子集 \( P_1, ..., P_k \)，使得：
- \( \bigcup P_i = V \) 且 \( P_i \cap P_j = \emptyset \)（不重疊且完全覆蓋）
- 各子集大小均衡（\( |P_i| \) 接近一致）
- \( \sum w(e) \times \text{cut}(e) \) 最小化，其中 cut(e) = 1 若 e 的頂點分佈於多個部分

電路分割是 NP-hard 問題，實務上依賴啟發式演算法。Kernighan-Lin（K-L, 1970）和 Fiduccia-Mattheyses（F-M, 1982）是其中最具影響力的兩個演算法。K-L 奠定了增益驅動疊代改善的基礎框架，F-M 則透過桶列資料結構將時間複雜度降至線性，並支援了超圖直接處理。

## 分割在 EDA 中的重要性

- **分而治之**：將大型設計分割為子電路，獨立最佳化後合併，大幅降低計算複雜度。這使平行處理成為可能，對現代數十億電晶體的晶片設計至關重要。
- **FPGA 多晶片分割**：當設計太大而無法裝入單一 FPGA 時，需考慮各晶片容量和 I/O 腳位限制（通常很有限）。
- **階層式佈局（Floorplanning）**：將相關邏輯群組在一起，減少高頻寬區塊間的通訊距離，優化晶片熱分佈。
- **FPGA 封裝（Packing）**：將 LUT 和 FF 分組到 CLB 的 slice 中，考慮進位鏈、時脈使能等資源共用。這本質上就是一個分割問題。
- **時脈域分組**：將同域邏輯集中放置以簡化時脈樹綜合，減少跨時脈域走線，降低時脈偏差。

## Kernighan-Lin 演算法

K-L 演算法由 Brian Kernighan 與 Shen Lin 於 1970 年發表於 The Bell System Technical Journal，題為 "An Efficient Heuristic Procedure for Partitioning Graphs"。這是電路分割領域最具開創性的工作之一。

### 問題模型

K-L 處理由圖 \( G(V, E) \) 描述的電路，目標是將 \( V \) 劃分為兩個大小相等（或近乎相等）的子集 \( A \) 和 \( B \)，最小化連接兩者的邊權重總和。每條邊 \( (u, v) \) 有權重 \( w(u, v) \)，未加權時為 1。

### 增益計算

對每個頂點 \( v \)，定義：
- \( I(v) \) = 與 \( v \) 同側的邊權重總和（internal cost）
- \( E(v) \) = 與 \( v \) 不同側的邊權重總和（external cost）
- \( D(v) = E(v) - I(v) \)（正值表示移動後 cut size 減少）

交換頂點 \( a \in A \) 和 \( b \in B \) 的增益為：

\[
\text{gain}(a, b) = D(a) + D(b) - 2 \times w(a, b)
\]

減去 \( 2 \times w(a, b) \) 的原因是：若 \( a \) 和 \( b \) 之間有邊相連，這條邊在交換後會從跨分割變為同分割，需要額外扣除一次。

### 演算法流程

K-L 的執行分為多個 pass，每個 pass 由多步組成：

```
procedure KL_Partition(G):
    A, B = initial_partition(G)
    repeat:
        unlock_all_vertices()
        for i = 1 to |V|/2:
            (a, b) = argmax gain(unlocked_A, unlocked_B)
            if no pair found: break
            swaps[i] = (a, b)
            best_gains[i] = best_gains[i-1] + gain(a, b)
            swap(a, b); lock(a); lock(b)
            update_D_values(neighbors(a) ∪ neighbors(b))
        k = argmax(best_gains)
        if best_gains[k] <= 0: break
        undo_swaps from k+1 to |V|/2
    until no improvement
    return (A, B)
```

### 關鍵特性

- **爬山策略**：單步可能接受增益為負的交換（短期變差），以獲得更大的長期累積增益。這使它能逃離局部最小值。
- **鎖定機制**：頂點一旦被交換就在該 pass 中被鎖定，防止同一頂點被反覆移動，保證收斂。
- **多 pass 疊代**：一個 pass 中記錄最佳中間狀態，保留該狀態後開始新的 pass，直到無法再改善。
- **等大小限制**：要求 \( |A| = |B| \)（或最多差 1），無法處理不平衡分割——這是它最大的實務限制。

### 時間複雜度

樸素實作 \( O(n^3) \) 每 pass（每次找最佳交換 \( O(n^2) \)，共 \( O(n) \) 步）。使用優先佇列維護增益值可降至 \( O(n^2 \log n) \)。

### 局限

1. 僅支援 2-way 分割：需遞迴處理 k-way，但層級決策不可逆，早期錯誤會傳播。
2. 等大小約束：無法處理大小不等的分割要求。
3. 圖模型限制：電路中的連線（net）通常連接多個終端（超邊），直接將超邊展開為多條邊會丟失資訊且切割計算不準確。
4. 初始分割敏感：不同的初始分割可能導致收斂到不同的局部最佳解。

## Fiduccia-Mattheyses 演算法

F-M 演算法由 Charles Fiduccia 與 Robert Mattheyses 於 1982 年提出，論文題為 "A Linear-Time Heuristic for Improving Network Partitions"。這是對 K-L 的重大改進，解決了 K-L 的多個根本性限制，並成為業界標準。

### 核心改進

F-M 對 K-L 做出了四項關鍵改進：

#### 1. 單位元移動（Single Cell Move）

F-M 每次只移動一個頂點，而非交換一對：
- **更靈活的平衡控制**：可即時調整分割大小，自然支援不平衡分割
- **更細粒度的搜尋**：單頂點移動的搜索空間更大，不易錯過最佳中間狀態

#### 2. 桶列資料結構（Bucket List）

這是 F-M 最著名的貢獻。桶列是一組按增益值索引的雙向鏈表：

```
增益範圍: [-max_gain, +max_gain]
桶陣列: [bucket[-max_gain], bucket[-max_gain+1], ..., bucket[+max_gain]]
```

每個頂點存儲在其對應增益值的桶中。選取最佳頂點時，從最高增益桶向下掃描，取第一個非空桶中的任一頂點——時間複雜度 O(1)。當頂點增益發生變化時，從舊桶中移除並插入新桶——同樣 O(1) 攤銷時間。

#### 3. 超圖直接處理（Hypergraph Support）

F-M 直接處理超圖（hypergraph），無需將超邊展開為圖。超邊的切割是二元的：所有頂點在同一側則不切割，分佈在多側則切割。這種「全有或全無」的邏輯比圖展開更精確——圖展開會將部分切割的超邊計為部分權重，造成誤差。

增益計算基於超邊中頂點的分佈狀態：
- 若移動 \( v \) 後某超邊從 cut 變為非 cut：gain +1（稱為「移除」）
- 若移動 \( v \) 後某超邊從非 cut 變為 cut：gain -1（稱為「新增」）
- 其他情況：gain 0

#### 4. 平衡約束處理

在選擇要移動的頂點時，只考慮那些移動後不會破壞平衡約束的頂點。這在桶列掃描中從最高增益向低搜尋，找到第一個通過平衡檢查的頂點：

```
balance_ok(v, A, B):
    計算移動後新的大小
    檢查 |new_size_A - new_size_B| ≤ max_imbalance
```

### 演算法流程

```
procedure FM_Partition(H):
    A, B = initial_partition(H)
    repeat:
        unlock_all_vertices()
        gain_buckets = build_buckets(A, B)
        for i = 1 to |V|:
            v = select_best_vertex(gain_buckets, balance)
            if v is None: break
            moves[i] = v
            best_gains[i] = best_gains[i-1] + gain(v)
            move(v, A, B); lock(v)
            for each net e connected to v:
                for each vertex u in e:
                    if not locked(u):
                        update_gain(u, gain_buckets)
        k = argmax(best_gains)
        if best_gains[k] <= 0: break
        undo_moves_from(k+1 to |V|)
    until no improvement
    return (A, B)
```

### 時間複雜度

每次 pass \( O(|E|) \)（與超邊數量線性相關），遠優於 K-L 的 \( O(n^2 \log n) \)。這使 F-M 能處理遠大於 K-L 的電路規模。

### K-L 與 F-M 的比較

| 特性 | K-L | F-M |
|------|-----|-----|
| **移動單位** | 交換一對頂點 | 單個頂點 |
| **平衡控制** | 等大小嚴格 | 可配置寬鬆度 |
| **圖形模型** | 圖（graph） | 超圖（hypergraph） |
| **資料結構** | 無特殊結構 | 桶列（bucket list） |
| **每 pass 複雜度** | O(n² log n) | O(\|E\|) |
| **頂點選取** | O(n²) 搜尋 | O(1) 桶查找 |
| **增益更新** | 重新計算 D(v) | O(degree(v)) 局部更新 |
| **擴展性** | 2-way 強制等分 | 可擴展至 k-way |
| **電路適用性** | 有限（圖非超圖） | 業界標準 |

## 超圖與圖的區別

理解超圖（hypergraph）與圖（graph）的區別至關重要。在電路中，一個連線（net）通常連接到多個終端，構成超邊（hyperedge）。例如，一個時脈訊號可能驅動數百個正反器，形成一個連接大量頂點的超邊。

如果用圖表示超邊，需要將其展開為 \( (v_1, v_2), (v_1, v_3), ..., (v_1, v_k) \) 共 k-1 條邊，每條邊分配權重 1/(k-1)。這種展開在切割計算上不精確——超邊的切割是二元的（要嘛全在一側，要嘛有被切到），但圖展開會將部分切割的超邊計為部分權重（如只切到一半的邊時只計一半權重）。F-M 直接對超圖操作，正確處理了這種「全有或全無」的切割邏輯。

## 應用於 FPGA 設計

### FPGA 封裝（Packing）

在 FPGA 設計流程中，分割演算法被應用於封裝步驟：
- 輸入：技術映射後的網表（LUT + FF 為基本單元）
- 目標：將 LUT 和 FF 分組到 CLB 的 slice 中
- 約束：每個 slice 有固定的 LUT/FF 數量、輸入埠數量（如 4-LUT）、可共用的特殊資源（carry chain、clock enable）

策略上，將 LUT/FF 視為頂點，nets 視為超邊，使用 K-L 或 F-M 進行聚類。

### eda4 的 v2f-pnr

v2f-pnr 目前**沒有**獨立的封裝步驟——每個邏輯單元直接映射到一個 iCE40 邏輯磚塊。iCE40 每個 logic tile 包含一個 LUT4 + 正反器 + 進位邏輯。對簡單設計（blinky、adder）這種一對一映射足夠，但更複雜的設計可能需要封裝多個基本單元到一個 tile 中。

若未來引入封裝步驟，F-M 是首選：
- 直接支援超圖（與電路網表格式契合）
- 線性時間複雜度（適合大規模設計）
- 靈活的平衡約束（可配置每個 tile 的利用率）

### 與大型 FPGA 的對比

- Xilinx 7 系列：每個 CLB 含 2 個 slice，每個 slice 含 4 個 LUT + 8 個 FF
- Intel Stratix 10：每個 ALM 含 8 個 LUT + 16 個 FF

這些架構必須將大量 LUT/FF 封裝到有限的 CLB/ALM 中，本質上就是大規模分割問題。

## 多層次分割

處理超大規模電路時，直接 F-M 仍可能太慢。多層次分割包含三個階段：
1. **粗化**：合併相鄰頂點建立粗化圖（基於超邊連接的匹配），重複直到圖足夠小
2. **初始分割**：在最粗圖上使用 F-M 進行快速分割
3. **反粗化/細化**：逐步展開，在每個層級使用 F-M 細化

### hMetis

hMetis（George Karypis 等人）是最著名的多層次超圖分割工具：
- 專為超圖設計，使用基於超邊連接的匹配演算法
- 細化階段使用 F-M
- 可處理數百萬頂點，分割品質和執行速度均優於直接 F-M

### MLpart

VTR 工具鏈中的多層次分割工具，基於 hMetis 思想，與 VPR 封裝步驟整合，支援 carry chain 等 FPGA 特定約束。

## 其他分割演算法

- **頻譜分割**：基於圖拉普拉斯矩陣的第二小特徵向量（Fiedler vector）。數學基礎堅實（可證明是連續鬆弛的最優解），但 O(n³) 的特徵分解代價高，且不支援超圖。需要額外步驟將連續解離散化為合法的分割方案。
- **流導向分割**：基於最大流最小割定理，對 2-way 可求精確最佳解。使用 Ford-Fulkerson 或 Push-Relabel 演算法計算最大流，最小割對應於最佳分割。但擴展性差，不支援大小平衡和 k-way 分割。
- **隨機最佳化**：遺傳演算法或模擬退火，靈活可整合任意約束，但收斂慢且缺乏針對分割問題的專用最佳化。適用於需要處理特殊約束（如非均勻面積、溫度限制）的場景。

## F-M 實現要點

在實作 F-M 演算法時需要留意以下細節：

**桶列的實現**：桶列的核心是固定大小的陣列，每個元素指向一個雙向鏈表的頭節點。陣列大小為 \( 2 × max\_gain + 1 \)，其中 \( max\_gain \) 是單一頂點移動能達到的最大增益值。對於電路分割，\( max\_gain \) 通常等於連接該頂點的最大超邊數量。

**增益的初始計算**：在第一次迭代前，需要計算所有頂點的初始增益。這需要遍歷所有超邊，對每個超邊中的頂點計算其移動對 cut 狀態的影響。初始化的時間複雜度為 \( O(|E|) \)。

**增益的增量更新**：當移動一個頂點 \( v \) 後，只需要更新與 \( v \) 有共同超邊的頂點的增益值。這被稱為「局部增益更新」，是 F-M 達到線性時間複雜度的關鍵。

**鎖定與解鎖**：每個 pass 中，被移動的頂點被鎖定（lock），防止在同一 pass 中被再次移動。一個 pass 完成後，所有頂點被解鎖（unlock）以進行下一 pass。鎖定機制保證了收斂性。

**平衡約束的鬆緊度**：\( max\_imbalance \) 設為 0 時強制等大小分割（類似 K-L），設為較大值時允許較大的不平衡。典型的設定是允許 5% 至 10% 的大小差異。

**多輪起始（Multi-start）**：由於 F-M 對初始分割敏感，常見做法是從多個隨機初始分割出發，分別執行 F-M，最後選取其中最好的結果。這種方法可以顯著提升分割品質。

## 分割演算法的評估指標

在實際應用中，評估分割品質通常使用以下指標：

**Cut Size（切割大小）**：跨越多個分割區域的連線數量或總權重。這是最基礎的評估指標，也是 K-L 和 F-M 直接最佳化的目標。對於加權超圖，cut size 是所有被切割超邊的權重和。

**割線比（Cut Ratio）**：cut size 除以總連線數，用於消除設計規模對評估的影響。同一個演算法在不同規模設計上的表現可以透過此指標公平比較。

**執行時間（Runtime）**：對於給定設計，分割演算法所需的執行時間。F-M 的線性時間複雜度使其在此指標上遠優於 K-L。對於包含數十萬頂點的設計，F-M 可以在數秒內完成一個 pass。

**伸縮性（Scalability）**：演算法在設計規模增加時的效能退化程度。多層次分割的伸縮性最好，可以處理數百萬頂點的超大型電路。

**穩定性（Stability）**：同一演算法在不同初始條件下產生相似結果的能力。K-L 和 F-M 對初始分割敏感，可透過多次隨機起始（multi-start）選取最佳結果來改善。典型做法是執行 10 到 50 次隨機起始後取最佳解。

**記憶體使用（Memory）**：演算法在運行過程中佔用的記憶體量。K-L 需要儲存 D(v) 和增益表；F-M 需要儲存桶列和超邊結構；多層次分割需要儲存多層次的圖表示。

## 分割與封裝在 FPGA 流程中的整合

在標準的 FPGA EDA 流程中，分割（partitioning）和封裝（packing）通常位於技術映射（technology mapping）之後、佈局（placement）之前。流程如下：

1. **技術映射**：將 HDL 綜合後的邏輯閘網表轉換為 LUT 和 FF 組成的網表
2. **封裝/聚類**：將多個 LUT 和 FF 分組到一個 CLB 或 slice 中（這一步就是分割的應用）
3. **佈局**：將 CLB 分配到 FPGA 的實體位置
4. **繞線**：在 CLB 之間建立連接

在 eda4 的 v2f-pnr 中，由於 iCE40 的每個 logic tile 只包含一個 LUT+FF，封裝步驟被省略了。但對於更複雜的設計或目標架構（如 Xilinx 的 CLB 含 4 個 LUT），封裝是不可或缺的步驟。

## 延伸應用：超大型電路分割策略

對於現代超大規模電路（數百萬邏輯閘），單純的 F-M 即使搭配多層次框架也可能不夠高效。實務上常用的策略包括：

**遞迴二分法**：重複將電路一分為二，直到每個子電路小於閾值。這種方法簡單且高效，但層級決策不可逆——早期的分割決策如果是次優的，後續步驟無法修正。

**平行分割**：將電路劃分為多個區塊，在不同的處理器或機器上平行執行分割。hMetis 支援平行版本（ParMetis），可以在數十個節點上平行處理。

**增量分割**：在設計迭代過程中，只對發生變化的部分進行重新分割，保留大部分既有的分割決策。這對於工程變更（ECO）場景特別有用。

**混合分割**：使用多種分割演算法的組合，例如先用頻譜分割取得全域方向，再用 F-M 進行局部細化。

## eda4 的建議

若為 v2f-pnr 引入封裝步驟，建議：
1. 從 F-M 演算法開始（約 200-300 行程式碼）
2. 使用桶列管理增益值
3. 支援超圖輸入（直接從 `PnrNetlist` 轉換為超圖表示）
4. 平衡約束設為每個 tile 的 LUT/FF 容量
5. 若處理更大設計，引入多層次框架（粗化 + F-M 細化）

## 參考文獻

- Kernighan, B. W., & Lin, S. (1970). An Efficient Heuristic Procedure for Partitioning Graphs. The Bell System Technical Journal, 49(2), 291–307. 原始 K-L 論文，奠定了增益驅動疊代改善的基礎框架。
- Fiduccia, C. M., & Mattheyses, R. M. (1982). A Linear-Time Heuristic for Improving Network Partitions. DAC '82, 175–181. 原始 F-M 論文，引入了桶列資料結構和超圖支援。
- Karypis, G., & Kumar, V. (1998). A Fast and High Quality Multilevel Scheme for Partitioning Irregular Graphs. SIAM Journal on Scientific Computing, 20(1), 359–392. hMetis 的論文，多層次分割的里程碑。
- Alpert, C. J., & Kahng, A. B. (1995). Recent Directions in Netlist Partitioning: A Survey. Integration, 19(1-2), 1–81. 全面的分割演算法回顧，適合入門參考。
- [Place & Route (PNR) 佈局與繞線](pnr.md) — 分割結果的下一步：佈局與繞線
- [模擬退火](simulated_annealing.md) — 另一種佈局最佳化方法
- [iCE40 架構](ice40.md) — 理解 FPGA 架構對分割策略的影響
- [網表 / Yosys-JSON](netlist.md) — 分割操作的輸入輸出格式

## 延伸閱讀

- [Graph Partition (Wikipedia)](https://en.wikipedia.org/wiki/Graph_partition)
- [Kernighan–Lin Algorithm (Wikipedia)](https://en.wikipedia.org/wiki/Kernighan%E2%80%93Lin_algorithm)
- [Hypergraph (Wikipedia)](https://en.wikipedia.org/wiki/Hypergraph)
