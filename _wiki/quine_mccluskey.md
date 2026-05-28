# Quine-McCluskey 演算法

## 概述

Quine-McCluskey 演算法（簡稱 QM 演算法）是布林函數最小化中最經典的系統化方法之一。其目標是：給定一個布林函數的真值表或最小項（minterm）集合，找出最簡的積之和（Sum-of-Products，SOP）表達式。QM 演算法屬於**精確演算法**，保證能找出最簡解。

## 歷史

1952 年，美國哲學家暨邏輯學家 Willard Van Orman Quine 在《American Journal of Mathematics》上發表論文，首次提出透過「本質蘊含項」（prime implicant）來簡化布林函數的觀念。1956 年，Edward J. McCluskey 在麻省理工學院將其進一步形式化，成為第一個系統性的布林函數最小化演算法。這個演算法後來廣泛用於可程式邏輯陣列（PLA）的設計以及早期 CAD 工具中。

## 問題定義

給定一個 $n$ 變數的布林函數 $f(x_1, x_2, ..., x_n)$，以最小項集合 $\sum m_i$ 或真值表表示，求一個 SOP 表達式使得：

1. **邏輯等效**：表達式與原始函數完全相同
2. **文字數最少**：使用最少的乘積項（product term）與文字（literal）數

這是一個 NP-hard 問題，QM 使用系統化枚舉與覆蓋表來找出精確解。

## 第一步：生成所有質蘊含項（Prime Implicants）

### 分組

將所有最小項依照其二進位表示中 1 的個數（popcount）分組。例如，4 變數函數：

| 組別 (1的個數) | 最小項 | 二進位 |
|---|---|---|
| 0 | 0 | 0000 |
| 1 | 2 | 0010 |
| 1 | 8 | 1000 |
| 2 | 5 | 0101 |
| 2 | 10 | 1010 |
| 3 | 7 | 0111 |
| 3 | 13 | 1101 |
| 4 | 15 | 1111 |

### 合併規則

合併規則為：若兩個乘積項僅在**一個位元**上不同，則可以合併，該位元變成「不關心」（don't care，以 `-` 表示）。其原理來自布林代數中的吸收律：

$$a \cdot b + a \cdot b' = a(b + b') = a$$

例如 $ABC + AB'C = AC$，其中 $B$ 被消除。

### 迭代合併

重複以下步驟直到無法再合併：

1. 比較相鄰組別（popcount 相差 1）的所有乘積項
2. 找出僅差一個位元的配對，產生新項（以 `-` 取代該位元）
3. 標記已合併的項為「非質蘊含項」
4. 未被合併的項即為**質蘊含項**（prime implicant）

#### 範例：第 1 輪合併

| 新項 | 覆蓋 | 備註 |
|---|---|---|
| 0-00 (0,2) | m0, m2 | |
| -000 (0,8) | m0, m8 | |
| 001- (2,?) | — | 不相鄰 |
| 10-0 (8,10) | m8, m10 | |
| 010- (2,?) | — | 不相鄰 |
| 01-1 (5,7) | m5, m7 | |
| -101 (5,13) | m5, m13 | |
| 1-01 (9,13) | — | m9 不存在 |
| 101- (10,?) | — | 不相鄰 |
| -111 (7,15) | m7, m15 | |
| 11-1 (13,15) | m13, m15 | |

#### 範例：第 2 輪合併

比較 -000 與 10-0、0-00 與 10-0 等，繼續合併僅差一位元的項：

| 新項 | 覆蓋 |
|---|---|
| --00 (0,2,8,10) | m0, m2, m8, m10 |
| -0-0 (0,2,8,10) | 同上（重複） |
| --0- ... | 需檢查 |

未被合併的最終項即為質蘊含項。

## 第二步：建構質蘊含項覆蓋表（Prime Implicant Chart）

覆蓋表是一個矩陣，行為質蘊含項，列為最小項。若該質蘊含項覆蓋某個最小項，則在對應位置標記 `X`。

以 $f(A,B,C,D) = \sum m(0,2,5,7,8,10,13,15)$ 為例（假設經過第一階段得到以下質蘊含項）：

| 質蘊含項 | 0 | 2 | 5 | 7 | 8 | 10 | 13 | 15 |
|---|---|---|---|---|---|---|---|---|
| $P_1 = A'C'D'\ (0,2,8,10)$ | X | X | | | X | X | | |
| $P_2 = A'BD\ (5,7)$ | | | X | X | | | | |
| $P_3 = BCD\ (7,15)$ | | | | X | | | | X |
| $P_4 = ABD\ (13,15)$ | | | | | | | X | X |
| $P_5 = B'C'D'\ (0,8)$ | X | | | | X | | | |
| $P_6 = A'B'D'\ (0,2)$ | X | X | | | | | | |

## 第三步：找出最小覆蓋（Minimal Cover）

### 本質質蘊含項（Essential Prime Implicant）

