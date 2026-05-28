# A* 演算法於 FPGA 繞線路徑搜尋

## 什麼是 A* 演算法

A* (A-Star) 是一種基於最佳優先搜尋的圖形路徑搜尋演算法，廣泛應用於路徑規劃、地圖導航、遊戲 AI 和 EDA 工具中的繞線問題。其核心思想是結合 Dijkstra 演算法的精確性和貪婪最佳優先搜尋的效率，透過一個評價函數來引導搜尋方向。

在 eda4 專案的 v2f-pnr crate 中，A* 被應用於 FPGA 的繞線階段——即在完成邏輯單元佈局後，為每個連線網（net）尋找從驅動端到所有負載端的實際繞線路徑。FPGA 的繞線通道構成一個圖形結構，其中每個 tile 對應一個節點，可程式繞線資源構成節點之間的邊。

A* 的關鍵優勢在於其啟發式搜尋能大幅減少搜尋空間，相比盲目的迷宮繞線（Lee's algorithm）速度快上數倍，且透過恰當的啟發函數設計可保證找到最短路徑。

## 歷史背景

A* 演算法由 Peter Hart、Nils Nilsson 和 Bertram Raphael 於 1968 年提出，論文發表於 IEEE Transactions on Systems Science and Cybernetics，題為 "A Formal Basis for the Heuristic Determination of Minimum Cost Paths"。三位作者均任職於史丹佛研究所（Stanford Research Institute, SRI），當時正在研究 Shakey 機器人專案的路徑規劃問題。

Hart、Nilsson 和 Raphael 的貢獻在於將啟發式搜尋置於嚴格的數學基礎上。他們定義了可容許性（admissibility）和單調性（consistency）的條件，證明採用滿足這些條件的啟發函數保證找到最優路徑。

A* 的命名源自於符號 A*——在早期的論文中，演算法被命名為 "A"（Algorithm A），而 "A*" 代表一種可容許的、最佳的 Algorithm A 變體。星號表示該演算法在給定的啟發函數下是最佳的。

A* 演算法很快成為人工智慧領域的核心演算法之一，並在 1980 年代開始被應用於 VLSI 和 PCB 繞線。與此同時，基於迷宮搜尋的 Lee 演算法（1961 年）和 Hadlock 演算法（1977 年）也在繞線領域佔有一席之地。A* 憑藉其啟發式加速和理論保證，在現代 EDA 工具中獲得了廣泛應用。

## 核心公式

A* 演算法的核心在於評價函數：

```
f(n) = g(n) + h(n)
```

其中：
- **n**：搜尋樹中的節點（對應 FPGA tile）
- **f(n)**：從起點經過 n 到終點的預估總成本
- **g(n)**：從起點到 n 的實際累積成本
- **h(n)**：從 n 到終點的啟發式估計成本

此公式的直觀意義是：當演算法決定從 open set 中選取下一個要探索的節點時，它選擇 f 值最小的節點，而非像 Dijkstra 那樣只考慮 g。這使得 A* 在探索過程中帶有「方向感」，傾向於朝終點方向前進。

### g(n) 的計算

g(n) 從起點開始累積。若從起點 s 到節點 n 經過路徑 s → n1 → n2 → ... → n，則：

```
g(n) = cost(s, n1) + cost(n1, n2) + ... + cost(nk, n)
```

在 FPGA 繞線中，cost 通常包含線段長度、資源使用率和擁塞懲罰。v2f-pnr 中每個 tile 間的連線成本為基礎步長 1，加上可調整的擁塞加權。

### h(n) 的設計

啟發函數 h(n) 的選擇決定了 A* 的行為特徵：
- 若 h(n) = 0：A* 退化為 Dijkstra 演算法，保證最短路徑但搜尋範圍廣
- 若 h(n) 非常大（遠大於實際成本）：A* 退化為貪婪最佳優先搜尋，速度快但不保證最優
- 若 h(n) 接近但不超過實際成本：A* 在效率和最優性間取得良好平衡

