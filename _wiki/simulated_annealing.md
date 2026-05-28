# 模擬退火演算法 (Simulated Annealing) 於 FPGA 佈局最佳化

## 什麼是模擬退火

模擬退火是一種機率型全域最佳化演算法，靈感來自冶金學中的退火製程。在金屬退火中，材料被加熱到高溫後緩慢冷卻，使原子有足夠時間重新排列以達到能量最低的晶格結構。模擬退火將此物理過程抽象為數學最佳化方法，用於在大型、離散、非凸的搜尋空間中逼近全域最佳解。

在 eda4 專案的 v2f-pnr crate 中，模擬退火被應用於 FPGA 的邏輯單元佈局問題——即將經合成後的邏輯電路元件（cell）分配到晶片中特定位置的邏輯方塊（logic tile）上，以最小化線長和擁塞，同時滿足時序約束。

模擬退火的核心優勢在於其逃離局部最小值的能力。傳統的貪婪演算法一旦陷入局部最佳解便無法脫身，而模擬退火透過以一定機率接受較差的解，在早期高溫階段能廣泛探索搜尋空間，隨後在低溫階段聚焦於收斂。

## 歷史背景

模擬退火演算法由 Scott Kirkpatrick、Daniel Gelatt 和 Mario Vecchi 於 1983 年首次提出，論文發表於 Science 期刊，題為 "Optimization by Simulated Annealing"。三人當時任職於 IBM Thomas J. Watson 研究中心，他們將統計力學中的 Metropolis-Hastings 演算法應用於組合最佳化問題。

同年，捷克科學家 Václav Černý 亦獨立提出了相同的演算法，但其論文直到 1985 年才發表。因此，文獻中通常將 Kirkpatrick 等人視為主要貢獻者，而 Černý 的工作則作為獨立發現被引用。

Kirkpatrick 等人的核心洞察是認識到固體物理中的退火過程與組合最佳化問題之間存在深刻的數學類比。他們將目標函數視為系統的能量，將參數配置視為微觀狀態，並將溫度作為控制探索範圍的調節參數。此方法首次被應用於 VLSI 電路佈局問題，正是因為該問題具有大規模、離散、多局部極值的特性。

模擬退火很快成為 EDA（電子設計自動化）領域的標準工具之一，與演算法同時期發展的還有基於切割的佈局方法（min-cut partitioning）和解析佈局方法（analytical placement）。儘管後來在超大規模設計中部分被更快速的啟發式方法取代，模擬退火至今仍是評估新佈局演算法的重要基準，且在中等規模設計中仍然實用。

## 冶金退火類比

理解模擬退火需要先了解其物理靈感來源。冶金退火是金屬熱處理的一個步驟：

當金屬被加熱到熔點以上時，原子獲得足夠動能脫離晶格約束，進入隨機運動的高能狀態。此時若瞬間冷卻（淬火），原子會來不及排列成最低能量結構，導致晶格缺陷被凍結，金屬變得又硬又脆。反之，若讓金屬緩慢降溫，原子有充足的時間探索不同排列方式並逐漸趨向能量最低的穩定晶格結構，使金屬更柔韌、缺陷更少。

在模擬退火中，此類比被映射如下：

- 金屬的物理狀態對應於問題的候選解（一組元件的佈局配置）
- 系統能量對應於成本函數（wirelength + congestion penalty）
- 溫度對應於控制參數，決定接受較差解的機率
- 緩慢冷卻對應於退火排程，決定演算法何時收斂

高溫讓演算法廣泛跳躍，如同液態金屬中原子的大幅運動；低溫則限制在小範圍調整，類似固態金屬中原子只在晶格附近震盪。若降溫太快（模擬淬火），演算法會過早收斂到局部最小值，效果不佳。

## 演算法流程

模擬退火的標準流程如下：

