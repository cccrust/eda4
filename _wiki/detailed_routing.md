# 詳細繞線 (Detailed Routing)

## 什麼是詳細繞線

詳細繞線是實體設計流程中繞線階段的最後一步。在前一階段的全域繞線決定每條訊號線 (net) 通過哪些繞線區域之後，詳細繞線負責在每個區域內分配**具體的金屬軌道 (track)**、**金屬層 (layer)** 與 **導孔 (via)**，完成所有連線的幾何層級實現。

詳細繞線的輸入包括：
- 全域繞線結果（每條 net 的區域級路徑）
- 標準單元 (standard cells) 的擺置位置
- 單元引腳 (pin) 的實際幾何座標
- 設計規則 (design rules) 檔案

詳細繞線的輸出則為完整的實體佈局資料，通常以 GDSII 或 LEF/DEF 格式儲存。這個佈局將直接送往光罩製造 (mask fabrication) 流程。

## 在實體設計流程中的位置

詳細繞線位於全域繞線之後，是晶片設計的最終實現階段：

```
平面規劃 → 元件擺置 → 全域繞線 → 詳細繞線 → 佈局後處理
                                    ↑
                                 (本階段)
```

詳細繞線完成後，設計進入佈局後處理階段，包括：
- **設計規則檢查 (DRC)**：驗證所有幾何圖形是否符合製造規則。
- **電路萃取 (RC Extraction)**：從佈局中萃取出寄生電阻與電容。
- **靜態時序分析 (STA)**：驗證時序收斂。
- **天線效應修復 (Antenna Fix)**：修正不符合天線比率規範的線段。
- ** redundant via 插入**：在關鍵導孔旁增加備用導孔以提高良率。

若上述檢查中發現違規，需要回到詳細繞線甚至更早的階段進行修正，形成設計閉合迴圈 (design closure loop)。

## 關鍵約束

詳細繞線必須滿足大量的製造約束（設計規則），這些規則在半導體製程中至關重要。

### 最小間距 (Minimum Spacing)

同一金屬層上的兩條相鄰導線之間必須保持不小於最小間距的距離。間距規則取決於：

- 金屬層（下層金屬間距通常比上層小）
- 導線平行長度（平行長度越長，間距要求越大）
- 轉角效應（轉角處的間距通常比直線處寬鬆）

在 7nm 以下製程中，最小間距規則變得極為複雜，需要考慮多種圖形依賴效應 (pattern-dependent effects)。

### 最小寬度 (Minimum Width)

每一條金屬導線的寬度必須大於或等於製程所允許的最小寬度。不同金屬層的最小寬度可能不同：下層金屬（M1）通常最窄，上層金屬（M4-M6 以上）較寬以承載較大的電流。

### 導孔間距與包覆 (Via Spacing and Enclosure)

導孔 (via) 的約束包括：

- **導孔間距 (via-to-via spacing)**：相鄰導孔之間必須保持足夠距離。
- **導孔包覆 (via enclosure)**：導孔周圍的金屬必須有足夠的包覆面積，以確保導孔的結構強度與導電性。
- **導孔堆疊 (via stacking)**：多層導孔堆疊時有額外的間距與 alignment 要求。

### 天線規則 (Antenna Rules)

在電漿蝕刻過程中，長金屬線段會累積電荷，可能損壞與其連接的電晶體閘極氧化物。天線比率定義為金屬面積與閘極面積的比值，必須低於製程規定的上限。繞線器需要在天線比例過高的線段中插入天線二極體或透過跳層繞線來「斷開」天線。

### 金屬層方向

在現代 ASIC 製程中，金屬層採用交替方向以最佳化繞線效率：

- M1：水平（標準單元 pin 存取）
- M2：垂直
- M3：水平
- M4：垂直
- M5：水平
- M6：垂直（或用作電源/時脈分配）

層級越高，金屬厚度與寬度通常越大，電阻越低，適合長距離訊號傳輸。詳細繞線器必須遵循各層的 preferred direction，在需要改變方向時透過導孔切換層級。

### 引腳存取 (Pin Access)

引腳存取是詳細繞線中最具挑戰性的問題之一。標準單元引腳通常位於 M1 層，周圍被電源軌跡 (VDD/VSS) 和其他單元內部繞線所包圍。繞線器必須：

