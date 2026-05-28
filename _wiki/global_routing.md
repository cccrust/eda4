# 全域繞線 (Global Routing)

## 什麼是全域繞線

全域繞線是實體設計流程中繞線階段的第一個步驟。在這一階段中，晶片被劃分為多個繞線區域（如 tiles、channels 或 GCells），演算法決定每一條訊號線（net）需要經過哪些區域，但尚未指定具體的金屬軌道（track）或層級（layer）。換句話說，全域繞線提供一個巨觀的路徑規劃，作為後續詳細繞線的輸入。

全域繞線的輸出通常是一個區域層級的繞線資源分配表，記錄每個繞線區域的預期使用量（demand）與容量（capacity），以及每條 net 所經過的區域序列。詳細繞線再根據這些資訊在每個區域內進行精確的軌道分配與金屬連線。

## 在實體設計流程中的位置

典型的 ASIC 實體設計流程如下：

1. **平面規劃 (Floorplanning)** — 決定晶片總體佈局，包括各功能區塊的位置、I/O pad 的擺放、電源網路規劃。
2. **元件擺置 (Placement)** — 將標準單元 (standard cells) 放置到規劃好的位置上，目標是最小化總線長、滿足時序約束。
3. **全域繞線 (Global Routing)** — 決定每條 net 經過哪些繞線區域，不涉及具體的軌道或金屬層。
4. **詳細繞線 (Detailed Routing)** — 在每個繞線區域內分配具體的軌道、金屬層與導孔 (via)，完成所有幾何層級的連線。
5. **佈局後處理 (Layout Finishing)** — 執行 DRC 修正、天線效應修復、OPC 光學鄰近修正等最終處理。

全域繞線介於擺置與詳細繞線之間，起到橋樑作用。它不僅為詳細繞線提供了搜尋空間的縮減，也能在早期階段評估繞線擁塞程度，必要時回饋調整擺置或平面規劃。

## 為什麼需要全域繞線

詳細繞線的計算複雜度極高。以一顆先進製程晶片為例，訊號線數量可達數百萬條，繞線網格動輒數億個格點。若直接對整個晶片進行詳細繞線，所需的記憶體與執行時間將難以負荷。

全域繞線透過以下方式解決此問題：

- **問題分解**：將大規模的繞線問題拆解為多個較小的子問題（每個繞線區域獨立處理）。
- **資源預測**：在早期階段就識別擁塞熱點 (congestion hotspot)，避免在後期才發現無法繞通的情況。
- **迭代優化**：全域繞線的反饋迴圈可以調整 net 的路徑，使各區域的資源使用平衡。
- **設計收斂**：提供詳細繞線一個良好的初始解，減少 rip-up 與 reroute 的次數。

## 繞線資源模型

全域繞線的核心是將晶片覆蓋上一層粗網格 (coarse grid)，稱為 GCell (Global Routing Cell) 網格：

```
+------+------+------+------+
| GCell| GCell| GCell| GCell|
| 0,0  | 1,0  | 2,0  | 3,0  |
+------+------+------+------+
| GCell| GCell| GCell| GCell|
| 0,1  | 1,1  | 2,1  | 3,1  |
+------+------+------+------+
| GCell| GCell| GCell| GCell|
| 0,2  | 1,2  | 2,2  | 3,2  |
+------+------+------+------+
```

每個 GCell 的邊界（edge）都有一個**容量 (capacity)**，代表該邊界上可用的繞線軌道數量。容量取決於金屬層的數量、最小間距 (minimum spacing)、以及該區域已被電源網路、時脈網路等佔用的資源。

當一條 net 從 GCell A 穿過邊界到 GCell B 時，該邊界的**使用量 (demand)** 就增加 1。若 demand 超過 capacity，則稱為**溢流 (overflow)**，表示該區域發生擁塞。

## 繞線區域類型

在傳統的閘陣列 (gate array) 與標準單元設計中，繞線區域可分為以下幾種：

- **通道 (Channel)**：位於兩列標準單元之間的繞線區域，經典通道繞線即在這些固定高度的水平區域中進行。
- **開關盒 (Switchbox)**：通道與通道的交會區域，通常位於通道的端點，連接著多個方向的繞線資源。
- **元件上方繞線層 (Over-the-Cell Routing)**：隨著多層金屬技術的進步，繞線可以在標準單元上方進行，充分利用垂直空間。一般來說，下層金屬（M1-M2）用於單元內部連線與 pin 存取，中上層金屬（M3 以上）用於全域訊號繞線。

