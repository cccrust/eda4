# PathFinder：FPGA 協商式擁塞繞線演算法

## 什麼是 PathFinder

PathFinder 是 FPGA 繞線領域最具影響力的演算法之一，由 Larry McMurchie 與 Carl Ebeling 於 1995 年提出。其核心貢獻是引入了**協商式擁塞**（Negotiated Congestion）的概念：不同於傳統繞線方法試圖一次性地為所有網路找到無衝突路徑，PathFinder 採用疊代式的「解除繞線—重新繞線」（rip-up and reroute）策略，逐步讓競爭同一繞線資源的網路協商出各自的路徑。

PathFinder 的關鍵洞察在於：與其試圖避免衝突，不如先繞線再解決衝突。在每一輪疊代中，每個網路都被重新繞線，同時考慮歷史擁塞資訊來引導決策。這種方法保證最終收斂到一個無衝突的繞線方案，且在高擁塞的 FPGA 繞線環境中表現出色。

PathFinder 是眾多學術與商業 FPGA 繞線工具的基石，包括 VPR（Versatile Packing and Routing）和開源的 nextpnr。理解 PathFinder 對於設計和評估任何 FPGA 繞線器都具有根本性的意義。

## 歷史背景

PathFinder 由 Larry McMurchie 和 Carl Ebeling 在華盛頓大學（University of Washington）開發，於 1995 年在 ACM/SIGDA 國際 FPGA 研討會上發表，論文題為 "PathFinder: A Negotiation-Based Performance-Driven Router for FPGAs"。

在 PathFinder 出現之前，FPGA 繞線主要依賴兩類方法。第一類是一次性全域繞線（global routing followed by detailed routing），先將所有網路的路徑全局分配再逐個詳細繞線，但早期決策錯誤會傳播到後續階段且難以修正。第二類是基於通道的繞線（channel routing），將其應用於 FPGA 時面臨架構限制，因為 FPGA 的繞線通道在製造時就已固定，無法像 ASIC 那樣靈活調整寬度。

McMurchie 與 Ebeling 的靈感來自於觀察到 FPGA 繞線問題與通訊網路中的流量控制有著深層的類比：多個網路如同多個資料流，競爭有限的繞線資源。借鑒 TCP 擁塞控制思想，他們設計了一種讓網路在競爭中「協商」出資源分配的機制。

## FPGA 繞線問題

在理解 PathFinder 之前，需要先清楚 FPGA 繞線問題的特殊性。

### 預製繞線架構

與 ASIC 不同，FPGA 的繞線資源在晶片製造時就已完全固定。FPGA 晶片包含大量預先鋪設的繞線線段（wire segments）和可程式化開關（programmable switches），繞線器的任務是在這個固定的拓撲結構中選擇合適的路徑。

以 iCE40 架構為例，每個邏輯磚塊（logic tile）之間存在：
- **水平繞線通道**：連接相鄰磚塊
- **垂直繞線通道**：連接上下磚塊
- **開關方塊（switch box）**：允許訊號在不同方向的繞線通道之間切換
- **連接方塊（connection box）**：允許訊號進入或離開邏輯磚塊

### 問題形式化

給定一組網路 \( N = \{n_1, ..., n_k\} \)（每個連接一組已放置的終端）和一組繞線資源 \( R = \{r_1, ..., r_m\} \)（每個資源有容量 \( c(r) \)，通常為 1），要為每個網路 \( n_i \) 找到一組連通的資源 \( S_i \subseteq R \)，使得對所有資源 \( r \)，使用量 \( usage(r) \leq c(r) \)。當 \( usage(r) > c(r) \) 時稱為**溢位**（overflow）。最佳化目標是最小化繞線總長度，並可選地滿足時序約束。

### 困難之處

FPGA 繞線的困難來自三方面。第一是全域性：網路的繞線決策相互依賴，一個網路的選擇會影響其他網路的可用資源，這使得繞線問題無法被分解為獨立的子問題來求解。第二是 NP-hard 本質：即使忽略資源限制，多終端網路繞線（Steiner 樹問題）已經是 NP-hard；加入資源容量限制後問題更加困難。第三是預製限制：不像 ASIC 可以增加繞線層或調整通道寬度，FPGA 的繞線拓撲完全固定，繞線器只能在既有資源中取捨。

