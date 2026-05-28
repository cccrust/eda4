# 最佳樹覆蓋 (Optimal Tree Covering)

## 概述

最佳樹覆蓋（Optimal Tree Covering）是一種基於動態規劃的演算法，用於技術映射（technology mapping）階段，目標是從一個給定的 Boolean 電路樹狀結構中，找出使用函式庫單元（library cells）實作時成本最小的方式。這個問題由 Kurt Keutzer 在 1987 年的 DAGON 系統中首次正式提出並求解，成為後續所有合成工具中技術映射的核心理論基礎。

## 問題背景

### 技術映射 (Technology Mapping)

在數位電路合成的流程中，技術映射是將邏輯最佳化後、與製程無關（technology-independent）的 Boolean 網路，對應到特定製程的標準元件函式庫（standard cell library）上的步驟。輸入是一個由基本邏輯閘（如 NAND2、INV）組成的**主體圖**（subject graph），輸出是一個由函式庫單元組成的網表。

### DAG 覆蓋法 (DAG Covering)

實際電路是有向無環圖（DAG），而非單純的樹。DAG 覆蓋法的策略是：

1. **樹分解**：將電路 DAG 在扇出節點（fanout nodes）處切開，切成一組樹（forest of trees）。
2. **逐樹覆蓋**：對每棵樹獨立執行最佳樹覆蓋演算法。
3. **組合結果**：將各樹的覆蓋結果合併為完整電路。

這種分解犧牲了全域最佳性——因為扇出節點處的邏輯無法跨樹共享——但換來了可處理的計算複雜度。

### 主體圖 (Subject Graph)

主體圖是技術映射的輸入電路表示。典型做法是將原始電路分解為規範形式（canonical form），例如：

- **NAND2 分解**：所有邏輯閘轉換為 2-input NAND 和反向器。
- **AND-INV 圖 (AIG)**：使用 AND 閘和反向器的組合。

規範化分解確保函式庫中的任何模式（pattern）都能在主體圖中找到結構對應。

## 樹的定義與分解

### 樹的數學定義

此處的「樹」指有根樹（rooted tree），每個節點有一個輸出和任意數量的輸入（扇入），但只有一個扇出（fanout = 1）。在電路語境中：

- **內部節點**：代表邏輯閘（如 NAND2、NOR2）。
- **葉節點**：代表電路輸入（primary inputs）或前級扇出節點。
- **根節點**：代表電路輸出（primary outputs）或後級扇出驅動點。

### 扇出分解 (Fanout Decomposition)

在扇出節點處切斷電路會產生兩種後果：

1. **邏輯複製**：扇出節點在多棵樹中重複出現，造成面積成本估算誤差。
2. **全域最佳性喪失**：各樹獨立覆蓋，無法跨樹邊界進行最佳化。

這是樹覆蓋方法的根本限制，也是後續 DAG-aware 技術映射演算法試圖改進的重點。

## 模式函式庫 (Pattern Library)

函式庫中的每個單元（cell）被表示為一棵**模式樹**（pattern tree），結構與主體圖使用相同的基元（如 NAND2 + INV）。

### 範例：AOI21 單元的模式表示

一個 AOI21 閘（AND-OR-INVERT，2-input AND 驅動 NOR 的其中一個輸入）的模式樹為：

```
      AOI21
      /    \
    NAND2  INV
    /    \
   A     B
```

其中 A、B、C 為外部輸入接腳。模式樹的葉節點是函式庫單元的輸入接腳。

### 模式庫建構

模式樹通常由函式庫自動產生：

- 對每個函式庫單元，使用相同的基本基元（如 NAND2、INV）描述其內部結構。
- 每個單元可有多種模式表示（例如，將 INV 串接視為 buffer）。
- 模式庫的規模通常為數十至數百個模式。

## 動態規劃演算法 (Keutzer 1987)

### 符號定義

- **主體樹 T**：由基本節點構成的樹。
- **模式庫 L**：一組模式樹 {P1, P2, ..., Pm}。
- **成本函數 cost(P)**：使用模式 P 的面積（或其他度量）成本。
- **cost(v)**：以節點 v 為根的子樹的最低覆蓋成本。

### 自底向上 DP (Bottom-up Dynamic Programming)

對主體樹進行後序遍歷（post-order traversal），對每個節點 v：

```
對於每個在節點 v 處匹配的模式 P：
    令 P 的輸入接腳對應到主體樹中的子節點 v1, v2, ..., vk
    total_cost = cost(P) + sum(cost(v_i) for i=1..k)
選擇 total_cost 最小的模式 P*
記錄 cost(v) = total_cost*
記錄 best_match(v) = P*
```

### 成本傳播

- **葉節點**：cost(leaf) = 0（直接接線，無需邏輯閘），或使用輸入緩衝器成本。
- **內部節點**：依照上述公式計算。
- **根節點**：cost(root) 即為整棵樹的技術映射最低成本。

### 自頂向下回溯 (Top-down "Best Match" Traversal)

DP 完成後，從根節點開始，沿著 `best_match` 指標輸出所選的函式庫單元，直到抵達葉節點：