## 可容許性與單調性

### 可容許性 (Admissibility)

一個啟發函數 h(n) 被稱為可容許的，若且唯若對所有節點 n，h(n) 小於或等於從 n 到終點的真實最小成本：

```
h(n) ≤ h*(n)   for all n
```

其中 h*(n) 為從 n 到終點的真實最短路徑成本。

當 h 可容許時，A* 保證找到最短路徑。這是 A* 最重要的理論性質，使其區別於一般的啟發式搜索。證明思路為：若目標節點被選中進行擴展時，open set 中所有節點的 f 值都不超過最短路徑成本，則首次到達目標時必為最優路徑。

### 單調性 (Consistency / Monotonicity)

一個啟發函數被稱為單調的（或一致的），若對任意邊 (n, n') 滿足三角不等式：

```
h(n) ≤ cost(n, n') + h(n')
```

直觀上，這表示從 n 到終點的估計成本，不大於先到 n' 再到終點的估計成本。單調性比可容許性更強——如果 h 是單調的，則它必定是可容許的（當 n' 是終點時即得證）。

單調性的重要性在於：
1. 保證 A* 在第一次訪問節點時就找到到該節點的最短路徑
2. 無需重新檢查已關閉的節點，簡化實作
3. 保證 f 值沿路徑非遞減

曼哈頓距離滿足單調性，因此 v2f-pnr 的 A* 實作不需要重開關閉節點。

## 為何 FPGA 繞線使用 A*

FPGA 繞線本質上是在圖形中的路徑搜尋問題：

- FPGA 晶片由規則的二維 tiles 網格構成
- 每個 tile 包含邏輯單元和局部繞線資源
- 繞線通道（routing channel）連接相鄰 tile，構成圖形的邊
- 每個連線網需要連接一個驅動端（driver）到一個或多個負載端（load）

將此抽象為圖形後，尋找兩點間的繞線路徑即是標準的最短路徑問題。A* 在此場景中具有以下優勢：

1. **搜尋效率高**：啟發式引導使搜尋集中於起點和終點之間的區域，而非像 Dijkstra 那樣向外均勻擴散
2. **可整合擁塞資訊**：將 tile 使用次數納入成本函數，可引導繞線避開擁塞區域
3. **保證最短路徑**：使用可容許啟發函數（如曼哈頓距離）時，保證找到的繞線路徑最短
4. **增量式更新**：可輕鬆整合擁塞地圖的動態資訊，支援迭代改進
5. **實作簡潔**：核心演算法只需優先佇列和雜湊集合

### 曼哈頓距離啟發函數

在 FPGA tile 的二維網格中，最自然的啟發函數是曼哈頓距離：

```
h(n) = |xn - xgoal| + |yn - ygoal|
```

曼哈頓距離滿足可容許性——在只能上下左右移動的網格中，它恰好等於最短路徑的下界（忽略障礙物和繞線資源限制時），因此保證不超過實際成本。

曼哈頓距離也滿足單調性：對任何相鄰 tile n 和 n'，曼哈頓距離的變化量等於 1（即移動一步），而 cost(n, n') 也至少為 1（在無負權重時），因此三角不等式成立。

### 其他啟發函數選擇

在某些情況下，以下啟發函數也值得考慮：
- **歐幾里得距離**：√(Δx² + Δy²)，可容許但低估程度較小，不過計算涉及平方根，效率較低
- **對角線距離**：max(|Δx|, |Δy|) + (√2 - 1) × min(|Δx|, |Δy|)，允許對角移動時使用
- **八方向距離**：在可走斜線的網格中使用

對於 FPGA 網格（僅水平垂直移動），曼哈頓距離是最合適的選擇。

## v2f-pnr 實作細節