- 從 M1 pin 向上連接到更高層的金屬。
- 避開相鄰單元的 pin 與內部繞線。
- 在有限的空間中找到可用的「逃逸路徑」(escape path)。

在先進製程中，pin 變得越來越小、越來越密集，pin accessibility 已成為繞線成敗的關鍵因素。

## 軌道分配 (Track Assignment)

詳細繞線是在一組規則的金屬軌道上進行的。軌道是以最小間距 (minimum pitch) 為單位均勻分布在金屬層上的平行線。每條軌道可以容納一條金屬導線。

軌道分配的過程：

1. **讀取全域繞線結果**：取得每條 net 在每個 GCell 內的通道容量需求。
2. **分配軌道**：將每段 wire segment 分配給該區域內一條具體的軌道。
3. **解決衝突**：若多條 wire segment 分配到同一條軌道且發生重疊，則需要使用 rip-up and reroute 或 ILP 解決。
4. **優化**：調整軌道分配以最小化導孔數、平衡各層使用率。

軌道分配問題通常可以模型化為**二部圖匹配 (bipartite matching)** 或**網路流 (network flow)** 問題。

## 繞線層級

### 下層金屬 (M1-M3)

- **M1**：主要用於標準單元內部的 pin 連接、單元之間的短距離局部連線。
- **M2-M3**：用於局部繞線與中等距離的連線。
- 特點：較窄、間距小、電阻較高，不適合長距離傳輸。

### 中層金屬 (M4-M5)

- 用於中等長度的訊號繞線。
- 寬度與間距介於下層與上層之間。
- 常用於時脈網路 (clock tree) 的部分走線。

### 上層金屬 (M6 以上)

- 用於長距離全域訊號、電源輸送網路 (PDN)、時脈分配。
- 特點：較寬、間距大、電阻低。
- 上層金屬的製造成本最高，因此使用時需要精打細算。

## 演算法方法

### 迷宮繞線 (Maze Routing)

迷宮繞線是最基礎、最通用的詳細繞線演算法。代表性演算法包括：

#### Lee 演算法

由 C. Y. Lee 於 1961 年提出，是迷宮繞線的經典方法：

1. 將繞線區域網格化（每個網格點代表一個可能的繞線位置）。
2. 從起點開始廣度優先搜尋 (BFS)，逐層向外擴張波前 (wavefront)。
3. 當波前觸及終點時，從終點回溯至起點，即為最短路徑。

Lee 演算法的優點是保證找到最短路徑，但時間與空間複雜度 O(n²) 使其在大型設計中速度較慢，且無法直接處理多層繞線。

#### A* 搜尋

A* 是 Lee 演算法的改良版本，透過啟發式函數 (heuristic function) 引導搜尋方向：

```
f(n) = g(n) + h(n)
```

- `g(n)`：從起點到當前節點 n 的實際路徑成本。
- `h(n)`：從節點 n 到終點的估計成本（通常使用曼哈頓距離）。

A* 大幅減少了需要探索的節點數量，是現代詳細繞線器中應用最廣泛的搜尋演算法。

#### Soukup 演算法

Soukup 演算法結合深度優先與廣度優先的優點，先嘗試沿著目標方向前進，遇到障礙物時再進行廣度繞行，速度通常比 Lee 演算法快，但不保證最短路徑。

### 樣式繞線 (Pattern Routing)

在詳細繞線中，樣式繞線用於處理簡單的兩端點連線：

- **L 型**：水平 + 垂直，最簡單的兩層繞線方式。
- **Z 型**：水平-垂直-水平 或 垂直-水平-垂直，用於避免阻礙。
- **U 型**：繞過障礙物的三層繞線模式。
- **C 型**：更複雜的多轉角繞線模式。

詳細繞線器通常先嘗試樣式繞線（速度快），若失敗再使用迷宮繞線做為備用方案。

### Rip-Up 與 Reroute

當繞線過程中發生衝突（例如兩條 net 分配到同一條軌道而重疊），需要解除衝突 net 的繞線並重新繞線。Rip-up 與 reroute 是詳細繞線核心的反饋機制：

