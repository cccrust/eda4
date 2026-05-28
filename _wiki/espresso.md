# Espresso 啟發式邏輯最小化演算法

## 概述

Espresso 是一個啟發式（heuristic）布林函數最小化器，由加州大學柏克萊分校的 Robert K. Brayton 等人開發。與 Quine-McCluskey 演算法不同，Espresso 不保證找到絕對最簡解，但在實務上能夠以遠高於 QM 的速度獲得接近最簡的結果。對於超過 25 個變數的大規模問題，QM 因複雜度過高而無法使用，Espresso 則能在數秒至數分鐘內完成。

## 歷史

1984 年，Brayton 等人發表了 Espresso-II 演算法，取代了先前的 MINI 和 PRESTO 等啟發式最小化器。Espresso 的名稱源自義大利濃縮咖啡，象徵其高效能。該演算法整合了「EXPAND」、「REDUCE」、「IRREDUNDANT」三個核心運算，形成一個迭代改善循環。

Espresso 隨後成為 UC Berkeley 開放原始碼 CAD 工具套件的重要元件，並被整合到更大型的邏輯綜合系統（如 MIS、SIS）中。至今，Espresso 仍然是兩層邏輯最小化的黃金標準。

## 為何需要啟發式方法

QM 演算法是精確演算法，但有以下瓶頸：

- 質蘊含項數量在最壞情況下呈 $O(3^n / n)$ 增長
- 覆蓋表的最小覆蓋問題是 NP-hard
- 對於 $n > 25$，QM 的記憶體與時間需求超出實務可行範圍

Espresso 不枚舉所有質蘊含項，而是透過迭代改善的方式，從一個初始覆蓋開始，逐步改良，因此其時間與空間需求大幅降低。

## Espresso 核心演算法

Espresso 的核心是一個迭代改善迴圈，包含三個主要運算：

```
REPEAT
    EXPAND       — 展開：擴大蘊含項，移除文字
    REDUCE       — 縮減：縮小非必要的蘊含項
    IRREDUNDANT  — 去冗：移除被覆蓋的蘊含項
UNTIL 無改善
LAST_GASP       — 最後嘗試：尋找替代覆蓋
MAKE_SPARSE     — 稀疏化：轉換為非 SOP 表示
```

### EXPAND（展開）

對每一個乘積項（cube），嘗試刪除其中的文字（literal）。刪除文字相當於在布林超立方體中擴大覆蓋範圍。展開過程需要確保：

- 展開後的乘積項**不超出** ON-set（函數值為 1 的區域）
- 展開後的乘積項**不進入** OFF-set（函數值為 0 的區域）

實作上，需要 maintain OFF-set 的補集覆蓋（complement cover）來檢查展開是否合法。對於每個文字，檢查將其移除後，新的乘積項是否與 OFF-set 相交（intersect）。若不相交，則可安全移除該文字。

展開的順序會顯著影響結果，Espresso 使用啟發式排序：優先展開「看起來最有希望」的乘積項。

### REDUCE（縮減）

將乘積項縮小，使其僅覆蓋不可被其他乘積項取代的本質最小項。

目的：將一個大的乘積項縮小後，可能釋放出空間讓其他乘積項展開得更大，從而找到更好的全局覆蓋。

縮減操作使用**超胞體差集**（sharp product，# 運算）來從乘積項中移除某些最小項：

$$A \# B = \{x \in A \mid x \notin B\}$$

即從 A 中移除所有也屬於 B 的最小項。

### IRREDUNDANT（去冗）

移除那些被其他乘積項完全覆蓋的冗餘乘積項。一個乘積項 $c$ 是冗餘的，若：

$$c \subseteq \bigcup_{d \in S, d \neq c} d$$

其中 $S$ 是當前乘積項集合。去冗可細分為：

- **本質冗餘**：可完全移除
- **部分冗餘**：可縮小後保留

IRREDUNDANT 步驟會將集合劃分為三個子集：
1. **本質的**（essential）：必須保留
2. **完全冗餘的**（totally redundant）：可移除
3. **部分冗餘的**（partially redundant）：需進一步判斷

### LAST_GASP（最後一搏）