在 eda4 專案的 v2f-pnr crate 中，A* 實作位於 `v2f-pnr/src/router/astar_router.rs`。以下是其核心資料結構與演算法設計：

### 資料結構

#### AStarNode
每個搜尋節點封裝以下資訊：

```rust
struct AStarNode {
    coord: TileCoord,   // tile 座標 (x, y)
    g: f64,             // 從起點到當前節點的實際成本
    f: f64,             // f = g + h，估計總成本
}
```

PriorityQueue 根據 f 值排序，f 最小的節點優先被擴展。f 值相同時可依 g 值或插入順序二次排序。

#### TileCoord
FPGA tile 的二維座標：

```rust
struct TileCoord {
    x: usize,
    y: usize,
}
```

在 iCE40 架構中，tile 陣列的大小取決於具體型號：
- HX1K: 12×16
- HX4K: 20×32
- HX8K: 26×38
- LP1K: 12×16
- UP5K: 16×24

#### RoutingPath

```rust
struct Routing {
    paths: Vec<Vec<TileCoord>>,  // 所有 nets 的繞線路徑
}
```

paths 的每個元素對應一個 net 的路徑，路徑本身是 TileCoord 的序列，從驅動端 tile 一路連接到負載端 tile。

### 演算法流程

```
function astar_route(start, goal, congestion_map):
    open_set = PriorityQueue<AStarNode>   // 按 f 排序
    came_from = HashMap<TileCoord, TileCoord>
    g_score = HashMap<TileCoord, f64>

    g_score[start] = 0
    f_score[start] = h(start, goal)
    open_set.push(start, f_score[start])

    while not open_set.is_empty():
        current = open_set.pop()

        if current == goal:
            return reconstruct_path(came_from, current)

        for neighbor in get_neighbors(current):
            tentative_g = g_score[current] + cost(current, neighbor, congestion_map)

            if tentative_g < g_score.get(neighbor, ∞):
                came_from[neighbor] = current
                g_score[neighbor] = tentative_g
                f_score[neighbor] = tentative_g + h(neighbor, goal)

                if neighbor not in open_set:
                    open_set.push(neighbor, f_score[neighbor])

    return None  // 無路徑
```

其中 cost(current, neighbor, congestion_map) 的計算公式為：

```
cost = base_cost + congestion_penalty × congestion_map[neighbor]
```

base_cost = 1（移動一步的成本），congestion_penalty 為可調整的權重（在迭代繞線中遞增），congestion_map 記錄每個 tile 已被多少條繞線使用。

### 路徑重建

找到終點後，透過 came_from 映射表反向追溯路徑：

```
function reconstruct_path(came_from, current):
    path = []
    while current in came_from:
        path.prepend(current)
        current = came_from[current]
    path.prepend(current)  // 加入起點
    return path
```

### 關鍵優化

v2f-pnr 的 A* 實作包含以下優化：

1. **提前終止**：一旦從 open set 中彈出終點節點，立即結束搜尋，無需等待 open set 清空
2. **鄰居快取**：每個 tile 的鄰居列表（上下左右四個方向，邊界處減少）在初始化時預先計算
3. **g_score 預設值**：g_score 初始為 ∞（使用 HashMap 的 entry API），首次訪問時才加入
4. **避免重入 closed set**：由於曼哈頓距離滿足單調性，已處理的節點不需再訪

## 擁塞感知繞線

單純追求最短繞線路徑會導致擁塞問題——多個連線網爭奪同一條繞線通道。v2f-pnr 透過擁塞地圖結合迭代繞線解決此問題。

### 擁塞地圖 (Congestion Map)

擁塞地圖是一個與 tile 陣列同形的二維陣列，記錄每個 tile 目前被多少條繞線路徑經過。核心資料結構：

```rust
type CongestionMap = Vec<Vec<usize>>;
```

每當一條 net 完成繞線，其路徑上的所有 tile 的計數器加 1。在後續迭代中，cost 函數根據計數器值增加通過該 tile 的代價。

