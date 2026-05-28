# SAT 求解器（Boolean SAT Solver）

## 什麼是 SAT 問題

SAT（Boolean Satisfiability Problem，布林可滿足性問題）是計算理論中最經典的問題之一：給定一個布林公式，是否存在一組變數賦值（0 或 1），使該公式的結果為真？

例如，公式 `(a OR b) AND (NOT a OR c)` 是可滿足的：取 `a=1, b=0, c=1` 即可。若公式為 `(a) AND (NOT a)`，則不存在任何賦值能使結果為真，稱為不可滿足（UNSAT）。

SAT 問題雖然定義簡單，卻能編碼極複雜的組合邏輯與約束條件，使其成為 EDA 領域的核心工具。

## NP 完備性與 Cook-Levin 定理

1971 年，Stephen Cook 與 Leonid Levin 分別證明了 SAT 是 NP 完備的（NP-complete）。這意味著：

- 任何 NP 問題都可以在多項式時間內歸約為 SAT
- 如果 SAT 能在多項式時間內被解決，則所有 NP 問題都能在多項式時間內被解決（即 P = NP）
- 儘管 SAT 是 NP 完備的，現代 SAT 求解器能在實際應用中處理數百萬變數的實例

Cook-Levin 定理的證明方法是將非確定性圖靈機的運算過程編碼為一個巨大的布林公式，該公式可滿足若且唯若該圖靈機接受輸入。

## SAT 在 EDA 中的重要性

許多電子設計自動化（EDA）問題可以歸約為 SAT：

- **等價檢查（Equivalence Checking）**：證明兩個電路功能等價，可透過檢查它們的 XOR 是否不可滿足
- **模型檢查（Model Checking）**：驗證一個時序電路是否滿足某個屬性，BMC（Bounded Model Checking）將電路展開 k 個時間步後編碼為 SAT
- **自動測試向量生成（ATPG）**：將故障檢測條件編碼為 SAT 問題來尋找測試向量
- **FPGA 繞線（Routing）**：將繞線約束編碼為 SAT，尋找合法的繞線路徑
- **邏輯最佳化（Logic Optimization）**：檢查兩個邏輯表達式是否等價，或尋找冗餘邏輯

由於 SAT 求解器在過去三十年取得了巨大進展，如今 EDA 工具鏈普遍依賴高效能的 SAT/SMT 求解器。

## CNF（Conjunctive Normal Form）

SAT 求解器通常要求輸入公式為 CNF（合取範式）。CNF 由以下元素構成：

- **變數**（Variable）：布林值（0 或 1），如 a, b, c
- **文字**（Literal）：變數或其否定，如 a, ¬a, b, ¬b
- **子句**（Clause）：多個文字的析取（OR），如 `(a OR ¬b OR c)`
- **公式**：多個子句的合取（AND），如 `(a OR b) AND (¬a OR c)`

CNF 公式的範例：
```
(a + b)(a' + c)(b' + c')
```

每個括號為一個子句，+ 表示 OR，相鄰括號表示 AND，撇號表示 NOT。這個公式表達了 `(a OR b) AND (NOT a OR c) AND (NOT b OR NOT c)`。

CNF 的表達力等同於一般布林公式，任何布林公式都有 CNF 表示。

## Tseitin 轉換

將一般電路或布林公式轉換為 CNF 時，直接展開會導致子句數量指數增長。Tseitin 轉換（Tseitin Transformation）透過引入中間變數，在**線性時間**內生成一個等價可滿足（equisatisfiable）的 CNF 公式。

對每個邏輯閘引入一個新的中間變數，並添加描述該閘輸入輸出關係的 CNF 子句：

- AND 閘 `z = x AND y`：`(¬x ∨ ¬y ∨ z)(x ∨ ¬z)(y ∨ ¬z)`
- OR 閘 `z = x OR y`：`(x ∨ y ∨ ¬z)(¬x ∨ z)(¬y ∨ z)`
- NOT 閘 `z = NOT x`：`(x ∨ z)(¬x ∨ ¬z)`

Tseitin 轉換是 SAT 求解流程中不可或缺的前處理步驟，使任意電路都能被 SAT 求解器處理。

## DPLL 演算法（Davis-Putnam-Logemann-Loveland）

DPLL 於 1962 年提出，是現代 SAT 求解器的基礎框架。它是一個深度優先搜尋演算法，在 CNF 公式上遞迴地嘗試變數賦值。

### 單元傳播（Unit Propagation）