## 全域繞線目標

全域繞線的問題本質上是一個多目標最佳化問題，需要同時考量以下目標：

### 最小化總線長 (Minimize Total Wirelength)

總線長直接影響晶片面積、功耗與延遲。全域繞線階段通常以曼哈頓距離 (Manhattan distance) 或 Steiner 樹線長作為近似評估。

### 平衡擁塞 (Balance Congestion)

避免大量 net 集中通過少數區域是全域繞線的首要任務。擁塞區域會導致詳細繞線無法完成，進而需要重新擺置或修改平面規劃。常用的擁塞指標包括：

- **溢流總量 (Total Overflow)**：所有超出容量的邊界之超額量總和。
- **最大溢流 (Max Overflow)**：單一邊界最大的超額量。
- **擁塞峰值 (Peak Congestion)**：需求與容量的最大比值。

### 滿足時序約束 (Meet Timing Constraints)

關鍵路徑上的 net 需要更短、更快的路徑。全域繞線可以透過以下方式滿足時序約束：

- 為關鍵 net 預留專屬的快速路徑。
- 限制關鍵 net 的最大繞線長度。
- 減少關鍵 net 的導孔數量，以降低電阻與延遲。

### 最小化導孔數量 (Minimize Via Count)

每個導孔 (via) 都會引入：

- 額外的電阻與電容，增加訊號延遲。
- 可靠度風險，導孔是金屬疲勞與電遷移 (electromigration) 的薄弱環節。
- 良率損失，導孔缺陷是常見的製造缺陷來源。

因此在全域繞線中應盡量減少 net 的層級切換次數。

## 演算法方法

全域繞線的演算法可大致分為以下幾類：

### 順序繞線 (Sequential Routing)

每次處理一條 net，為其選擇最佳路徑，已繞好的 net 路徑視為障礙物。

- **優點**：實作簡單，執行速度快。
- **缺點**：先前繞好的 net 可能阻擋後續 net 的路徑，導致順序依賴性問題。

代表性方法：

- **整數線性規劃 (ILP, Integer Linear Programming)**：將每個 GCell 邊界的容量作為約束條件，以線性規劃求解每條 net 的 flow 分配。ILP 能夠找到全域最優解，但對於大規模設計的求解時間過長，通常用於小區塊或局部最佳化。
- **協商擁塞繞線 (Negotiated Congestion Routing, PathFinder)**：PathFinder 演算法最初用於 FPGA 繞線，其核心概念是「歷史擁塞成本」。每次迭代中，所有 nets 同時重新繞線，若某條 net 使用了擁塞的資源，該資源的成本會提高。透過多次迭代，nets 會逐漸避開擁塞區域。PathFinder 的關鍵優勢在於它能自然地處理順序依賴性，不需要顯式的 rip-up 順序。

PathFinder 的成本函數為：

```
cost(n) = baseCost(n) + history(n) * present(n)
```

其中 `history(n)` 為歷史擁塞懲罰值，`present(n)` 為當前迭代的資源使用次數。

### 並行繞線 (Concurrent Routing)

同時考慮所有 nets 的路徑，求解一個全域最佳化問題。

- **多商品流 (Multicommodity Flow)**：將每條 net 視為一種「商品」，每個 GCell 邊界視為容量有限的「管線」。問題轉化為在容量限制下最大化流量的線性規劃。由於變數數量龐大，通常需要結合分解技術（如 Lagrangian relaxation）。
- **迭代刪除 (Iterative Deletion)**：先為所有 nets 建立過多的候選路徑（每條 net 有多條 alternative），然後逐步刪除不必要的路徑以滿足容量限制。

並行繞線的優勢在於能達到更高的繞線品質，但計算成本遠高於順序繞線。

### 階層式繞線 (Hierarchical Routing)

將晶片遞迴地劃分為更小的區域，從頂層開始依序處理：

1. 將晶片劃分為 2×2 或 4×4 個子區域。
2. 在當前層級決定每條 net 穿越子區域邊界的位置。
3. 遞迴地在每個子區域內重複步驟 1-2，直到區域小於某個門檻。
4. 在底層使用詳細繞線或細粒度的全域繞線完成連線。

階層式繞線的優點是能夠有效處理超大規模設計，但路徑品質可能不如扁平式 (flat) 繞線，且上下層之間的路徑一致性需要特別注意。

### 樣式繞線 (Pattern Routing)

對於一條兩端點連線，使用預先定義好的幾何樣式來生成路徑：