## 核心洞察

PathFinder 的關鍵理念是「先繞線，再解決衝突」。傳統思維是「繞線時避免衝突」，但對於大型設計，不可能一開始就知道哪些資源會被競爭。一個網路的最短路徑可能正好經過某個關鍵瓶頸區域，但這個瓶頸區域只有在所有網路都嘗試繞線後才會顯現。

PathFinder 的協商機制運作如下：
1. 初始：所有網路都選擇對自己最有利的路徑（A* 最短路徑）
2. 衝突發生後：提升衝突資源的成本，讓網路被迫重新考慮替代路徑
3. 疊代：重複此過程，讓網路逐步讓出過度競爭的資源
4. 收斂：最終每個網路找到一條可行路徑，且所有資源使用量不超過容量

這個過程類似於人類協商：多個人同時爭奪同一資源時，先各自提出需求，發現衝突後相互讓步，最終達成各方都可接受的分配方案。PathFinder 將這個直觀的過程形式化為一個可證明收斂的演算法。

## 成本函數

PathFinder 的核心是其精心設計的成本函數。每個繞線節點（node）的成本為：

```
cost(node) = b(node) × h(node) × p(node)
```

- **\( b(node) \)**：基本成本（base cost），使用該資源的固有代價。通常設為 1，短線段的成本低於長線段，開關點的成本略高於直通線段（因開關引入延遲）。基本成本的設定反映了 FPGA 架構的物理特性，是成本計算的基準。
- **\( h(node) \)**：歷史擁塞因子（historical congestion factor），累積過去疊代中該節點的擁塞程度
- **\( p(node) \)**：當前擁塞因子（present congestion factor），反映當前疊代中該節點被多少網路使用

### 歷史擁塞因子

歷史擁塞因子是 PathFinder 最關鍵的創新。它的作用類似於「長期記憶」：

```
h_i(node) = h_{i-1}(node) + 1    若 node 在上一次疊代中有溢位
h_i(node) = h_{i-1}(node)         否則
```

這個值從不減少（在某些變體中可設定上限，但原始版永不衰減），確保了兩件關鍵的事。

**記憶效應**：即使當前的繞線方案暫時解決了某個資源的衝突，歷史記錄仍然存在，防止演算法退回原來的衝突狀態。如果沒有歷史記憶，演算法可能解決了 A 資源的衝突卻讓 B 資源的衝突重現。

**避免震盪**：這是 PathFinder 保證收斂的關鍵。如果沒有歷史因子，演算法可能在兩個方案之間來回震盪——方案一擁塞在資源 A，方案二擁塞在資源 B，永遠無法收斂。歷史因子提供單調增加的「厭惡度」，強迫網路尋找全新的路徑。

### 當前擁塞因子

```
p(node) = 1 + max(0, usage(node) - capacity(node))
```

- 無溢位時 \( p = 1 \)（無額外懲罰）
- 有溢位時 \( p = 1 + overflow \)

當前擁塞因子讓網路在當下疊代中避開已過度使用的節點，歷史因子則確保網路不會重複犯同樣的錯誤。兩者相輔相成：當前因子處理「現在」的擁塞，歷史因子記住「過去」的教訓。

### 網路總成本

一個網路的總成本為其所用節點的成本和：

```
cost(net) = Σ_{node in S} b(node) × h(node) × p(node)
```

PathFinder 使用 A* 搜尋來尋找最低成本的路徑。對多終端網路，使用逐步 Steiner 樹建構法：從源頭開始，迭代尋找樹中節點到最近未連接終端的最短路徑，最後選擇性移除冗餘分支。

## 演算法流程

### 主循環

```
procedure PathFinder(N, R):
    for each node r in R: h(r) = 1
    repeat:
        for each net n in N: rip_up(n)
        for each net n in N:
            S_n = A_Star_Route(n, R, cost)
        overflow_found = false
        for each node r in R:
            usage(r) = count_nets_using(r)
            if usage(r) > capacity(r):
                overflow_found = true
                h(r) = h(r) + 1
        if not overflow_found: return SUCCESS
        for each node r in R:
            p(r) = 1 + max(0, usage(r) - capacity(r))
        if iterations > MAX_ITERS: return FAILURE
    end repeat
```