如果一個子句中只有一個文字尚未被賦值，則該文字必須為真才能使子句成立。例如子句 `(a OR b)` 中，若 a 已被賦值為 0，則 b 必須為 1。單元傳播會反覆執行直到不再有單元子句。

### 純文字消除（Pure Literal Elimination）

如果某個變數在整個公式中只以同一極性出現（全是正文字或全是負文字），則可以將該文字設為真，因為這不會使任何子句變為假。

### 決策（Decision）

從尚未賦值的變數中選一個，賦予一個值（0 或 1），進入新的決策層級（Decision Level）。

### 回溯（Backtrack）

當衝突發生（某個子句所有文字都為假），演算法回到上一個決策點，嘗試另一種賦值。如果所有賦值都試過仍衝突，則公式不可滿足。

### 完整流程

1. 執行單元傳播與純文字消除
2. 若所有子句都滿足 → SAT
3. 若存在衝突 → 回溯
4. 否則選擇一個未賦值變數，賦值後回到步驟 1

DPLL 的優勢在於單元傳播可大幅縮減搜尋空間，使 SAT 求解在實際應用中可行。

## CDCL（Conflict-Driven Clause Learning）

CDCL 於 1996 年左右被提出（Marques-Silva 與 Sakallah 的 GRASP，以及 Bayardo 與 Schrag 的 rel_sat），是 DPLL 的重大改進，也是現代 SAT 求解器能處理百萬變數問題的關鍵。

### 蘊含圖（Implication Graph）

在決策過程中，求解器記錄每個賦值的原因。蘊含圖是一個有向無環圖：

- 節點：賦值給某個變數（含其決策層級）
- 邊：從引發蘊含的文字指向被蘊含的文字
- 例如，若 `(a OR b)` 且 `a=0` 導致 `b=1`，則有 `a=0 → b=1` 的邊

衝突發生時，蘊含圖中會出現一個衝突節點（兩個相反極性的賦值指向相同變數）。

### 衝突分析（Conflict Analysis）

衝突發生時，演算法從衝突節點往回走遍蘊含圖，找到一個稱為**衝突子句**（Conflict Clause）的子集——這些是導致衝突的最小決策組合。

衝突分析的關鍵概念是**唯一蘊含點**（UIP，Unique Implication Point）：在蘊含圖中，目前決策層級上的一個節點，它是該層級中所有通往衝突節點的路徑的匯合點。通常選擇第一個 UIP（最靠近衝突的）。

### 子句學習（Clause Learning）

衝突分析得到的衝突子句會被加入子句資料庫。這防止了未來再次遇到相同的衝突賦值組合。學習到的子句類似於「記取教訓」，極大提高了搜尋效率。

學習的子句可能很多，求解器會定期進行**子句清理**（Clause Deletion），移除活躍度低的子句。

### 非時序回溯（Non-Chronological Backtracking）

傳統 DPLL 回溯到上一個決策層級。CDCL 則直接回到 UIP 所在的決策層級，跳過中間不會影響衝突的決策。例如若衝突發生於層級 7，但 UIP 在層級 4，則直接回溯到層級 4。

### 雙監聽文字（Two-Watched Literals）

高效檢測單元子句是 CDCL 的瓶頸。雙監聽文字技術為每個子句監控兩個文字，只有當這兩個文字之一被賦值為假時才檢查該子句的狀態。這使得監控成本與公式大小成亞線性關係。

### VSIDS 分支啟發式（Variable State Independent Decaying Sum）

Chaff 求解器（2001）引入的 VSIDS 是 CDCL 中最成功的決策啟發式：

- 每個文字有一個計數器，初始值為該文字在原始公式中的出現次數
- 每次衝突學習新子句時，新子句中的文字計數器增加
- 所有計數器每隔一段時間乘以一個常數（< 1）進行衰退
- 決策時選擇計數器值最高的未賦值變數

VSIDS 的改進版本 EVSIDS（Exponential VSIDS）使用指數加權，是當前主流 SAT 求解器的標準選擇。

### 重啟（Restart）

求解器定期放棄當前搜尋狀態，從頭開始，但保留已學習的子句。重啟可以避免陷入某個局部的不良搜尋空間。現代求解器使用動態重啟策略（如 Glucose 的 LBDA）。

### 蘊含圖範例

假設電路邏輯為 `(x1 OR x2) AND (¬x1 OR x3) AND (¬x3 OR x4) AND (¬x2 OR ¬x4 OR x5)`，變數賦值 `x1=0, x3=0`：