- **L 型繞線 (L-shaped)**：由一條水平線與一條垂直線組成，最簡單的兩端點連線方式，有兩種選擇（先水平再垂直，或先垂直再水平）。
- **Z 型繞線 (Z-shaped)**：H-V-H 或 V-H-V 三線段組成，提供更多的路徑多樣性以避開障礙物。
- **U 型繞線 (U-shaped)**：在不允許 L 型或 Z 型路徑時使用，繞過障礙物。

樣式繞線速度快，但在密集區域的效果有限，通常與迷宮繞線 (maze routing) 結合使用。

## Steiner 樹在全域繞線中的應用

對於多端點 net（multi-pin net），全域繞線需要建立一棵連接所有端點的樹狀結構。**直角 Steiner 樹 (Rectilinear Steiner Tree, RST)** 是理論上的最佳線長解。

一個 Steiner 樹與最小生成樹 (Minimum Spanning Tree, MST) 的關鍵差異在於：Steiner 樹允許引入額外的 Steiner 節點，以減少總線長。在最壞情況下，Steiner 樹的線長可比 MST 減少約 13.4%。

實務上，由於建構最佳 Steiner 樹是 NP-hard 問題，全域繞線通常使用啟發式演算法：

- **FLUTE (Fast Lookup Table based Steiner Tree)**：透過預先計算的查詢表，在 O(n log n) 時間內建構接近最佳解的 Steiner 樹，廣泛應用於各家 EDA 工具中。
- **Prim-Dijkstra 混合演算法**：結合 Prim MST 與 Dijkstra 最短路徑的優點，在建構樹的同時考慮路徑成本。

## 擁塞圖 (Congestion Map)

全域繞線完成後，會產生一張**擁塞圖**來視覺化每個區域的資源使用情況。擁塞圖通常用熱力圖 (heatmap) 表示：

- **綠色**區域代表低使用率（demand << capacity）。
- **黃色**區域代表中度使用率（demand ≈ capacity）。
- **紅色**區域代表擁塞（demand > capacity），即溢流區域。

擁塞圖對於設計工程師非常重要，因為它能直觀地指出需要重新擺置或調整平面規劃的區域。在現代 EDA 工具中，擁塞圖是設計收斂過程中的關鍵診斷工具。

## 全域繞線指標

評估全域繞線品質的主要指標包括：

| 指標 | 說明 |
|------|------|
| **總溢流 (Total Overflow)** | 所有擁塞邊界上 demand 超過 capacity 的總和 |
| **最大溢流 (Max Overflow)** | 單一邊界上最大的超額量，應為 0 表示無擁塞 |
| **線長 (Wirelength)** | 所有 net 的總曼哈頓長度或 Steiner 長度 |
| **導孔數 (Via Count)** | 所有 net 所使用的導孔總數（若在全域階段估算） |
| **執行時間 (Runtime)** | 演算法完成所需的 CPU 時間 |
| **可繞通性 (Routability)** | 詳細繞線階段能否成功完成所有連線 |

其中溢流指標是最重要的約束條件，理想的全域繞線結果應達到零溢流 (zero overflow)。

## ISPD 全域繞線評比基準

2008 年，國際實體設計研討會 (ISPD) 舉辦了著名的全域繞線競賽，提供了標準化的評比基準 (benchmark) 與評估流程。這些基準電路來自 IBM 的實際工業設計，涵蓋不同的規模與複雜度。競賽的主要貢獻包括：

- 標準化的 benchmark 格式（包含 GCell 網格定義、容量資訊、net 列表）。
- 統一的評估腳本，計算溢流、線長與執行時間。
- 建立了可重複的學術研究基準。

著名的競賽作品包括：

- **NTUgr** (國立臺灣大學) — 採用多階段最佳化，結合 ILP、Steiner 樹與 pattern routing。
- **FastRoute** (UCLA) — 以快速的 pattern routing 結合迷宮繞線為核心。
- **BFG-R** (IBM) — 採用基於 box 的繞線與協商擁塞機制。
- **FGR** (UIUC) — 使用多商品流與 Lagrangian relaxation。

這些競賽極大地推動了全域繞線演算法的發展，許多技術至今仍被業界工具所採用。

## FPGA 脈絡下的全域繞線

在 FPGA 的脈絡中，繞線問題的性質與 ASIC 有顯著不同：

### FPGA 繞線結構

FPGA 的繞線架構是預先製造好的，包含：