1. **偵測衝突**：檢查所有軌道分配，找出違反設計規則的 net。
2. **撕除 (Rip-up)**：刪除衝突 net 的當前繞線路徑。
3. **重新繞線 (Reroute)**：為被撕除的 net 尋找替代路徑。
4. **迭代**：重複上述過程直到所有衝突解決或達到最大迭代次數。

在業界工具中，rip-up 的順序對結果品質有重大影響。常用的策略包括：

- **先入先出 (FIFO)**：先被繞線的 net 優先保留。
- **後入先出 (LIFO)**：後被繞線的 net 優先被撕除。
- **成本導向 (Cost-driven)**：按照 net 的臨界程度（如時序鬆弛）決定撕除優先順序。

## 詳細繞線的兩種主要範式

### 通道繞線 (Channel Routing)

通道繞線是經典的詳細繞線方法，主要用於閘陣列 (gate array) 或早期標準單元設計：

- 繞線在固定高度的通道（兩列單元之間的區域）內進行。
- 通道的上下邊界有 pin 需要連接。
- 經典演算法包括：
  - **Left-Edge 演算法**：按 pin 的水平位置由左至右分配軌道，簡單高效但繞線品質有限。
  - **Dogleg 繞線**：允許 net 在通道內分段，減少通道高度需求。
  - **YACR (Yet Another Channel Router)**：基於 greedy 策略的通道繞線器。

通道繞線的缺點是無法充分利用晶片面積（通道高度的浪費），且在現代密集設計中通道邊界已不復存在。

### 面積繞線 (Area Routing)

面積繞線是現代標準單元設計的主流方法：

- 繞線可以在整個晶片面積上進行，不再受限於固定通道。
- 繞線器需要處理任意形狀的障礙物（如 RAM、事先擺放的巨集單元）。
- 面積繞線更靈活，能夠更好地利用多層金屬資源。
- 通常以迷宮繞線為核心，結合樣式繞線與 rip-up and reroute。

現代 EDA 工具（如 Cadence Innovus、Synopsys IC Compiler II）中的詳細繞線器都採用面積繞線範式。

## 詳細繞線的主要階段

詳細繞線的內部流程通常可分為以下階段：

### 1. 引腳存取 (Pin Access)

此階段的目標是將每個標準單元的 M1 pin 連接到上層金屬（M2 或 M3），使其可以被繞線器存取。Pin access 是繞線流程中最關鍵也最困難的步驟，因為：

- 標準單元的 pin 密度極高（尤其是在 7nm 以下製程）。
- 相鄰 pin 之間的間距可能僅為最小間距。
- 電源軌跡 (VDD/VSS) 佔據了 M1 的大量空間。

Pin access 的輸出是每條 net 的一組「逃逸點」(escape points)，這些點位於較高金屬層上，後續繞線器使用這些點進行完整的路徑連接。

### 2. 軌道分配 (Track Assignment)

將全域繞線產生的每段 wire segment 分配給具體的軌道。這一步需要考慮：

- 各層的 preferred direction。
- 不可用的軌道（已被電源網路、時脈網路佔用）。
- 最小間距限制。
- 導孔的 alignment 要求。

軌道分配可模型化為 ILP 或 bipartite matching 問題。

### 3. 迷宮繞線 (Maze Routing)

在軌道分配完成後，使用迷宮繞線完成剩餘的詳細連接。此階段處理的是最複雜的繞線情況，需要：

- 繞過已存在的 net 與障礙物。
- 滿足所有設計規則。
- 盡量減少線長與導孔數。

### 4. 導孔插入 (Via Insertion)

當 net 需要在不同金屬層之間切換時，必須插入導孔。導孔插入需要考慮：

- 導孔的雙層或三層結構。
- 導孔周圍的金屬包覆面積。
- 導孔與相鄰導線的間距。

### 5. 繞線後最佳化 (Post-Route Optimization)

最後階段對繞線結果進行品質提升：

- **DRC 違規修復**：修正最小間距、最小寬度等違規。
- **天線效應修復**：插入天線二極體或跳層繞線。
- **冗餘導孔插入 (Redundant Via Insertion)**：在關鍵導孔旁增加備用導孔，提高良率。
- **線長優化**：在不破壞設計規則的前提下縮短多餘線段。
- **CMP 密度平衡**：調整金屬密度以滿足化學機械研磨 (CMP) 的密度要求。