- `x1=0` 搭配子句 `(x1 OR x2)` 蘊含 `x2=1`
- `x3=0` 搭配子句 `(¬x3 OR x4)` 蘊含 `x4=1`
- `x2=1, x4=1` 搭配子句 `(¬x2 OR ¬x4 OR x5)` 蘊含 `x5=1`
- 若此時某個子句要求 `x5=0`（例如來自另一個衝突源），則發生衝突

蘊含圖中可以看到 `x1=0` 與 `x3=0` 兩條決策路徑在 `x5` 交匯，形成一個 unique implication point。

### 第一 UIP 切

第一 UIP（First UIP）是衝突分析中最常用的切法。從衝突節點往決策層級回溯，第一個遇到的 UIP 即為第一 UIP。其特性是：

- 該 UIP 在目前決策層級上是唯一的
- 由此產生的衝突子句包含該決策層級中「最少」的文字
- 通常可以回溯到較低的決策層級，跳過更多無關的決策

以圖論的觀點來看，第一 UIP 是蘊含圖中從衝突節點反向遍歷時，第一個覆蓋所有當前層級路徑的節點。

### 學習子句的修剪

學習到的子句若不節制地累積，會使資料庫膨脹，拖慢單元傳播的速度。因此現代求解器使用以下策略管理學習子句：

- **活性度量**（Activity）：記錄每個子句被用於單元傳播或衝突分析的次數
- **週期清理**：每隔 N 次衝突，刪除活性低的子句（約 50% 的學習子句）
- **大小限制**：長度大於某個閾值的子句傾向於不被學習或優先被刪除
- **Glucose 類型**：評估子句的「有用性」——若一個子句在學習後長時間未被用於單元傳播，則優先刪除

## SAT 求解器的主要元件

一個完整的 CDCL SAT 求解器包含以下元件：

- **前處理器（Preprocessor）**：在求解前簡化公式，如變數消除（Variable Elimination）、子句次級超集（Subsumption）等
- **決策啟發式（Decision Heuristic）**：決定下一個要賦值的變數及其極性
- **傳播引擎（Propagation Engine）**：執行布林約束傳播（BCP），使用雙監聽文字
- **衝突分析器（Conflict Analyzer）**：生成衝突子句並決定回溯層級
- **學習資料庫（Clause Database）**：儲存原始子句與學習子句，包含清理機制
- **重啟管理器（Restart Manager）**：決定何時重啟

## 主流 SAT 求解器

- **MiniSAT**（2003）：參考實現，體積小、效能佳，是許多後續求解器的基礎
- **Glucose**：基於 MiniSAT，加入 LBDA 重啟與子句的葡萄糖度量
- **Lingeling**：Armin Biere 開發，在許多競賽中表現優異
- **CaDiCaL**：接替 Lingeling 的新一代求解器
- **CryptoMiniSAT**：專為密碼學問題優化
- **Z3**：微軟開發的 SMT 求解器，內含高效 SAT 核心

## SAT 在 EDA 中的應用細節

### BMC（Bounded Model Checking）

BMC 由 Biere 等人於 1999 年提出，將時序電路的模型檢查問題展開為 k 個時間步的 SAT 問題：

1. 將電路的轉移關係展開 k 步：`I(s0) ∧ T(s0,s1) ∧ ... ∧ T(sk-1,sk)`
2. 加入否定屬性：`¬P(sk)`
3. 如果 SAT 求解器回傳 SAT，則找到一條違反屬性的反例（counterexample）

BMC 在硬體驗證中極其成功，尤其是與無界模型檢查（Unbounded Model Checking）結合時。

### 等價檢查（Equivalence Checking）

給定兩個電路 C1 和 C2，要證明它們功能等價：

1. 建立 Miter 電路：將 C1 和 C2 的輸出接至 XOR，XOR 的輸出為 1 時表示兩個電路不同
2. 將 Miter 電路編碼為 SAT 問題
3. 若 UNSAT，則 C1 與 C2 等價；若 SAT，則產生反例

### ATPG 作為 SAT

傳統 ATPG 演算法（如 D-Algorithm）可被 SAT 求解器取代：

1. 將故障效應（D 或 D'）編碼為變數
2. 將電路的邏輯行為編碼為 CNF
3. 加上故障活化與傳播的約束
4. SAT 求解器回傳的滿足賦值即為測試向量

### FPGA 繞線（Routing）

FPGA 繞線問題可編碼為 SAT：

- 每個繞線資源（線段、開關盒）為一個布林變數
- 繞線約束轉換為 CNF 子句
- SAT 求解器尋找合法的繞線配置

## SAT 與 SMT 的關係

SMT（Satisfiability Modulo Theories）是 SAT 的擴展，在命題邏輯的基礎上加入背景理論：