### 擁塞成本計算

```
cost(base, congestion, coeff) = base + coeff × congestion
```

其中 coeff 在每次迭代中遞增：
- 第 1 次迭代：coeff = 1
- 第 2 次迭代：coeff = 5
- 第 3 次迭代：coeff = 10
- 第 n 次迭代：coeff = 10 × 2^(n-3)

擁塞越高的 tile，後續迭代中繞過它的傾向越強。

## 迭代繞線 (Iterative Routing)

單次 A* 繞線無法解決 nets 之間的資源競爭。v2f-pnr 採用迭代 rip-up and reroute 策略：

### 演算法流程

```
procedure route_all_nets(design):
    // 第一輪：不考慮擁塞，各自找最短路徑
    for each net in design.nets:
        path = astar_route(net.driver, net.load, congestion_map=empty)
        paths[net.id] = path
        congestion_map.update(path)

    // 迭代改善：最多 5 次
    for iteration in 1..MAX_ITERATIONS:
        congested_nets = identify_congested_nets(paths, congestion_map)

        if congested_nets.is_empty():
            break  // 收斂

        for net in congested_nets:
            // 拆除舊路徑並釋放資源
            congestion_map.remove(paths[net.id])
            paths[net.id] = None

            // 以新的擁塞權重重繞
            new_path = astar_route(net.driver, net.load, congestion_map, coefficient=iteration_coeff(iteration))
            paths[net.id] = new_path
            congestion_map.update(new_path)

    return paths
```

### 收斂判斷

每次迭代結束後檢查是否仍有 tile 的擁塞值超過其容量上限。若所有 tile 都未超載，繞線成功並結束。若達到最大迭代次數（5 次）仍未收斂，報告繞線失敗。

### MAX_ITERATIONS = 5

v2f-pnr 設定最大 5 次迭代的理由：
- 對於小型設計（如 blinky、adder），通常在 1-2 次迭代內即可收斂
- 5 次足以處理中等複雜度的設計
- 超過 5 次若仍未收斂，可能表明設計本身不可繞線或佈局品質太差

## 與其他繞線演算法的比較

### 迷宮繞線 (Maze Routing / Lee Algorithm)

由 C. Y. Lee 於 1961 年提出，是 A* 的前身。Lee 演算法本質上是 BFS（廣度優先搜尋）在網格上的應用，從起點開始波前擴散直到到達終點。

優點：
- 保證找到最短路徑
- 實作非常簡單

缺點：
- 搜尋範圍遍及整個晶片（類似 Dijkstra），時間和空間複雜度為 O(N²)，N 為網格大小
- 對大規模設計效率極低

與 A* 的比較：A* 使用啟發式引導搜尋方向，搜尋範圍明顯小於 Lee 演算法，在保持最優保證的同時大幅提升速度。

### 協商擁塞繞線 (PathFinder)

由 Larry McMurchie 和 Carl Ebeling 於 1995 年提出，是現代 FPGA 繞線的標準演算法。PathFinder 同時考慮了繞線拓撲和擁塞管理。

核心機制：
- 每次迭代中，每條 net 的歷史擁塞成本不斷累積
- 共享資源的競爭越激烈，成本越高
- 迭代到收斂為止

優點：
- 對大型 FPGA 設計效果極佳
- 可處理複雜的時序約束
- 是 VPR、nextpnr 等主流工具的核心繞線引擎

缺點：
- 實作較 A* 複雜
- 需要精心調整歷史成本權重

A* 與 PathFinder 的比較：PathFinder 是一種更高層次的繞線框架，其底層的路徑搜尋子問題仍可用 A* 來解決。v2f-pnr 選擇純 A* 而非完整的 PathFinder，是因為專案規模較小，A* 已足夠滿足需求。

### 史坦納樹繞線 (Steiner Tree Routing)