### 解除繞線與重新繞線

每次疊代中，先解除所有既有路徑（從資源使用表移除該網路佔用的節點並清空其繞線路徑），再用當前擁塞資訊重新繞線。這保證了每個網路都有公平的機會選擇低成本路徑——如果只解除溢位資源上的部分網路而保留其他網路的路徑，那麼被保留的網路就會處於不公平的優勢地位。

為什麼需要解除所有網路？因為即使一個網路當前使用的資源沒有溢位，它在後續疊代中也可能因為其他網路的繞線決策變化而失去最佳路徑。完全重新繞線確保了演算法在每個疊代中都能基於最新的擁塞資訊做出全局最優的分配。

### A* 繞線子程序

A* 的評價函數為 \( f(node) = g(node) + h(node) \)，其中：
- \( g(node) \)：從起點到當前節點的實際路徑成本，等於路徑上所有節點 \( b × h × p \) 的累加
- \( h(node) \)：從當前節點到終點的估計成本（可接納的啟發式函數，如曼哈頓距離）

由於 A* 保證在可接納啟發式下找到最短路徑，PathFinder 在每一輪疊代中都能為每個網路找到當前成本定義下的最佳路徑。這保證了每次疊代都是在給定擁塞條件下的最優選擇。

對多終端網路，PathFinder 採用逐步 Steiner 樹建構法：
1. 初始化 Steiner 樹，包含源頭終端
2. 重複直到所有終端都被包含：
   - 使用 A* 尋找樹中任一節點到最近未連接終端的最短路徑
   - 將該路徑及終端加入 Steiner 樹
3. 最後對整棵 Steiner 樹進行選擇性移除冗餘分支（branch pruning）

### 收斂保證

由於歷史擁塞因子 \( h \) 單調增加，當某節點被過度使用 \( k \) 次時 \( h(r) \) 至少增加 \( k \)，該節點的繞線成本持續上升。當 \( h(r) \) 足夠大時，任何網路的 A* 搜尋都會傾向於避開該節點，轉向替代路徑。

替代路徑的存在由 FPGA 繞線架構的冗餘度保證。若架構本身不可繞線（即使最佳分配也無法滿足所有網路的資源需求），PathFinder 會檢測到無法收斂的情況而報告失敗。換句話說，PathFinder 不僅是一個繞線演算法，同時也是一種繞線可行性驗證工具——如果它無法收斂，就表示該設計在當前 FPGA 架構上無法實現繞線。

收斂所需的疊代次數取決於幾個因素：擁塞程度（資源越緊缺疊代越多）、繞線架構冗餘度（替代路徑越多收斂越快）、網路規模（越多網路干擾越大）以及 A* 啟發式的精準度。典型情況下 PathFinder 需要 20 到 100 次疊代達到收斂，對大型設計可能需要更多。

### 時序驅動版本

原始論文同時提出了時序驅動的變體，引入**時臨界度**（criticality）因子：

```
cost(node) = b(node) × h(node) × p(node) × crit(net)
```

或更常見的雙項形式：

```
cost(node) = b(node) × (1 - crit(net)) × h(node) × p(node) + crit(net) × delay(node)
```

其中 \( crit(net) \) 基於網路的時序寬裕度（slack）計算：\( crit(net) = 1 - slack(net) / D_max \)，其中 \( D_max \) 是關鍵路徑的最大延遲。寬裕度越小的網路（時序越緊迫），臨界度越高，傾向於走最短路徑（即使通過擁塞區域）；非關鍵網路則避開擁塞。這需要靜態時序分析（STA）引擎的支援，增加了實作複雜度。

## 與 v2f-pnr 繞線策略的比較

v2f-pnr 的繞線器位於 `v2f-pnr/src/route.rs`，採用疊代式 A* 繞線但沒有 PathFinder 的協商式擁塞循環。

v2f-pnr 的策略要點：
1. 建立空的擁塞地圖 `HashMap<TileCoord, u32>`
2. 對每個網路使用 A* 繞線，A* 成本含繞線長度 + 擁塞懲罰（penalty × 10）
3. 繞線後將使用到的磚塊擁塞計數加 1
4. 最多 5 次疊代，每次疊代後將所有擁塞計數減 1（衰減機制）