- **位元向量（Bit-Vector）**：可以直接表達固定寬度的整數運算
- **陣列（Array）**：表達記憶體讀寫的等價性
- **算術（Arithmetic）**：整數與實數的線性/非線性算術

SMT 求解器通常採用「SAT + 理論求解器」的架構，透過 DPLL(T) 框架將 SAT 與各理論的決策程序結合。

常見的 SMT 求解器包括 Z3、CVC5、Bitwuzla、Boolector（已退役）等。

### 變數消除（Variable Elimination）

前處理階段的變數消除（又稱 DP 步驟，源自 Davis-Putnam 原始演算法）透過解析（resolution）操作將變數從公式中移除：

1. 選取目標變數 x
2. 收集所有包含 x 的子句（正出現）與包含 ¬x 的子句（負出現）
3. 對每對正負子句進行解析，生成新子句
4. 將所有原始子句移除，加入新子句

如果新子句增加的總文字數少於原始子句的文字數，則變數消除有益。超過一定門檻（如原始大小的 10%）則不對該變數進行消除。

### 子句次級超集（Subsumption）

若子句 A 的文字集合是子句 B 的文字集合的超集（即 A 的文字比 B 多），且 B 已被公式包含，則 A 是多餘的，可以刪除。例如 `(a OR b)` 從屬於 `(a OR b OR c)`，因為任何滿足前者的賦值也滿足後者。

反向次級超集（Self-Subsumption）則是利用解析來縮短子句長度，進一步簡化公式。

## 決策啟發式的演進

除了 VSIDS/EVSIDS，學術界提出了多種決策啟發式：

- **Jeroslow-Wang**：根據子句長度與變數出現頻率的加權組合
- **DLIS**（Dynamic Largest Individual Sum）：選擇在未滿足子句中出現次數最多的文字
- **VMTF**（Variable Move To Front）：每次使用變數後將其移至佇列前端
- **LRB**（Learning Rate Branching）：基於變數在近期衝突中的學習率
- **Maple 系列**：結合多種啟發式，根據求解狀態動態切換

## 隨機化與重啟策略

### 隨機極性（Random Polarity）

在決策時選擇變數的極性（0 或 1）可以採用隨機方式，或偏好之前衝突中成功的極性（phase saving）。

Phase saving 是最常見的技術：記錄每個變數最後一次被賦值的極性，下次決策時優選該極性。這利用了搜尋的局部性。

### 重啟策略的類型

- **固定間隔**：每 N 次衝突重啟一次
- **幾何增長**：間隔以幾何級數增長（如 100, 200, 400, ...）
- **Luby 序列**：內建重複模式的序列，理論上保證隨機重啟最優
- **LBDA**（Learnt Batches and Delayed Average）：Glucose 使用的動態策略，根據近期學習子句的品質決定是否重啟

## 與其他求解方法的比較

### 隨機區域搜尋（SLS）

隨機區域搜尋演算法（如 WalkSAT）每次翻轉一個變數的值，嘗試減少不滿足子句的數量。SLS 在隨機產生的 SAT 實例上表現優秀，但不適用於必須證明 UNSAT 的情況（因為 SLS 無法確定不可滿足）。

### 分支與邊界（Branch and Bound）

用於 #SAT（計數可滿足賦值數量）與 MaxSAT（最大化滿足子句數量）等變體問題。

### SMT 與 SAT 的分工

SMT 求解器在內部使用 SAT 核心，但增加了理論層的推理能力。對於純命題邏輯問題，專門的 SAT 求解器通常比 SMT 求解器更快；但對於含位元向量或陣列運算的問題（如大部分 EDA 問題），SMT 求解器更方便且高效。

## 在 eda4 中的角色

eda4 的正式驗證（formal verification）流程使用 SymbiYosys（sby）作為前端，後端可接入多種 SAT/SMT 求解器：

- **Boolector**：高速 SMT 求解器，支援 QF_BV（量化自由位元向量）
- **Bitwuzla**：新一代 SMT 求解器，接替 Boolector
- **Z3**：微軟的全功能 SMT 求解器

透過 sby 與這些求解器，eda4 可以對 Verilog 設計進行 BMC 證明與屬性檢查，是確保硬體正確性的關鍵技術。

## 延伸閱讀

- [Boolean Satisfiability Problem (Wikipedia)](https://en.wikipedia.org/wiki/Boolean_satisfiability_problem)
- [SAT Solver (Wikipedia)](https://en.wikipedia.org/wiki/SAT_solver)
- [NP-Completeness (Wikipedia)](https://en.wikipedia.org/wiki/NP-completeness)