- **可程式化繞線資源**：已固定存在的金屬線段 (wire segment) 與可程式開關 (programmable switch)。
- **繞線通道**：每個 tile（或 LAB/CLB）周圍有固定數量的繞線通道，通道內包含不同長度的 wire segment（如 single、double、hex、long 等）。
- **切換塊 (Switch Block)**：位於 tile 之間的連接點，透過可程式開關實現不同 wire segment 的連接。

### FPGA 繞線 = 全域 + 詳細的混合

FPGA 繞線器（如 VPR）的傳統做法是同時處理全域與詳細繞線。由於 FPGA 的繞線資源是固定的、離散的，繞線器在選擇某個 wire segment 時，既決定了路徑的大致方向（相當於全域繞線），也決定了具體使用的資源（相當於詳細繞線）。因此 FPGA 繞線通常不分為獨立的兩個階段。

### v2f-pnr 的繞線策略

在 v2f-pnr（本專案的繞線工具）中：

- 採用 A* 搜尋演算法在 tile 層級進行路徑搜尋。
- 單一 pass 同時完成路徑規劃與資源選擇 — 沒有獨立的全域繞線階段。
- 迭代式 A* 繞線過程中的擁塞圖 (congestion map) 某種程度上提供了全域繞線的意識。
- 當某個 tile 的繞線資源不足時，歷史擁塞成本會增加，驅使後續的繞線嘗試避開該 tile。

這意味著 v2f-pnr 的繞線相當於將全域繞線與詳細繞線合而為一，以 tile 層級的繞線資源競爭來隱式地達到全域繞線的效果。

### FPGA 與 ASIC 的關鍵差異

| 層面 | ASIC | FPGA |
|------|------|------|
| 繞線架構 | 客製化金屬層，無預設路徑 | 預先製造的 wire segment 矩陣 |
| 繞線階段 | 全域 → 詳細，兩個獨立階段 | 通常合併為一個階段 |
| 資源 | 理論上無限制（受限於設計規則） | 固定數量，無法增加 |
| 目標 | 最小化面積與延遲 | 在固定資源下完成所有連線 |
| 容量瓶頸 | 金屬層密度 | 通道使用率與 switch 使用率 |

## 全域繞線 vs 詳細繞線

總結兩者的分工：

| 項目 | 全域繞線 | 詳細繞線 |
|------|----------|----------|
| 顆粒度 | GCell 層級（數十至數百個電晶體寬度） | 金屬軌道層級（奈米尺度） |
| 決定事項 | net 經過哪些繞線區域 | net 使用哪些具體軌道與導孔 |
| 模型 | 容量 + 需求（無幾何細節） | 最小間距、最小寬度、設計規則 |
| 輸出 | 區域層級的路徑分配表 | 完整的 GDSII 佈局 |
| 計算量 | 相對較低，可處理全晶片 | 極高，需要分割處理 |
| 主要演算法 | ILP、多商品流、PathFinder | Lee、A*、通道繞線、軌道分配 |

全域繞線的品質直接決定了詳細繞線能否順利完成。一個全域繞線零溢流的結果並不能保證詳細繞線百分百成功 — 但若全域繞線存在大量溢流，詳細繞線幾乎必然失敗。

## 參考資料

- [Wikipedia: Global routing](https://en.wikipedia.org/wiki/Global_routing)
- [Wikipedia: Electronic design automation](https://en.wikipedia.org/wiki/Electronic_design_automation)
- [Wikipedia: Place and route](https://en.wikipedia.org/wiki/Place_and_route)
- [Wikipedia: Steiner tree problem](https://en.wikipedia.org/wiki/Steiner_tree_problem)
- [ISPD 2008 Global Routing Contest](http://www.ispd.cc/contests/08/ispd08rc.html)
- [NTUgr: National Taiwan University Global Router](https://github.com/ntu-adsl/NTUgr)
- [VTR Project (includes VPR router for FPGAs)](https://github.com/verilog-to-routing/vtr-verilog-to-routing)
- [PathFinder: A Negotiation-Based Performance-Driven Router for FPGAs](https://doi.org/10.1145/244192.244222) — L. McMurchie and C. Ebeling, FPGA 1995

## 延伸閱讀

- [Global Routing (Wikipedia)](https://en.wikipedia.org/wiki/Place_and_route#Global_routing)
- [Integer Programming (Wikipedia)](https://en.wikipedia.org/wiki/Integer_programming)
- [Minimum Spanning Tree (Wikipedia)](https://en.wikipedia.org/wiki/Minimum_spanning_tree)