1. 初始化：設定起始溫度 T_start，生成初始解 S（隨機或經由貪婪方法）
2. 對當前解 S 進行隨機擾動，產生鄰近解 S'
3. 計算成本變化 Δ = cost(S') - cost(S)
4. 若 Δ < 0（新解更好），直接接受 S = S'
5. 若 Δ > 0（新解較差），以機率 P = exp(-Δ / T) 接受 S = S'
6. 根據冷卻排程降低溫度 T
7. 重複步驟 2 至 6，直到滿足終止條件（T < T_end 或連續多次無改善）

接受較差解的能力是模擬退火不陷入局部最小值的關鍵。在溫度高時，exp(-Δ / T) 趨近於 1，幾乎所有移動都被接受，演算法行為類似隨機漫步。溫度降低後，exp(-Δ / T) 逐漸趨近於 0，演算法逐漸退化成貪婪爬坡。

### 接受機率公式

接受機率的數學形式源自 Metropolis 準則：

```
P(accept) = 1                     if Δ < 0 (新解較佳)
P(accept) = exp(-Δ / T)          if Δ > 0 (新解較差)
```

其中 Δ = cost(new) - cost(current)，T 為當前溫度。此公式來源於統計力學的 Boltzmann 分佈：在熱平衡狀態下，系統處於能量 E 的機率與 exp(-E / kT) 成正比。

若 Δ = 0（新舊解成本相同），接受機率為 1，即新解總是被接受。這在某些實作中是刻意的——即使成本不變，改變配置可能有助於探索。

在實作中，每次迭代產生一個均勻分佈的隨機數 r ∈ [0, 1)，若 r < P(accept) 則接受該移動。這確保了接受與否是機率性的。

### 退火排程

退火排程決定溫度如何隨迭代次數下降，對演算法行為影響極大：

#### 線性降溫
```
T = T - ΔT
```
最簡單的形式，每個迭代將溫度減去固定值。缺點是需要預先決定總迭代次數，且在低溫時降幅比例過大。在 v2f-pnr 中未採用此方法。

#### 指數降溫 (幾何降溫)
```
T = α × T
```
其中 α ∈ (0, 1) 為冷卻速率，通常設在 0.8 至 0.99 之間。每個迭代將溫度乘上一個小於 1 的常數，溫度呈指數衰減。此為最常用的排程，也是 v2f-pnr 所採用的方法。指數降溫的優點是簡單、只需一個參數，且在高溫時快速下降，在低溫時緩慢收斂。

#### 自適應排程
更進階的排程根據演算法當下的表現動態調整降溫幅度。例如：若近期接受率過高，可能表示溫度太高，應加速降溫；若接受率過低，可能溫度太低，應保持或微升。自適應排程能提升穩健性，但增加了實作複雜度。v2f-pnr 目前未使用自適應排程，而是搭配停滯檢測機制。

### 停滯檢測

單純的固定次數迭代可能浪費計算資源（若已收斂）或提早終止（若尚未收斂）。v2f-pnr 實作了停滯檢測：

若連續 5 次迭代中成本均未改善，演算法即判定已達收斂，提前終止退火過程。這與最低溫度條件並行使用——只要達到任一條件即停止。

停滯次數的閾值（5 次）是經驗值。閾值過小可能過早終止，過大則浪費計算。在 v2f-pnr 中此值可透過配置調整。

## v2f-pnr 實作細節

在 eda4 專案的 v2f-pnr crate 中，模擬退火實作的核心組件位於 `v2f-pnr/src/placer/sa_placer.rs`。以下是關鍵參數與設計決策：

### 參數設定

| 參數 | 值 | 說明 |
|------|-----|------|
| T_start | 100.0 | 起始溫度，決定初始探索範圍 |
| T_end | 1.0 | 終止溫度，低於此值停止退火 |
| cooling_rate | 0.9 | 指數冷卻因子 α，每迭代 T = 0.9 × T |
| stall_limit | 5 | 連續無改善次數上限 |
| max_iterations | 10000 | 最大迭代次數（安全上限） |

起始溫度 100.0 經過實測調整，確保在初期階段接受機率足夠高（對於典型成本範圍的移動，95% 以上會被接受）。終止溫度 1.0 則確保到後期幾乎只接受改善性移動。冷卻速率 0.9 提供較為漸進的降溫曲線。

### 鄰居選取策略

v2f-pnr 的鄰居選取採用隨機交換法：

1. 從已佔用且該位置存在邏輯單元的 tiles 集合中，均勻隨機選取兩個 tile
2. 從每個 tile 上隨機選取一個 cell
3. 交換這兩個 cell 的位置

此方法的優點是：
- 每次移動對解的擾動幅度適中
- 保證交換後兩個位置均仍然合法（tile capacity 不變）
- 實作簡單，時間複雜度 O(1)

交換完成後，僅增量式更新受影響的連線成本（HPWL），無需重新計算全域成本，大幅提升效率。

### 成本函數

v2f-pnr 的成本函數封裝於 `PlacerCost` 結構體：

```rust
struct PlacerCost {
    w_wire: f64,     // 1.0
    w_cong: f64,     // 0.5
}
```

總成本計算公式為：

```
total_cost = w_wire × HPWL + w_cong × overlap_penalty × 1000
```

其中：
- **HPWL（Half-Perimeter Wirelength）**：對每個連線網（net），計算其所連接的所有 cell 邊界的半周長——即所有 cell 的 x 座標範圍與 y 座標範圍之和。HPWL 是 FPGA 佈局中最常用的線長估計，因為它計算快速且與實際繞線長度高度相關。
- **overlap_penalty**：當超過一個 cell 被分配到同一個 tile 時，超出容量的數量乘以懲罰權重。此項確保各 tile 的 cell 數量不超過其物理容量。
- **權重比例 1.0 : 0.5 × 1000**：overlap 違規被放大 1000 倍再乘 0.5，相當於每單位 overlap 懲罰 500，確保其在總成本中佔主導地位，引導演算法在早期就消除非法重疊。

HPWL 的計算僅需遍歷每個 net 連接的 cell 集合，找 x 和 y 的極值，時間複雜度 O(net_pins)。對於 FPGA 設計而言，HPWL 雖不完美但已是線長估計的黃金標準。

### 演算法偽碼

```
procedure simulated_annealing_place(design, device):
    T = T_start = 100.0
    current = random_placement(design, device)
    current_cost = cost(current)
    best = current
    best_cost = current_cost
    stall_count = 0

    while T > T_end and stall_count < stall_limit:
        (tile_a, cell_a, tile_b, cell_b) = select_random_swap(current)
        swap(current, tile_a, cell_a, tile_b, cell_b)
        new_cost = cost(current)

        Δ = new_cost - current_cost
        if Δ < 0 or random() < exp(-Δ / T):
            current_cost = new_cost
            if new_cost < best_cost:
                best = current
                best_cost = new_cost
                stall_count = 0
            else:
                stall_count += 1
        else:
            swap_back(current, tile_a, cell_a, tile_b, cell_b)
            stall_count += 1

        T = cooling_rate × T

    return best
```

此實作採用「立即交換—評估—決定保留或還原」的模式，避免複製整個解資料結構，只需 O(1) 額外記憶體。

## 與其他佈局演算法的比較

### 基於切割的佈局 (Min-Cut Partitioning)

此方法將晶片區域反覆二分，同時將電路切割為兩個子集，最小化兩區域間的連線數量。經典演算法為 Kernighan-Lin (1970) 和 Fiduccia-Mattheyses (1982)。

優點：
- 執行速度極快（對大型設計尤明顯）
- 可平行化
- 天然可以處理區域約束

缺點：
- 分層切割的決策不可逆，早期錯誤會傳播
- 難以直接優化 HPWL
- 對遞迴深度敏感

模擬退火與之相比雖然較慢，但可以全域最佳化而不受層級限制。

### 二次佈局 (Quadratic Placement)

將線長表示為二次函數，透過求解線性方程組獲得最優解。經典實作為 GORDIAN 和 mPL 系列。

優點：
- 數學上優美，可微分
- 收斂快速，適合大規模設計
- 可結合力導向方法

缺點：
- 二次模型與實際 HPWL 有落差
- 需要後處理將連續解離散化為合法佈局
- 對離散架構（如 FPGA）較不直接

模擬退火不需要求解方程組，且直接操作於離散空間，更適合 FPGA 的 tile-based 架構。

### 力導向佈局 (Force-Directed Placement)

將 cell 之間的連線視為彈簧的拉力，求解力平衡位置。

優點：
- 直觀且可視化
- 對大規模設計擴展性佳

缺點：
- 容易陷入局部均衡
- 處理非均勻架構較複雜

模擬退火雖較慢，但在小型到中型設計中，其靈活性和實現簡潔度勝過力導向方法。

## 為何 FPGA 佈局採用模擬退火

FPGA 架構具有獨特的離散性和規律性，使模擬退火成為佈局的合理選擇：

1. **規則的 tile 陣列**：FPGA 的邏輯單元排列成規律的二維網格，離散座標系統與模擬退火的隨機交換操作完美契合。

2. **離散決策空間**：每個 cell 必須分配到整數座標的 tile 上，無連續鬆弛空間。模擬退火直接在此離散空間中操作。

3. **非凸最佳化**：FPGA 佈局的目標函數包含線長、擁塞、時序、功耗等多個相互衝突的項，構成高度非凸的搜尋空間。模擬退火的機率跳躍機制特別適合此類問題。

4. **繞線可驅動性**：相較於 ASIC，FPGA 的繞線資源更受限且更具規律性。模擬退火可將擁塞懲罰融入成本函數，引導佈局為後續繞線預留空間。

5. **實作簡潔**：不需要複雜的數學工具或線性代數庫，核心演算法約 100 行即可實作完成。對小型專案如 v2f-pnr 而言，這是最務實的選擇。

6. **可靠性**：經過數十年驗證，模擬退火的行為已被充分理解，只要調整得當，結果品質有保證。

### 缺點與限制

誠然，模擬退火也有顯著缺點：

- **收斂緩慢**：需要大量迭代才能達到良好品質，對超過十萬個 cell 的設計不實用
- **超參數敏感**：T_start、T_end、cooling_rate 等參數依賴經驗調整，在不同設計間缺乏可轉移性
- **非確定性**：隨機種子影響結果，無法保證兩次執行結果一致
- **缺乏保證**：理論上僅保證在無限慢降溫下收斂到全域最佳，實務上無法達成

在 eda4 專案中，模擬退火的設計目標是處理中小型 FPGA 設計（例如教學範例 blinky 和 adder），在可接受的執行時間內提供合理的佈局品質。

## 參考文獻

- Kirkpatrick, S., Gelatt, C. D., & Vecchi, M. P. (1983). Optimization by Simulated Annealing. Science, 220(4598), 671–680.
- Černý, V. (1985). Thermodynamical approach to the traveling salesman problem: An efficient simulation algorithm. Journal of Optimization Theory and Applications, 45(1), 41–51.
- Metropolis, N., Rosenbluth, A. W., Rosenbluth, M. N., Teller, A. H., & Teller, E. (1953). Equation of State Calculations by Fast Computing Machines. The Journal of Chemical Physics, 21(6), 1087–1092.
- Betz, V., Rose, J., & Marquardt, A. (1999). Architecture and CAD for Deep-Submicron FPGAs. Kluwer Academic Publishers.
- Sechen, C., & Sangiovanni-Vincentelli, A. (1985). The TimberWolf placement and routing package. IEEE Journal of Solid-State Circuits, 20(2), 510–522.