兩種策略的詳細對比：

| 特性 | PathFinder | v2f-pnr |
|------|-----------|---------|
| **歷史擁塞** | 累積增加，永不衰減 | 每次疊代衰減，無長期記憶 |
| **溢位檢測** | 顯式計算每個資源溢位 | 無顯式溢位計算 |
| **收斂保證** | 有（歷史因子單調增加） | 無（可能震盪） |
| **重新繞線策略** | 解除所有網路再全部重繞 | 在先前的路徑上增量繞線 |
| **最大疊代** | 無強制限制（至收斂） | 上限 5 次 |
| **資源模型** | 資源級別（每個開關/線段） | 磚塊級別（每個 tile） |
| **時序驅動** | 可選（criticality 因子） | 不支援 |
| **擁塞因子形式** | 乘法（b × h × p） | 加法（長度 + penalty） |

v2f-pnr 可借鑑的改進方向：引入累積式歷史擁塞因子（取代衰減機制，避免震盪）、添加顯式溢位檢測（指導成本調整而非盲目懲罰所有使用過的資源）、每次疊代完全重新繞線（確保公平性）、加入收斂檢測提前終止。

## 為何有效

**博弈論視角**：PathFinder 的疊代過程類似非合作賽局。每個網路是試圖最小化成本的理性參與者。歷史擁塞因子是反壟斷機制——當資源被過度使用時成本持續上升，直到某些網路讓步。可證明 PathFinder 的成本函數對應於嚴格的勢函數（potential function），每次疊代降低系統總成本，最終收斂到 Nash 均衡。

**機器學習視角**：PathFinder 類似強化學習——狀態為當前繞線方案，行動為選擇新路徑，負的繞線成本為獎勵，歷史因子累積了過去的「失敗經驗」。每次疊代如同經驗回放，從錯誤中學習並避免重蹈覆轍。

**物理類比**：多個彈性繩同時通過一個圓環，每根都想走直線，但圓環容量有限。每次疊代如同一次「抖動」，在圓環處施加額外阻力，直到部分繩子繞道其他圓環。

## 在業界的應用

- **VPR**（University of Toronto）：完全採用 PathFinder 協商式擁塞機制，支援時序驅動繞線和多種 FPGA 架構模型。VPR 8.0 引入了加性加權（additive weighting）變體，將歷史因子以加法而非乘法形式融入成本函數，改善了收斂行為。
- **nextpnr**（Project IceStorm）：繞線引擎基於 PathFinder 思想但使用疊代加深策略（從少量疊代開始，必要時增加），使用 A* 路徑搜尋，支援 iCE40、ECP5、Nexus 系列 FPGA。nextpnr 的繞線資源圖包含完整的繞線線段和開關，比 v2f-pnr 的磚塊級模型精細得多。
- **商用工具**：Xilinx Vivado、Intel Quartus 採用類似疊代式協商繞線策略，具體實作細節各有差異但核心思想一致。

## 變體與擴展

- **時序驅動 PathFinder**：引入基於 slack 的 criticality 因子，使時序關鍵網路優先使用最短（低延遲）路徑。臨界度的計算公式為 \( crit(net) = 1 - slack(net) / D_{max} \)，需要 STA 引擎提供路徑延遲資訊。
- **低功耗 PathFinder**：成本加入功耗項，避免長線段（寄生電容大）和開關點（動態功耗），合併共用路徑減少冗餘切換。功耗項的權重可根據設計的功耗預算動態調整。
- **串擾感知 PathFinder**：考慮平行繞線長度和訊號翻轉率，對長距離平行走線施加額外的成本懲罰。適用於深次微米製程中串擾成為主要瓶頸的設計。
- **多目標 PathFinder**：每個目標（延遲、功耗、串擾）有獨立的歷史因子和當前因子，總成本為各項加權和。可同時最佳化多個目標，但參數調校更為複雜。
- **PathFinder-Hybrid**：結合 PathFinder 與啟發式初始繞線（如基於 Steiner 樹的預測），減少收斂所需的疊代次數。適合需要快速原型驗證的設計流程。