## DRC (設計規則檢查)

詳細繞線完成後，必須通過完整的設計規則檢查。常見的 DRC 檢查項目包括：

- **最小間距**：同一層上兩條相鄰導線的距離。
- **最小寬度**：導線的最小寬度。
- **最小面積**：孤立金屬區塊的最小面積。
- **導孔包覆**：導孔與金屬的接觸面積。
- **導孔重疊**：不同層導孔的重疊區域。
- **金屬密度**：每單位面積的金屬覆蓋率必須在一定範圍內（用於 CMP）。
- **天線比率**：金屬面積與閘極面積的比值。

若 DRC 報告發現違規，需要回到詳細繞線階段進行修復，修復後再次執行 DRC。如此反覆直到所有違規被清除（DRC-clean）。

## 開源詳細繞線器

### TritonRoute

TritonRoute 是 OpenROAD 專案中的開源詳細繞線器，採用基於迷宮繞線的方法：

- 支援多層金屬、多種設計規則。
- 使用高效的資料結構管理網格與間距檢查。
- 內建 DRC 引擎，支援增量式 DRC 檢查。
- 與 OpenROAD 的擺置器、全域繞線器緊密整合。
- 採用 area routing 範式，支援大規模設計。

### drcU

drcU 是 OpenLANE 專案中使用的詳細繞線器，以 TritonRoute 為基礎進行了最佳化與擴充。

### VPR (FPGA 繞線)

VPR (Versatile Pack, Place, and Route) 是 VTR 專案中的 FPGA 繞線器：

- 使用 PathFinder 演算法進行協商擁塞繞線。
- 支援多種 FPGA 架構（島狀、階層式）。
- 同時處理全域與詳細繞線（在 FPGA 脈絡中兩者合一）。
- 輸出繞線資源使用率與路徑延遲的詳細報告。

## FPGA 中的詳細繞線

在 FPGA 中，詳細繞線的性質與 ASIC 不同：

### FPGA 繞線架構

FPGA 中的繞線資源是預先製造好的：

- **繞線通道 (Routing Channel)**：每個 tile 之間有固定數量的繞線通道，每個通道包含多條 wire segment。
- **可程式開關 (Programmable Switch)**：位於 switch block 中，決定 wire segment 之間的連接關係。
- **連線盒 (Connection Box)**：連接 tile 的輸入/輸出 pin 到繞線通道。

### FPGA 繞線器的工作

FPGA 繞線器需要：

1. 為每條連接選擇一組 wire segment 與 switch 的序列，建立從源端到目的端的完整路徑。
2. 確保每個 wire segment 不被多條 net 同時使用。
3. 滿足時序約束（關鍵路徑使用高速連線資源）。

### v2f-pnr 的詳細繞線實現

在 v2f-pnr 中：

- 使用 A* 搜尋在 tile 層級進行繞線，每個 tile 對應 FPGA 中的一個基本邏輯單元位置。
- 繞線路徑記錄在 ASC (ASCII place-and-route) 檔案中。
- ASC 檔案中的 wiring entries 對應到 FPGA 中特定 routing mux 的設定值。
- 這些設定值最終透過 bitstream packing 轉換為 FPGA 的配置位元流 (bitstream)。
- 每個 routing mux 的選擇相當於詳細繞線中決定使用哪條 wire segment 與哪個 switch。

因此，v2f-pnr 的 A* 繞線結果直接對應到 FPGA 繞線中最細粒度的資源分配，等同於 ASIC 中的詳細繞線。

## 軌道分配的主要演算法

### 整數線性規劃 (ILP)

將軌道分配問題公式化為 ILP：

- 變數：每個 wire segment 到每個軌道的二元分配。
- 目標：最小化總線長、導孔數。
- 約束：每條軌道在同一位置最多容納一條 wire segment；所有 wire segment 必須被分配。

ILP 保證最優解，但求解時間隨問題規模指數增長，通常只應用於小區塊。

### 網路流 (Network Flow)

將軌道分配轉換為最小成本流問題：