當迭代迴圈無法再改善時，Espresso 執行「最後一搏」啟發式：

1. 對當前的覆蓋進行 REDUCE，但不完全縮到最小
2. 再次嘗試 EXPAND，期望找到新的、更好的覆蓋
3. 若找到更佳解則保留；否則維持原解

### MAKE_SPARSE（稀疏化）

傳統 SOP 將函數表示為乘積項的 OR。MAKE_SPARSE 操作嘗試將覆蓋轉換為更稀疏的形式，例如使用 XOR 分解或因式分解，以減少乘積項數量。

## 立方體運算（Cube Calculus）

Espresso 的核心資料結構是**立方體列表**（cube list）。一個立方體對應一個乘積項，以字串表示每個變數的狀態：

- `1`：該變數以原形出現（如 $A$）
- `0`：該變數以反相出現（如 $A'$）
- `-`：該變數不存在於乘積項中（don't care）

### 基本運算

#### 交集（Intersection）

兩個立方體 $a$ 與 $b$ 的交集 $a \cap b$ 定義為逐位元比較：

| a | b | 結果 |
|---|---|---|
| 0 | 0 | 0 |
| 0 | 1 | 衝突 |
| 0 | - | 0 |
| 1 | 0 | 衝突 |
| 1 | 1 | 1 |
| 1 | - | 1 |
| - | 0 | 0 |
| - | 1 | 1 |
| - | - | - |

若任何位元發生衝突（0 vs 1），則 $a \cap b = \emptyset$。

#### 包含關係（Containment）

$a \subseteq b$ 若且唯若 $a \cap b = a$。即 $a$ 的每一位元在所有不為 `-` 的位元上與 $b$ 一致。

#### Sharp 運算（#）

$a \# b$ 表示從 $a$ 中移除 $b$ 的覆蓋範圍，結果可能為多個立方體。若 $a \subseteq b$，則 $a \# b = \emptyset$。

#### 互補（Complement）

給定一個 ON-set 的覆蓋，互補運算計算其補集（即 OFF-set 的覆蓋）。這需要使用**周知遞迴**（recursive tautology check）演算法。

#### 一致性運算（Consensus）

兩個立方體的共識項（consensus）定義為：若它們在恰好一個位元上衝突（0 vs 1），則該位元設為 `-`，其餘位元取兩者的共同部分。例如：

- $a = 01-1$，$b = 0-10$ → 第三個位元衝突 → consensus = $0--0$

## 資料結構

### ON-set、OFF-set、DC-set

Espresso 維護三個立方體列表：