```
function emit_mapping(node):
    P = best_match(node)
    輸出模式 P 對應的函式庫單元
    for each input i of P:
        emit_mapping(corresponding_subject_node)
```

這保證了最終電路由函式庫單元合法組成，且總成本最小。

## 匹配演算法 (Matching Algorithms)

### 字串匹配 (String Matching)

將主體樹和模式樹線性化為字串（如前綴表示法 prefix notation），然後使用 **Aho-Corasick** 多模式字串匹配演算法，可在 O(n + m + total_matches) 時間內完成匹配，其中 n 為主體樹節點數，m 為模式庫大小。

- 優點：速度快，適合大規模模式庫。
- 缺點：需要樹的深度優先遍歷順序一致，對對稱輸入（如 NAND2 的兩個輸入互換）需預先正規化。

### 樹自動機 (Tree Automaton)

對模式庫建構一個確定性樹自動機（deterministic tree automaton），在主體樹上運行一次即可找出所有匹配。

- 狀態表示模式匹配的局部進度。
- 對每個節點，根據其子節點的狀態和當前節點類型，計算新狀態。
- 適合大規模生產環境，效率高。

### 結構匹配 (Structural Matching)

直接將主體樹子樹與每個模式樹進行同構比較（isomorphism check）。

- 對每個節點嘗試所有模式，最壞情況 O(|T| * |L| * size(T))。
- 優點：實作簡單，易於除錯。
- 缺點：對大模式庫效率較低，通常只用於小規模或用於產生 gold reference。

## 成本函數 (Cost Functions)

### 面積成本 (Area Cost)

最簡單的成本度量，定義為函式庫單元的電晶體計數或幾何面積：

```
cost(P) = transistor_count(P)
```

適用於面積最小化的應用。對於以 NAND2/INV 為主體的函式庫，每種單元有固定的面積係數。

### 延遲成本 (Delay Cost)

使用非線性延遲模型（nonlinear delay model），如：

```
arrival_time(v) = max(arrival_time(v_i)) + d(P, load)
```

其中 d(P, load) 為模式 P 在給定輸出負載下的本質延遲。負載由後級電路的輸入電容決定。

延遲最佳化的樹覆蓋比面積最佳化複雜得多，因為：

- 負載依賴性：選擇的模式影響輸出電容，進而影響前級延遲。
- 全域耦合：各樹的延遲透過時序約束耦合。

### 功率成本 (Power Cost)

動態功率由切換活動（switching activity）與負載電容的乘積決定：

```
power_cost(P) = switching_activity * load_capacitance(P)
```

在低功耗設計中，可與面積或延遲組成多目標成本函數。

### 面積-延遲乘積 (Area-Delay Product)

```
cost(P) = area(P) * delay(P)
```

用於平衡面積與效能的權衡。

## 最佳性保證 (Optimality Guarantee)

**定理**：對於一棵樹，在給定的模式集合和可加性成本函數下，上述動態規劃演算法保證找到**全域最佳**的覆蓋（即最小成本解）。

**證明要點**：
- 樹的子結構最佳性質：子樹的最佳覆蓋是全域最佳覆蓋的一部分。
- DP 枚舉了每個節點的所有可能模式匹配，不會遺漏可能解。
- 成本函數的可加性保證總成本可分解為子問題成本的總和。

**對 DAG 的限制**：當電路包含扇出時，樹分解破壞了最佳性保證。扇出節點的邏輯可能需要在多棵樹中重複計算，但 DP 無法感知這種重複。

## 限制與不足

### 扇出節點問題

扇出節點（即一個訊號驅動多個負載）產生非樹結構。標準樹覆蓋在這些節點處切斷電路，導致：

- 無法利用邏輯共享（logic sharing）。
- 面積成本可能被高估或低估。
- 跨樹的時序最佳化受限。

### Boolean don't-cares 未利用

樹覆蓋是結構性的（structural），僅比較圖的拓撲結構，不考慮 Boolean 等價性（即 `x & x = x`、`x | ~x = 1` 等化簡規則）。這表示：

- 若主體電路包含冗餘結構，樹覆蓋無法消除。
- 需要前置的 Boolean 最佳化步驟來精簡電路。

### 匹配預設結構依賴

若原始電路的分解方式不同（如使用 AND2 而非 NAND2 分解），匹配結果可能完全不同。樹覆蓋對主體圖的建構方式敏感。

### 延遲最佳化的不足

標準面積最佳化的 DP 對延遲最佳化不充分，因為：

- 延遲是路徑敏感的（path-sensitive），非可加性成本在 load-dependent 模型中難以分解。
- 需要 wavefront DP 或基於 cutting 的演算法（如 FlowMap）來處理。

## DAGON 系統 (Keutzer 1987)

Kurt Keutzer 在 UC Berkeley 開發的 DAGON 系統是第一個完整實現樹覆蓋技術映射的工具，其貢獻包括：

- **問題形式化**：首次將技術映射表述為樹覆蓋問題。
- **字串匹配加速**：使用 Aho-Corasick 演算法進行多模式匹配。
- **面積最佳化**：以函式庫單元的面積為成本進行 DP。
- **實用性驗證**：在實際工業設計上驗證了樹覆蓋的效果。