- 節點代表軌道位置或 wire segment 端點。
- 邊代表可能的分配。
- 容量代表每條軌道的可用空間。
- 透過最小成本最大流演算法求解。

網路流方法在效率與品質之間取得了良好的平衡。

### 二部圖匹配 (Bipartite Matching)

將問題簡化為左右兩側的匹配：

- 左側：所有的 wire segment。
- 右側：所有可用的軌道位置。
- 邊：wire segment 與軌道的相容性（位置、層級符合）。
- 目標：最大化匹配數量（即成功分配的 wire segment 數量）。

Hungarian 演算法或 Kuhn-Munkres 演算法可在多項式時間內求解。

## 現代挑戰

### 次 7nm 繞線難題

在 7nm 以下的先進製程中，詳細繞線面臨前所未有的挑戰：

- **多重圖案微影 (Multi-Patterning)**：由於光學解析度限制，單一金屬層需要拆分成多個光罩，繞線器必須考慮顏色分配 (coloring) 與衝突 (conflict)。
- **切割光罩 (Cut Mask)**：部分金屬層需要使用切割光罩來定義線端，增加了繞線的約束。
- **LELE (Litho-Etch-Litho-Etch)** 與 SADP (Self-Aligned Double Patterning) 等技術對繞線幾何形狀有嚴格限制。
- **複雜的設計規則**：先進製程的設計規則可達數千條，且存在大量相依性規則，難以同時滿足。

### 導孔電阻與電遷移

- 隨著製程微縮，導孔的電阻急劇增加，對訊號延遲與功耗產生顯著影響。
- 電遷移 (electromigration) 效應在細金屬線中更加嚴重，需要透過寬線、冗餘導孔與電流密度檢查來緩解。

### RC 萃取與模型精度

- 詳細繞線完成後需要進行 RC 萃取，以取得寄生電阻電容參數進行 STA。
- 先進製程中，三維耦合效應 (3D coupling)、邊緣效應 (fringe effect) 使得 RC 萃取變得極為複雜。
- RC 萃取的不準確可能導致 STA 結果過度樂觀或悲觀，影響時序收斂。

## 繞線閉合迴圈

詳細繞線並非一次性的流程。實際的設計流程形成一個閉合迴圈：

```
詳細繞線
    ↓
RC 萃取 (RC Extraction)
    ↓
靜態時序分析 (STA)
    ↓
有違規嗎？ ──是──→ 修復違規 (Fix Violations)
    ↓                      ↓
   否                     重新繞線 (Reroute)
    ↓                      ↓
DRC 檢查 ──有違規──→ 回到詳細繞線
    ↓
設計完成
```

這個迴圈可能需要多次迭代才能達到設計收斂。在每一步中，工程師需要分析違規報告，決定是局部修復繞線還是進行更大規模的調整。

## 參考資料

- [Wikipedia: Detailed routing](https://en.wikipedia.org/wiki/Detailed_routing)
- [Wikipedia: Electronic design automation](https://en.wikipedia.org/wiki/Electronic_design_automation)
- [Wikipedia: Place and route](https://en.wikipedia.org/wiki/Place_and_route)
- [Wikipedia: Design rule checking](https://en.wikipedia.org/wiki/Design_rule_checking)
- [Wikipedia: Maze routing](https://en.wikipedia.org/wiki/Maze_routing)
- [OpenROAD Project — TritonRoute](https://github.com/The-OpenROAD-Project)
- [VTR Project — VPR Router](https://github.com/verilog-to-routing/vtr-verilog-to-routing)
- [C. Y. Lee, "An Algorithm for Path Connections and Its Applications", IRE Trans. Electronic Computers, 1961](https://doi.org/10.1109/TEC.1961.5219222)
- [ISPD 2018/2019 Detailed Routing Contest](http://www.ispd.cc/contests/)

## 延伸閱讀

- [Place and Route (Wikipedia)](https://en.wikipedia.org/wiki/Place_and_route)
- [Routing (Electronic Design) (Wikipedia)](https://en.wikipedia.org/wiki/Routing_(electronic_design))
- [Very Large Scale Integration (Wikipedia)](https://en.wikipedia.org/wiki/Very_Large_Scale_Integration)