| 集合 | 含義 | 表示 |
|---|---|---|
| ON-set | $f = 1$ 的最小項集合 | 初始覆蓋 |
| OFF-set | $f = 0$ 的最小項集合 | 用於合法性檢查 |
| DC-set | $f = \text{don't care}$ 的最小項集合 | 可用於擴大 ON-set |

### PLA 檔案格式

Espresso 使用標準的 PLA 格式作為輸入與輸出：

```
.i 4          # 輸入變數數 = 4
.o 1          # 輸出變數數 = 1
.ilb a b c d  # 輸入變數名稱
.ob f         # 輸出變數名稱
.p 8          # 乘積項數量
0000 1        # m0 → f = 1
0010 1        # m2 → f = 1
0101 1        # m5 → f = 1
0111 1        # m7 → f = 1
1000 1        # m8 → f = 1
1010 1        # m10 → f = 1
1101 1        # m13 → f = 1
1111 1        # m15 → f = 1
.e            # 結束
```

輸出格式相同，但乘積項中的 `-` 表示該變數已被消除。

使用方式：

```
espresso input.pla output.pla
```

## 範例

假設我們有一個 4 變數函數 $f(A,B,C,D) = \sum m(0,2,5,7,8,10,13,15)$，PLA 輸入如上方所示。

Espresso 的迭代過程可能如下：

1. **初始 ON-set**：8 個乘積項，每個一個最小項
2. **EXPAND**：將 `0000` 展開為 `--00`（即 $C'D'$），覆蓋 m0, m2, m8, m10
3. **EXPAND**：將 `0101` 展開為 `-1-1`（即 $BD$），覆蓋 m5, m7, m13, m15
4. **IRREDUNDANT**：發現 `0111` 已被 `-1-1` 覆蓋，移除
5. **REDUCE**：縮小部分乘積項以嘗試更好的覆蓋
6. 最終結果可能為 $f = C'D' + BD$

輸出的 PLA 為：

```
.i 4
.o 1
.ilb a b c d
.ob f
.p 2
--00 1
-1-1 1
.e
```

## 與 Quine-McCluskey 的比較

| 特性 | Quine-McCluskey | Espresso |
|---|---|---|
| 方法 | 精確 | 啟發式 |
| 結果 | 絕對最簡 | 近似最簡 |
| 變數上限 | $\sim 25$ | $\sim 100+$ |
| 速度 | 指數增長 | 多項式級別 |
| 質蘊含項 | 全部枚舉 | 動態生成 |
| 覆蓋表 | 完整建構 | 迭代改善 |
| 多輸出 | 可擴展但複雜 | 內建支援 |

在實務上，對於大多數電路，Espresso 的結果與 QM 的最簡解相差不到 5-10%，但速度可快數千倍。

## 延伸版本

### Espresso-MV（多值邏輯）

擴展 Espresso 以處理多值輸入變數（即變數可以取多於兩個值）。這在有限狀態機的狀態編碼最佳化中特別有用。

### Espresso-EXACT

結合 Espresso 的啟發式方法與分支定界法（branch-and-bound），提供精確最小化解決方案。其流程為：

1. 使用 Espresso 的啟發式方法快速找到一個「夠好」的上界
2. 使用分支定界法搜尋精確解，用上界剪枝
3. 在中等規模的問題上可以比 QM 更有效率

### MIS / SIS 系統

MIS（Multi-level Interactive Synthesis）和其後繼者 SIS（Sequential Interactive Synthesis）是 UC Berkeley 開發的多層邏輯綜合系統。它們的核心兩層最小化引擎就是 Espresso。SIS 至今仍被許多研究團體使用。

## 在 EDA 中的重要性

Espresso 的影響遠超出兩層最小化本身：

1. **yosys/ABC 整合**：現代開源合成工具 yosys 依賴 ABC 進行邏輯最佳化。ABC 中的 SOP 約簡功能（如 `mfs`、`sop` 命令）繼承了 Espresso 的核心觀念

2. **邏輯綜合流程**：在綜合流程中，兩層最小化是技術映射（technology mapping）和 LUT 最佳化的重要子程序

3. **可程式邏輯陣列**：PLA、CPLD 等架構直接實現兩層邏輯，Espresso 是其必備工具

4. **教學與研究**：Espresso 是邏輯綜合課程的標準教材內容，其演算法架構影響了後續數十年的 CAD 研究

## 與 eda4 的關聯

在 eda4 專案中，`v2f-synth` 合成管線依賴 yosys 進行 RTL 合成與最佳化。yosys 在內部使用 ABC 進行邏輯層級的最佳化，而 ABC 的 SOP 操作深受 Espresso 演算法的影響：

- ABC 的 `sop` 命令：對節點進行 SOP 最小化
- ABC 的 `mfs` 命令：使用最小化進行節點簡化
- ABC 的 `compress` 命令：類似 Espresso 的迭代改善循環

理解 Espresso 的運作原理有助於深入掌握 yosys/ABC 的合成流程，進而在 eda4 的純 Rust 合成後端中實作類似的邏輯最佳化技術。

## 限制與注意事項

1. **啟發式本質**：Espresso 不保證全局最優，某些特定函數可能有比 Espresso 結果明顯更簡的解存在
2. **兩層邏輯限制**：與 QM 相同，Espresso 僅處理兩層 SOP 表示，現代設計需要多層邏輯綜合
3. **初始覆蓋敏感**：Espresso 的結果可能受初始覆蓋影響，不同的初始覆蓋可能導致不同的最終結果
4. **參數調校**：Espresso 有許多內部參數（如展開順序啟發式），對不同的電路可能需要調整

儘管有這些限制，Espresso 仍然是兩層布林函數最小化中最成功且影響最深遠的演算法之一。它完美地示範了如何在 NP-hard 問題上，以輕微的品質犧牲換取巨大的效能提升。

## 虛擬碼

```
function ESPRESSO(ON_set, DC_set):
    // 初始化覆蓋
    F = ON_set
    D = DC_set

    // 計算 OFF-set 補集
    R = complement(F ∪ D)

    repeat
        // EXPAND 階段
        F_expanded = EXPAND(F, R)
        cost_before = |F|

        // REDUCE 階段
        F_reduced = REDUCE(F_expanded, D)

        // IRREDUNDANT 階段
        F_new = IRREDUNDANT(F_reduced, D)
        cost_after = |F_new|

        // 更換覆蓋
        F = F_new
    until cost_after >= cost_before

    // LAST GASP
    F_last = LAST_GASP(F, D, R)
    if |F_last| < |F|:
        F = F_last

    // MAKE SPARSE
    F_sparse = MAKE_SPARSE(F)
    if |F_sparse| < |F|:
        F = F_sparse

    return F
```

## 關鍵實作細節

### EXPAND 的啟發式

EXPAND 的實作對效能影響極大，Espresso 使用以下啟發式：

1. **文字排序**：對每個乘積項中的文字排序，優先嘗試移除「最不可能造成衝突」的文字。判斷依據是該文字在 OFF-set 中的出現頻率
2. **上升下降法**（raising-lowering）：將乘積項視為布林超立方體中的一個點，逐步「上升」（將 0/1 改為 -）並檢查合法性
3. **必要性檢查**：對每個文字計算其「必要性權重」，權重低的文字優先移除

### 補集計算

OFF-set 的補集使用周知遞迴（recursive tautology check）計算：

1. 選取一個分歧變數（splitting variable），通常選擇在 ON-set 中最「平衡」的變數
2. 將 ON-set 分為 $x_i=0$ 和 $x_i=1$ 兩個子集
3. 分別遞迴計算補集
4. 合併結果

此演算法由 Brayton 在 Espresso 中提出，稱為「周知遞迴補集」（recursive complement using tautology）。

### 條件符號（Tautology Check）

判斷一個 SOP 表達式是否為永真（即恆等於 1）：

- 若某變數同時以原形和反形出現在不同乘積項中，且所有其他變數皆被覆蓋，則為永真
- 使用遞迴分歧法，對每個分歧變數檢查子集是否為永真
- 優化：若乘積項數量少於某閾值，直接展開檢查

## 在 yosys/ABC 中的實際應用

在 yosys 的合成流程中，Espresso 的概念透過 ABC 被間接使用：

### ABC 的 SOP 操作

```
sop [-a] [-l] [-m] [-n]  # SOP 最小化
mfs [-a] [-l] [-m] [-V]  # 最小化函數替代
compress [-l]             # 壓縮網路
```

這些命令都在節點層級對 SOP 表示進行最佳化，其核心演算法與 Espresso 同源。

### 在 iCE40 合成中的角色

對於 iCE40 FPGA 的 LUT 映射，yosys/ABC 執行以下步驟：

1. 將 RTL 分解為邏輯節點
2. 對每個節點進行 SOP 最小化（參照 Espresso 方法）
3. 執行技術映射到 4-LUT 或 5-LUT
4. 計算每個 LUT 的 init 值

步驟 2 直接受益於 Espresso 的高效率，使得整個合成流程可以快速完成。

## 參考文獻

- Brayton, R. K., et al. *Logic Minimization Algorithms for VLSI Synthesis*. Kluwer Academic Publishers, 1984.
- Brayton, R. K., et al. "ESPRESSO-II: A New Logic Minimizer for Programmable Logic Arrays." *IEEE Custom Integrated Circuits Conference*, 1984.
- Rudell, R. L. "Multiple-Valued Logic Minimization for PLA Synthesis." UCB/ERL M86/65, UC Berkeley, 1986.
- Sentovich, E. M., et al. "SIS: A System for Sequential Circuit Synthesis." UCB/ERL M92/41, UC Berkeley, 1992.
- Hachtel, G. D. and Somenzi, F. *Logic Synthesis and Verification Algorithms*. Kluwer Academic Publishers, 1996.