多終端連線網（multi-terminal net）的最佳拓撲是史坦納樹——找到連接所有節點的最短路徑樹結構。FLUTE 演算法是現代 EDA 中最廣泛使用的史坦納樹建構器。

優點：
- 對多端網的繞線品質顯著優於點到點串聯
- 線長可減少 10-30%

缺點：
- 實作複雜
- 需要配合詳細繞線處理障礙物

v2f-pnr 目前對多端網採用串聯方式（找一條最短路徑連接所有端點），未來可考慮引入史坦納樹以改善繞線品質。

## 為何 v2f-pnr 採用 A*

v2f-pnr 選擇 A* 作為繞線核心演算法，原因如下：

1. **實作簡潔**：核心演算法約 80 行程式碼，無需外部依賴，維護成本低
2. **足夠應付中小型設計**：對於教學範例級別的設計（數千個 nodes），A* 的表現已相當理想
3. **具備良好可擴展性**：可輕鬆整合擁塞感知模組和迭代繞線機制
4. **理論保證**：可容許啟發函數保證繞線結果為最短路徑（在給定成本模型下）
5. **教育價值**：ed4 專案本身偏重教學與展示，使用經典 A* 演算法讓讀者更容易理解 FPGA 繞線的原理
6. **相較 Dijkstra 更高效**：曼哈頓距離啟發式將搜尋範圍約束在起終點間的矩形區域內，搜尋節點數減少約 50-80%

v2f-pnr 的 A* 繞線在大學課程等級的設計（如 MCU0m、hackcpu）中表現良好，能夠快速收斂到無擁塞的繞線結果。對於更大的設計，可以將底層 A* 替換為更複雜的 PathFinder 實作，而整體繞線框架（迭代 rip-up and reroute）保持不變。

## A* 的延伸與變體

雖然 v2f-pnr 採用標準 A*，但值得了解其在 EDA 領域的常見變體：

| 變體 | 特性 | 適用場景 |
|------|------|----------|
| Weighted A* | f = g + w × h，w > 1 加速 | 對最優性要求不嚴格時 |
| D* (Dynamic A*) | 可處理動態成本變化 | 環境變化的重新規劃 |
| D* Lite | D* 的簡化版本 | 增量式路徑重規劃 |
| A* with JPS | 跳點搜尋，大幅減少節點 | 網格圖中的極速搜尋 |
| Hierarchical A* | 先粗後細的層級搜尋 | 超大規模圖形 |

## 參考文獻

- Hart, P. E., Nilsson, N. J., & Raphael, B. (1968). A Formal Basis for the Heuristic Determination of Minimum Cost Paths. IEEE Transactions on Systems Science and Cybernetics, 4(2), 100–107.
- Hart, P. E., Nilsson, N. J., & Raphael, B. (1972). Correction to "A Formal Basis for the Heuristic Determination of Minimum Cost Paths". ACM SIGART Bulletin, (37), 28–29.
- Lee, C. Y. (1961). An Algorithm for Path Connections and Its Applications. IRE Transactions on Electronic Computers, EC-10(3), 346–365.
- McMurchie, L., & Ebeling, C. (1995). PathFinder: A Negotiation-Based Performance-Driven Router for FPGAs. In Proceedings of the 1995 ACM Third International Symposium on Field-Programmable Gate Arrays (FPGA '95), 111–117.
- Dechter, R., & Pearl, J. (1985). Generalized Best-First Search Strategies and the Optimality of A*. Journal of the ACM, 32(3), 505–536.
- Betz, V., Rose, J., & Marquardt, A. (1999). Architecture and CAD for Deep-Submicron FPGAs. Kluwer Academic Publishers.
- Chu, C., & Wong, Y. C. (2007). FLUTE: Fast Lookup Table Based Rectilinear Steiner Minimal Tree Algorithm for VLSI Design. IEEE Transactions on Computer-Aided Design of Integrated Circuits and Systems, 27(1), 70–83.