## 與 v2f-pnr 繞線器整合的可能性

對於 eda4 專案的 v2f-pnr crate，引入 PathFinder 風格的繞線器是一個有價值的擴展方向。可能的整合方案包括：

1. **最小改動方案**：在現有 route.rs 的基礎上，將 congestion_map 的更新策略從「每次衰減」改為「累積式永不衰減」，並添加顯式 overflow 檢測。這約需 20 行程式碼修改。

2. **標準 PathFinder 方案**：重構 route.rs 以實現完整的 PathFinder 循環（解除所有網路 → 重新繞線 → 檢測 overflow → 更新歷史因子）。這約需 100-150 行程式碼新增。

3. **完整方案**：實現時序驅動的 PathFinder，包含簡單的 STA 引擎（基於單元延遲和線負載估計），使 v2f-pnr 能夠支援時序最佳化。這約需 300-500 行程式碼新增。

這些改進可以逐步實施，從最小改動開始，驗證效果後再擴展到更完整的實作。

## PathFinder 與其他繞線策略的比較

PathFinder 屬於疊代式迷宮繞線（iterative maze routing）的範疇，與其他繞線策略有顯著差異：

**一次性迷宮繞線**（non-iterative maze routing）：對每個網路依序使用 A* 或 Dijkstra 繞線，不對已繞線的網路做任何修改。這種方法速度快但品質差——先繞線的網路會佔據最短路徑，後繞線的網路被迫繞遠路。v2f-pnr 的第一次疊代相當於此方法。

**順序繞線（sequential routing）**：對網路按某種順序（如關鍵度排序）逐一繞線，已繞線的網路被鎖定。這種方法比一次性繞線好，但順序的選擇對結果影響巨大且不可逆。

**PathFinder**：所有網路在每輪疊代中被完全重新繞線，歷史擁塞因子確保了長期記憶。這是最公平也最魯棒的方法，但疊代次數較多。

**增量繞線（incremental routing）**：只對需要修改的局部區域進行重新繞線。v2f-pnr 的策略類似於此，但缺乏歷史擁塞因子導致可能震盪。增量繞線速度快但品質不如完全重新繞線。

## PathFinder 在 nextpnr 中的實作

nextpnr 是 eda4 專案的外部參考工具，其繞線引擎採用了 PathFinder 的思想但做了重要調整：

1. **資源圖模型**：nextpnr 為 iCE40 建立了完整的繞線資源圖（routing resource graph），包含所有可程式化開關、繞線線段和連接點。這比 v2f-pnr 的磚塊級模型精細得多，但也更複雜。

2. **疊代加深策略**：nextpnr 從少量疊代開始（預設約 30 次），如果未收斂則增加疊代次數。這種方法避免了在簡單設計上浪費計算時間。

3. **優先繞線**：nextpnr 支援對關鍵網路（如時脈、重置信號）優先繞線，將它們從普通網路的競爭中隔離出來。

4. **時序驅動**：nextpnr 內建靜態時序分析引擎，可以在繞線過程中即時計算路徑延遲並調整 criticality 因子。

5. **合法化檢查**：nextpnr 的繞線保證每個資源只被一個網路使用，不允許暫時的溢位——這與原始 PathFinder 允許疊代中存在溢位的做法不同。

## PathFinder 實現注意事項

在實作 PathFinder 時需要處理以下細節：

**圖模型建構**：需要將 FPGA 架構轉換為有向圖 \( G(V, E) \)，其中節點代表繞線資源（線段、開關、連接點），邊代表資源間的連通性。對於 iCE40 這樣的商用 FPGA，完整的繞線資源圖可能包含數十萬個節點。圖模型的精細程度直接影響繞線品質和執行時間。

**多終端網路處理**：正確的 Steiner 樹建構需要維護已連接終端集合，並在每次添加新路徑後更新樹結構。分支修剪（branch pruning）可以移除冗餘路徑段，但需要確保不破壞連通性。常見的實作是 Prim-Dijkstra 演算法的變體。