DAGON 的論文發表在 25th ACM/IEEE Design Automation Conference (1987)，題為 "DAGON: Technology Binding and Local Optimization by DAG Matching"，引用次數超過 1000 次，是 EDA 領域的經典文獻。

## 與 v2f-synth 的關係

v2f-synth 是 eda4 專案中的純 Rust Verilog 合成器，目前採取直接將電路展開為 Yosys 相容單元的策略，**不執行獨立的樹覆蓋技術映射**。理解樹覆蓋的意義在於：

- **背景知識**：Yosys 的 `techmap` 和 ABC 的 `if` 指令內部使用樹覆蓋與其變體。
- **未來擴展**：若 v2f-synth 需要支援特定 FPGA 裝置的原生技術映射，可參考樹覆蓋方法實作匹配引擎。
- **性能對比**：樹覆蓋提供了一個理論最佳解作為基準，用於評估啟發式演算法的品質。

## FlowMap：LUT 映射的樹覆蓋擴展

FlowMap (Cong & Ding, 1994) 將樹覆蓋的概念擴展到 LUT 型 FPGA 的技術映射：

- **K-feasible cut**：將 Boolean 網路劃分為 K 輸入函數，每個函數可放入一個 K-input LUT。
- **網路流演算法**：使用最大流/最小割（max-flow/min-cut）找出最佳 cut。
- **深度最佳化**：在面積與延遲之間權衡，保證延遲最小。

FlowMap 發表於 FPGA '94，是 FPGA 技術映射領域的里程碑。

## Binate 覆蓋問題 (Binate Covering)

當成本函數同時包含正反形式（true and complemented forms）時，問題退化為**雙元覆蓋**（binate covering problem），這是 NP-hard 問題。樹覆蓋透過以下方式避開這個困難：

- 主體圖使用統一基元（如 NAND2 + INV），不區分正反。
- 匹配過程只考慮結構同構，不涉及 Boolean 極性選擇。
- 極性問題由技術映射後的反向器最佳化（inverter optimization）處理。

## 實際應用範例

### 範例：NAND2 分解樹的覆蓋

考慮一棵由 NAND2 和 INV 組成的樹：

```
輸出 o = NAND2(NAND2(a, b), INV(c))
```

假設函式庫中有一個 **AOI21** 模式，其結構為 `NAND2(NAND2(a, b), INV(c))`，面積成本為 4。

替代方案是使用三個獨立單元：

- NAND2 (a, b) → 成本 2
- INV (c) → 成本 1
- NAND2 (上兩者的輸出) → 成本 2
- 總成本 = 5

DP 會選擇成本 4 的 AOI21 方案。

### 範例：多種匹配選擇

若某節點同時匹配 NAND2 模式和更複雜的 AND-OR 模式，DP 會比較兩者（及其子樹成本的總和）後選出最優者。這使得函式庫中複雜單元（如 OAI、AOI、MUX）能被正確選用。

## 參考文獻

- Keutzer, K. "DAGON: Technology Binding and Local Optimization by DAG Matching." *24th ACM/IEEE Design Automation Conference*, 1987. https://en.wikipedia.org/wiki/DAGON_(software)
- Keutzer, K. "Technology Binding and Local Optimization by DAG Matching." https://en.wikipedia.org/wiki/Technology_mapping
- Cong, J. and Ding, Y. "FlowMap: An Optimal Technology Mapping Algorithm for Delay Optimization in Lookup-Table Based FPGA Designs." *IEEE Transactions on Computer-Aided Design*, 1994. https://en.wikipedia.org/wiki/FlowMap
- De Micheli, G. "Synthesis and Optimization of Digital Circuits." McGraw-Hill, 1994. https://en.wikipedia.org/wiki/Logic_synthesis
- Hachtel, G. D. and Somenzi, F. "Logic Synthesis and Verification Algorithms." Springer, 2006. https://en.wikipedia.org/wiki/Logic_synthesis
- Aho, A. V. and Corasick, M. J. "Efficient String Matching: An Aid to Bibliographic Search." *Communications of the ACM*, 1975. https://en.wikipedia.org/wiki/Aho%E2%80%93Corasick_algorithm
- Brayton, R. K. et al. "MIS: A Multiple-Level Logic Optimization System." *IEEE Transactions on Computer-Aided Design*, 1987. https://en.wikipedia.org/wiki/Logic_synthesis
- https://en.wikipedia.org/wiki/Tree_automaton
- https://en.wikipedia.org/wiki/Dynamic_programming
- https://en.wikipedia.org/wiki/Boolean_dont-care
- https://en.wikipedia.org/wiki/Standard_cell_library

## 延伸閱讀

- [Dynamic Programming (Wikipedia)](https://en.wikipedia.org/wiki/Dynamic_programming)
- [Tree (Data Structure) (Wikipedia)](https://en.wikipedia.org/wiki/Tree_(data_structure))
- [Boolean Function (Wikipedia)](https://en.wikipedia.org/wiki/Boolean_function)