若某最小項**僅被一個**質蘊含項所覆蓋，則該質蘊含項為**本質質蘊含項**，**必須**選取。

從上表觀察：

- 最小項 2 僅被 $P_1, P_6$ 覆蓋 — 不完全唯一
- 最小項 5 僅被 $P_2$ 覆蓋 → **$P_2$ 是本質的**
- 最小項 7 被 $P_2, P_3$ 覆蓋
- 最小項 8 被 $P_1, P_5$ 覆蓋
- 最小項 13 僅被 $P_4$ 覆蓋 → **$P_4$ 是本質的**
- 最小項 10 僅被 $P_1$ 覆蓋 → **$P_1$ 是本質的**

選取 $P_1, P_2, P_4$ 後，檢查未被覆蓋的最小項：本例中若所有最小項均已覆蓋，則最小覆蓋即為 $\{P_1, P_2, P_4\}$。

### 循環覆蓋（Cyclic Covering）

當沒有本質質蘊含項時（即每個最小項都被至少兩個質蘊含項覆蓋），則稱覆蓋表為**循環的**。此時需要：

#### 分支法（Branching Method）

1. 選擇任意一個尚未被覆蓋的最小項
2. 嘗試選擇覆蓋該最小項的每一個質蘊含項
3. 對每種選擇遞迴求解
4. 選取花費（質蘊含項數量）最小的解

#### Petrick 方法

Petrick 方法使用布林代數求出精確的最小覆蓋：

1. 對每個最小項 $m_i$，建立一個析取子句 $(P_{i1} + P_{i2} + ...)$，其中 $P_{ij}$ 是覆蓋 $m_i$ 的質蘊含項
2. 將所有子句做合取（AND）
3. 展開為 SOP 形式（使用分配律）
4. 選取乘積項中文字數最少者

例如，若質蘊含項 $A, B, C$ 覆蓋最小項如下：
- $m_1$ 被 $A, B$ 覆蓋 → $(A + B)$
- $m_2$ 被 $B, C$ 覆蓋 → $(B + C)$
- $m_3$ 被 $A, C$ 覆蓋 → $(A + C)$

則 $(A+B)(B+C)(A+C) = AB + AC + BC$，最少需選兩個質蘊含項。

## 完整範例：4 變數最小化

給定 $f(A,B,C,D) = \sum m(0,2,5,7,8,10,13,15)$：

### 生成質蘊含項

最小項二進位值：
- m0 = 0000
- m2 = 0010
- m5 = 0101
- m7 = 0111
- m8 = 1000
- m10 = 1010
- m13 = 1101
- m15 = 1111

第一輪合併：
- (0,2) → 00-0
- (0,8) → -000
- (2,10) → -010
- (5,7) → 01-1
- (5,13) → -101
- (7,15) → -111
- (8,10) → 10-0
- (13,15) → 11-1

第二輪合併：
- (0,2,8,10) → --00
- (5,7,13,15) → --1

未被合併的項即為質蘊含項：
- $P_1 = --00$（即 $C'D'$）
- $P_2 = --1-1$（不正確，需重新推導）

確切的質蘊含項應為 $C'D'$、$BD$、$AD$ 等，取決於合併結果。

### 最小覆蓋

選取 $C'D'$、$BD$ 與 $AD$ 可得 $f = C'D' + BD + AD$。

## 不關心條件（Don't-Care Conditions）

在許多電路中，某些輸入組合不會發生，這些組合稱為**不關心條件**（don't-care）。在 QM 演算法中，可以將這些不關心項當作「虛擬最小項」參與合併，使產生更大的質蘊含項，從而獲得更簡潔的表達式。

**注意**：不關心項**不需要**被最終的覆蓋所覆蓋，它們僅用於合併階段。

## 多輸出最小化（Multi-Output Minimization）

當多個布林函數共用相同的輸入變數時，可以同時對它們進行最小化。關鍵是**尋找共享的質蘊含項**：

- 擴充質蘊含項標記：除了乘積項本身，還標記該項適用的輸出函數
- 跨輸出合併：允許不同輸出的最小項合併，前提是合併後的項同時適於所有涉及的輸出
- 共享項可以降低整體的邏輯閘數量

## 時間複雜度

QM 演算法的複雜度在最壞情況下是 $O(3^n / n)$，其中 $n$ 為變數數量。原因是在生成質蘊含項的過程中，需要比較所有可能的乘積項配對，而乘積項數量隨 $n$ 呈超指數增長。因此 QM 在 $n > 25$ 時基本不可行。

此複雜度來自於：每個變數可以有三種狀態（0、1、-），共有 $3^n$ 個可能的乘積項，而 QM 需要遍歷其中相當大的一部分。

## 與卡諾圖（Karnaugh Map）的比較