**成本計算溢位處理**：\( b × h × p \) 的乘積可能導致數值溢位，尤其當疊代次數較多時。可以對成本取對數或設定上限來避免此問題。VPR 使用了 \( cost = b × (h + p) \) 的加法形式來避免數值問題，同時保持了 PathFinder 的收斂特性。

**優先級排定**：在每輪疊代中，網路的繞線順序會影響結果。常見做法是先繞線時序關鍵的網路，或使用隨機順序來避免偏差。有些實作在每次疊代中隨機化網路順序，以確保公平性和避免順序偏差。

**提早終止**：若連續多輪疊代的擁塞情況沒有改善，可以提前終止以避免浪費計算。例如，若溢位節點數量連續 5 次疊代沒有減少，即可判定設計不可繞線或已達到繞線品質的極限。

## PathFinder 參數調校實務

在實作 PathFinder 時，以下參數對結果品質和執行時間有顯著影響：

**歷史因子初始值**：\( h_{start} \) 通常設為 1，但對於資源極度緊缺的設計，可以設為較大的初始值（如 10）以加速收斂。

**歷史因子增長幅度**：每次疊代的增量可以是 1（標準做法），也可以設為更大值（如 2 或動態調整）以加速收斂，但可能犧牲繞線品質。

**當前因子計算方式**：\( p(node) = 1 + K × overflow \) 中的 K 值影響擁塞懲罰的強度。K 值過大會讓網路過早放棄潛在的可行路徑，K 值過小則收斂緩慢。

**最大疊代次數**：雖然 PathFinder 理論上保證收斂，但實務上需設定上限。一般設為 100-500 次，視設計規模和時序要求而定。

**初始繞線策略**：第一次繞線可以使用忽略成本的 BFS（廣度優先搜尋）來建立基線路徑，也可以用完整的成本函數。前者收斂較慢但初始方案更均勻。

## 延伸閱讀

- McMurchie, L., & Ebeling, C. (1995). PathFinder: A Negotiation-Based Performance-Driven Router for FPGAs. FPGA '95. 此為原始論文，詳細描述了演算法的完整推導和實驗結果。
- Betz, V., & Rose, J. (1997). VPR: A New Packing, Placement and Routing Tool for FPGA Research. FPL '97. 介紹 VPR 如何整合 PathFinder 進行架構評估。
- Murray, K. E., et al. (2020). VTR 8: High-Performance CAD and Customizable FPGA Architecture Modelling. ACM TRETS. VTR 8 的架構和演算法改進，包括 PathFinder 的加性加權變體。
- Swartz, J. S., Betz, V., & Rose, J. (1998). A Fast Routability-Driven Router for FPGAs. FPGA '98. PathFinder 的加速變體。
- [Place & Route (PNR) 佈局與繞線](pnr.md) — eda4 專案中 PNR 階段的完整介紹
- [A* 路徑搜尋](a_star_routing.md) — PathFinder 使用的底層路徑搜尋演算法
- [模擬退火](simulated_annealing.md) — v2f-pnr 佈局階段使用的演算法
- [iCE40 架構](ice40.md) — 目標 FPGA 平台的硬體繞線架構
- [CRAM / Frame / 位元流](cram_bitstream.md) — 繞線結果最終轉為 FPGA 配置資料

---

*本文為 eda4 專案 wiki 的一部分，對應的繞線器實作位於 `verilog2fpga/v2f-pnr/src/route.rs`。PathFinder 的完整實作可參考 VPR 或 nextpnr 的原始碼。*

*主要參考文獻：McMurchie & Ebeling (1995) 原始論文、Betz & Rose (1997) VPR 論文、VTR 8 技術文件 (Murray et al., 2020)。*

*對應的 v2f-pnr 繞線器使用疊代式 A\* 搭配擁塞地圖，但尚未實現 PathFinder 的完整協商式擁塞機制。*

## 延伸閱讀

- [PathFinder (Wikipedia)](https://en.wikipedia.org/wiki/PathFinder_(FPGA_router))
- [Routing (Electronic Design) (Wikipedia)](https://en.wikipedia.org/wiki/Routing_(electronic_design))
- [Negotiated Routing (Wikipedia)](https://en.wikipedia.org/wiki/Routing_(electronic_design)#Negotiated_congestion_routing)