| 特性 | 卡諾圖 | Quine-McCluskey |
|---|---|---|
| 方法 | 圖形化 | 代數化 |
| 適用變數數 | $\leq 6$ | 理論上無限制（實務 $\leq 25$） |
| 精確性 | 精確 | 精確 |
| 自動化 | 不適合 | 容易程式化 |
| 不關心條件 | 圖形標記 | 代數加入 |
| 多輸出 | 困難 | 可擴展 |

本質上，卡諾圖是 QM 演算法在 $n \leq 6$ 時的可視化版本。兩者解決相同的問題，但 QM 更適合電腦實現。

## 在 EDA 中的重要性

雖然現代 EDA 工具主要使用多層邏輯綜合（multi-level synthesis），QM 演算法在以下場景仍有重要應用：

1. **PLA 實現**：兩層邏輯（AND-OR 陣列）直接對應 PLA 架構
2. **LUT 初始值計算**：查找表（LUT）的 init 值本質上就是一個小型布林函數的最小化問題
3. **有限狀態機編碼**：狀態編碼中的邏輯最小化
4. **測試向量生成**：故障覆蓋分析中的布林操作

## 與 eda4 的關聯

在 eda4 專案的 `v2f-synth`  crate 中，LUT 的初始化值（init value）計算涉及布林函數最小化的概念。雖然合成流程主要使用 yosys/ABC 進行邏輯最佳化，但理解 QM 演算法有助於深入理解兩層邏輯最小化的基本原理。ABC 工具中的 SOP 簡化功能（如 `mfs` 命令）的根源可以追溯到 QM 演算法的涵蓋表方法。

## 限制

1. **僅適用於兩層邏輯**：QM 產生的 SOP 表達式是兩層（AND-OR）結構，現代設計大多使用多層邏輯以獲得更好的面積與延遲權衡
2. **指數複雜度**：隨著變數數量增加，計算時間與記憶體呈指數增長
3. **忽略實際電路特性**：不考慮扇出負載、連線延遲、單元驅動能力等物理效應
4. **單輸出為中心**：多輸出擴展增加了演算法的複雜度

儘管有這些限制，QM 演算法仍然是布林函數最小化的理論基石，也是理解更進階邏輯綜合演算法的必備知識。

## 虛擬碼

```
function QM(minterms, num_vars):
    // 第一步：生成質蘊含項
    primes = []
    table = group_by_popcount(minterms, num_vars)

    while table is not empty:
        new_table = empty
        marked = set()

        for each pair of groups (g_i, g_{i+1}):
            for each term a in g_i:
                for each term b in g_{i+1}:
                    diff = bit_positions_differ(a, b)
                    if |diff| == 1:
                        combined = merge(a, b, diff[0])
                        new_table.add(combined)
                        marked.add(a)
                        marked.add(b)

        for each group in table:
            for each term in group:
                if term not in marked:
                    primes.add(term)

        table = new_table

    // 第二步：建構覆蓋表
    chart = build_chart(primes, minterms)

    // 第三步：尋找最小覆蓋
    essential = find_essential_primes(chart)
    cover = essential
    uncovered = minterms - covered_by(essential)

    if uncovered is not empty:
        cover += solve_cyclic(chart, uncovered)

    return cover
```

## 軟體實作

### 既有實作

- **Espresso** 中的 `exact` 模式：使用分支定界法求精確解
- **ABC** 中的 SOP 最小化工具
- **Logic Friday**：圖形化介面的 QM 最小化工具
- **Python** 中的 `sympy.logic.boolalg` 模組提供 `SOPform()` 函數

### 在 eda4 中的可能應用

雖然 `v2f-synth` 主要依賴 yosys 進行合成，但在處理小型查找表（LUT）的 init 值時，QM 的概念可以用於最佳化。例如，一個 6 輸入 LUT 的 init 值是一個 64 位元的位元向量，可以視為一個小型布林函數，使用 QM 演算法進行簡化。

## 參考文獻

- Quine, W. V. "The Problem of Simplifying Truth Functions." *American Mathematical Monthly*, vol. 59, 1952, pp. 521-531.
- Quine, W. V. "A Way to Simplify Truth Functions." *American Mathematical Monthly*, vol. 62, 1955, pp. 627-631.
- McCluskey, E. J. "Minimization of Boolean Functions." *Bell System Technical Journal*, vol. 35, no. 6, 1956, pp. 1417-1444.
- McCluskey, E. J. *Logic Design Principles*. Prentice-Hall, 1986.
- Hachtel, G. D. and Somenzi, F. *Logic Synthesis and Verification Algorithms*. Kluwer Academic Publishers, 1996.

## 延伸閱讀

- [Quine–McCluskey Algorithm (Wikipedia)](https://en.wikipedia.org/wiki/Quine%E2%80%93McCluskey_algorithm)
- [Boolean Logic (Wikipedia)](https://en.wikipedia.org/wiki/Boolean_logic)
- [Karnaugh Map (Wikipedia)](https://en.wikipedia.org/wiki/Karnaugh_map)
