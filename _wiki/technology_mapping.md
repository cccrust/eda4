# 技術映射 (Technology Mapping)

## 概述

技術映射 (technology mapping) 是邏輯合成 (logic synthesis) 流程中的關鍵步驟，負責將技術無關 (technology-independent) 的布林網路綁定到特定的細胞庫 (cell library) 上。輸入是一個經過最佳化的技術無關網路（由節點和邊組成的布林函數表示），輸出則是使用目標製程或架構中可用細胞實體化的網表 (netlist)。

在 ASIC 設計流程中，細胞庫包含標準細胞如 NAND2、NOR2、AOI（AND-OR-Invert）、DFF 等，每個細胞有其面積、延遲和功耗特性。在 FPGA 設計流程中，目標「細胞」是查找表 (LUT)、進位鏈 (carry chain) 和多工器 (MUX) 等可程式化邏輯元件。

## 合成流程中的位置

```
RTL (Verilog/VHDL)
  ↓
高層次合成 (HLS) / 行為合成 (optional)
  ↓
RTL 合成 ─────────────────────────────
  ├─ 邏輯最佳化 (logic optimization)  │  ── 技術無關最佳化
  │    ├─ 布林簡化 (Boolean simplification)
  │    ├─ 因式分解 (factoring)
  │    └─ 重定時 (retiming)
  ├─ 技術映射 (technology mapping) ──│  ── 此步驟
  │    ├─ 模式匹配 (pattern matching)
  │    └─ 覆蓋最佳化 (covering optimization)
  ├─ 技術相關最佳化 ──────────────────
  │    ├─ 時脈閘控 (clock gating)
  │    ├─ 電源最佳化 (power optimization)
  │    └─ 抗時序最佳化 (timing optimization)
  ↓
閘級網表 (gate-level netlist)
```

技術映射位於技術無關最佳化與技術相關最佳化之間。技術無關階段對電路進行布林操作簡化，不關心目標庫的細節；映射階段將簡化後的網路綁定到庫細胞上；技術相關階段則利用庫細胞的詳細時序與功耗模型進行最後的最佳化。

## 細胞庫 (Cell Library)

### ASIC 標準細胞庫

典型標準細胞庫包含數百至數千個細胞，每個細胞有：

| 細胞類型 | 功能 | 常用名稱 |
|---|---|---|
| 基本邏輯閘 | NAND, NOR, AND, OR, NOT | NAND2, NOR2, AND2, OR2, INV |
| 複合邏輯閘 | AND-OR-Invert, OR-AND-Invert | AOI21, OAI22, AOI222 |
| 序向元件 | D 型正反器, 鎖存器 | DFF, DFFR (含重置), DFFS (含設定) |
| 加法器細胞 | 全加器 | FA, HA |
| 多工器 | 2-to-1, 4-to-1 | MUX2, MUX4 |
| 三態緩衝器 | 三態輸出 | TBUF, BUF |

每個細胞有對應的：
- **面積**：通常以閘單位 (gate equivalents) 或平方微米表示
- **延遲**：從輸入到輸出的傳播延遲，以查找表 (NLDM) 或 CCS (Composite Current Source) 模型描述
- **功耗**：動態功耗與靜態漏電流
- **驅動強度**：不同尺寸的同一細胞提供不同驅動能力（如 INV_X1, INV_X2, INV_X4）

### FPGA 邏輯單元

FPGA 的目標「庫」則是可程式化的邏輯區塊：

| 元件 | 描述 |
|---|---|
| K-LUT (K-input LUT) | K 輸入查找表，可實現任意 K 變數布林函數 |
| 進位鏈 (carry chain) | 用於高效實現加法/減法運算的專用進位邏輯 |
| 多工器 (MUX) | 用於連線選擇和函數合成 |
| 正反器 (DFF) | 可程式化正反器，通常與 LUT 整合在同一 cell 中 |
| 記憶體區塊 (BRAM) | 大區塊 RAM，也可重構成 LUT |

**iCE40 的 ICESTORM_LC**：Lattice iCE40 系列 FPGA 的邏輯單元 (Logic Cell) 將 4-LUT、DFF 和進位邏輯整合在一個 primitive 中，可獨立或組合使用。

## DAG 覆蓋 (DAG Covering)

經典的技術映射方法由 Keutzer (1987) 提出，將問題形式化為有向無環圖 (DAG) 的覆蓋問題。

### 基本流程

1. **將電路表示為主體圖 (subject graph)** — 一個由基本閘（通常是 NAND2 和 INV）組成的 DAG
2. **將庫細胞表示為模式圖 (pattern graphs)** — 每個細胞對應一個由相同基本閘組成的小 DAG
3. **DAG 分割** — 在扇出節點 (fanout nodes) 處將 DAG 切割成樹狀子圖
4. **樹覆蓋 (tree covering)** — 對每個樹狀子圖進行最佳化覆蓋

### DAG 分割

由於樹覆蓋演算法僅適用於樹狀結構，必須在主體圖的扇出節點處進行分割：

```
輸入 DAG:
      a
     / \
    b   c
     \ /
      d   ← 扇出節點：若 d 有兩個以上輸出，則在此分割
      |
      e
```

分割後每個子樹獨立進行樹覆蓋，然後合併結果。

## 樹覆蓋 (Tree Covering)

### 主題圖與模式圖

主題圖是將電路分解為原語 (primitives) 的樹狀表示，通常使用 NAND2 和 INV 作為原語，因為任何布林函數都可以用這兩種閘實現。

模式圖是庫細胞在相同原語下的表示。例如：
```
NAND2:    INV:
  a b        a
   \/        |
   NAND      INV
```

AOI21 的模式圖：
```
    a
    |
    b   c
     \ /
     NAND
      |
      INV  ← 輸出
```

### 樹的表示：字串編碼

可以將樹節點編碼為字串，將模式匹配轉化為字串匹配問題。例如：
- 葉節點 (輸入引腳) 編碼為字母
- 內部節點編碼包含運算類型和子樹的字串

### 動態規劃覆蓋 (Tree DP)

樹覆蓋的最佳化問題可使用動態規劃求解。對於每個節點 `v`，演算法計算：
- `cost(v)`: 覆蓋以 `v` 為根的子樹所需的最小成本
- `match(v)`: 在 `v` 處匹配的庫細胞模式

**演算法**：
```
function TreeCover(node):
    for each child c of node:
        TreeCover(c)
    
    best_cost = ∞
    best_pattern = nil
    
    for each pattern p that matches at node:
        pattern_cost = cost(p) + Σ cost(children uncovered by pattern)
        if pattern_cost < best_cost:
            best_cost = pattern_cost
            best_pattern = p
    
    cost[node] = best_cost
    match[node] = best_pattern
```

**成本函數**：
- 面積面積模式：`cost(p)` = 細胞面積，目標是最小化總面積
- 延遲模式：`cost(p)` = 從輸入到輸出的延遲，目標是最小化關鍵路徑延遲
- 功耗模式：`cost(p)` = 細胞功耗，目標是最小化總功耗
- 多目標：加權組合或多階段最佳化

### 最優性保證

對於樹狀結構，給定模式的樹覆蓋問題存在多項式時間的最優解，這是因為動態規劃能夠枚舉所有可能的覆蓋方案，並利用最優子結構性質 (optimal substructure)。

## K-LUT 映射

FPGA 的技術映射問題是：將布林網路覆蓋到 K 輸入查找表 (K-LUT) 上，目標通常是最小化 LUT 數量或關鍵路徑深度。

### 問題形式化

給定一個布林網路（表示為 DAG），每個節點實現一個布林函數。K-LUT 映射將節點分組，使得每組的輸入數 ≤ K，每組對應一個 LUT。

### FlowMap 演算法 (Cong & Ding, 1994)

FlowMap 是首個能在多項式時間內找到 K-LUT 映射最優深度解的演算法。其核心思想是利用網路流 (network flow) / 最大流最小割 (max-flow min-cut) 來計算節點的「層次」(level)。

**演算法步驟**：

1. **計算層次**：對每個節點 `v`，計算 `l(v)` = 從輸入到 `v` 的最短路徑的 LUT 層數
2. **可行性測試**：對於節點 `v` 和候選層次 `l`，判斷是否存在 K-feasible cut 將 `v` 的輸入限制在 K 個以內
3. **網路流求解**：將可行性測試轉化為網路流問題，在轉換後的圖上求解最大流
4. **布林網路分割**：根據切割結果將節點分配給 LUT

**FlowMap 的關鍵貢獻**：
- 將深度最佳化問題轉化為一系列可行性測試
- 每個測試可用最大流演算法在多項式時間內求解
- 保證找到深度最優的映射結果

### 割枚舉 (Cut Enumeration)

現代 FPGA 映射器（如 ABC 中的 if 映射器）使用割枚舉來找出每個節點的所有 K-可行割 (K-feasible cuts)。

**K-可行割**：節點 `v` 的一個割是一組節點 `C`，使得從輸入到 `v` 的任何路徑都經過 `C`，且 `|C| ≤ K`。

**枚舉演算法**（使用優先遍歷和支配剪切）：
```
function EnumerateCuts(node, K):
    if node is input:
        return {{node}}  // 僅包含自身的集合
    
    cuts = {}
    for each child c of node:
        child_cuts = EnumerateCuts(c, K)
    
    // 合併子節點的割
    for each cut1 in left_child_cuts:
        for each cut2 in right_child_cuts:
            new_cut = cut1 ∪ cut2
            if |new_cut| ≤ K:
                // 檢查支配關係，去除冗餘割
                if not dominated(new_cut, cuts):
                    cuts = cuts ∪ {new_cut}
    
    return cuts
```

**優化技巧**：
- **支配剪枝**：若割 `A` 是割 `B` 的子集，則 `B` 被支配，可移除
- **優先遍歷順序**：按拓撲順序處理節點
- **割數量控制**：限制每個節點保留的割數量（如 100 個）

### 面積恢復 (Area Recovery)

深度最優映射通常會產生過多的 LUT。面積恢復階段在保持深度的前提下減少 LUT 數量：

1. **面積流 (area flow)**：估計每個節點被多少個 LUT 共享，使用啟發式成本函數
2. **重新合成 (re-synthesis)**：在深度約束下重新選擇割，優先選擇面積成本更低的割
3. **局部改造 (local transformation)**：對映射結果進行局部改造以減少 LUT

**面積流啟發式**：對每個節點 `v` 計算面積流 `af(v)`，表示實現 `v` 需要的新 LUT 數量（考慮共享）。在面積恢復階段，選擇最小化總面積流的割。

## 結構映射 vs. 布林映射

### 結構映射 (Structural Mapping)

直接使用電路的現有拓撲結構作為主題圖，在該結構上進行模式匹配和覆蓋。

**優點**：
- 演算法簡單，執行速度快
- 易於實現，適合大型設計

**缺點**：
- 受限於原始電路結構
- 可能錯過更好的實現方案

### 布林映射 (Boolean Mapping)

布林映射不局限於電路的原始結構，而是利用布林函數的代數特性（包括不關條件 don't-cares）來改變電路結構，找到更好的映射方案。

**布林匹配 (Boolean matching)**：判斷一個布林函數是否可以由某個庫細胞實現（不考慮具體結構），通常使用：
- **籤名匹配 (signature matching)**：使用函數的特性值（如真值表、沃爾什係數）快速過濾
- **NP 分類 (NPN classification)**：將函數歸類到 NPN 等價類中，每個等價類共享相同的映射模式

**不關條件利用**：
- **滿足不關 (satisfiability don't-cares, SDC)**：電路內部節點之間的布林關係產生的自由度
- **可觀測不關 (observability don't-cares, ODC)**：節點值不影響輸出的情況

### 比較

| 面向 | 結構映射 | 布林映射 |
|---|---|---|
| 執行速度 | 快 | 慢（需要布林推理） |
| 映射品質 | 依賴原始結構 | 可能更好（可重新綜合） |
| 實現複雜度 | 低 | 高 |
| 適用場景 | 大型設計快速原型 | 高效能、低功耗設計 |

## ABC 的 "if" 映射器

ABC 是 Berkeley 開發的邏輯合成與驗證工具，其 `if` (improved fanout-oriented) 映射器是當前最先進的 FPGA 技術映射器之一。

**特點**：
- 基於割枚舉的 K-LUT 映射
- 使用優先遍歷和支配剪切進行高效割枚舉
- 多階段面積恢復
- 支援多種成本函數（面積、延遲、功耗）
- 可以處理大容量設計（百萬 LUT 級別）

**使用方式**：
```
abc> read blif input.blif
abc> if -K 4  # 使用 4-LUT 映射
abc> write_blif output.blif
```

ABC 中還有其他映射指令如 `map`（經典 mapper）、`amap`（ASIC mapper）、`mfs`（mapping with free space 等）。

## Yosys 的映射流程

Yosys 是開源的 Verilog 合成工具，其技術映射流程：

1. **`synth`** 命令進行綜合與技術無關最佳化
2. **`techmap`** 命令將通用邏輯細胞 ($_AND_, $_OR_, $_MUX_ 等) 映射到目標架構的具體細胞
3. 可選的 ABC 後端：將電路傳遞給 ABC 進行進一步映射和最佳化

```
# Yosys 映射 iCE40 的典型流程
yosys> read_verilog design.v
yosys> synth -top top_module
yosys> abc -dff -D 100  # 使用 ABC 進行映射和最佳化
yosys> write_blif output.blif
```

## BLIF 格式

Berkeley Logic Interchange Format (BLIF) 是表示技術映射後網表的標準文字格式，由 Berkeley SIS 專案首創。

```
.model top
.inputs a b c
.outputs y
.names a b c y    # 組合邏輯
11- 1
1-1 1
-11 1
.latch d q        # 序向邏輯
.end
```

每個 `.names` 區塊定義一個 LUT 或組合閘，`.latch` 定義一個正反器。

## Technology Mapping in v2f-synth

v2f-synth 是專案 verilog2fpga 的純 Rust 合成引擎，採取了簡化的技術映射方法。

### CellKind 列舉

v2f-synth 使用 `CellKind` 列舉表示邏輯細胞類型，直接對應抽象閘級操作：

```rust
pub enum CellKind {
    And,
    Or,
    Xor,
    Not,
    Mux,
    Add,
    Dff,
    IcestormLc,
    // ... 及其他
}
```

### 映射策略

與傳統技術映射不同，v2f-synth **不進行獨立的割枚舉或樹覆蓋步驟**。它從 Verilog 解析得到的抽象語法樹 (AST) 直接生成細胞級網表，然後將抽象細胞映射到目標架構的 primitive：

| v2f-synth 抽象細胞 | Yosys 等效細胞 | iCE40 Primitive |
|---|---|---|
| `And` / `Or` / `Xor` | `$_AND_` / `$_OR_` / `$_XOR_` | ICESTORM_LC (LUT 模式) |
| `Mux` | `$_MUX_` | ICESTORM_LC (MUX 模式) |
| `Add` | `$add` | ICESTORM_LC (carry 模式) |
| `Dff` | `$_DFF_P_` | ICESTORM_LC (DFF 模式) |

這種方法更接近於「直接細胞產生」(direct cell generation) 而非傳統的 DAG 覆蓋。它在品質上不及 ABC 的 LUT 映射（無法進行多層邏輯重組和面積-延遲權衡），但好處是實現簡單、可預測性高、無外部依賴。

### ICESTORM_LC 的映射

ICESTORM_LC 是 iCE40 的邏輯單元 primitive，可以配置為四種模式：

1. **LUT 模式**：實現任意 4 輸入布林函數
2. **DFF 模式**：作為 D 型正反器使用
3. **Carry 模式**：作為進位鍵的一環
4. **Pass-through 模式**：作為簡單的緩衝器

當 v2f-synth 遇到一個 4 輸入以內的布林函數時，它可以直接產生一個 LUT 模式的 ICESTORM_LC。對於 5 輸入以上的函數，則需要手動分解。

### 與 ABC 映射的比較

| 面向 | v2f-synth 直接映射 | ABC / FlowMap |
|---|---|---|
| 演算法類型 | 直接細胞產生 | 基於割的 DAG 覆蓋 |
| 多層邏輯重組 | 無 | 有（可改變拓撲結構） |
| 面積最佳化 | 有限 | 多階段面積恢復 |
| 延遲最佳化 | 無 | FlowMap 最優深度 |
| LUT 利用率 (4-LUT) | 低（函數＞4輸入需手動分解） | 高（自動分解） |
| 執行速度 | 快 | 中等（割枚舉有開銷） |
| 外部依賴 | 無（純 Rust） | 需 C 編譯的 ABC |

## 參考文獻與相關條目

- [Technology mapping on Wikipedia](https://en.wikipedia.org/wiki/Technology_mapping)
- [Logic synthesis on Wikipedia](https://en.wikipedia.org/wiki/Logic_synthesis)
- [FPGA on Wikipedia](https://en.wikipedia.org/wiki/Field-programmable_gate_array)
- [Standard cell on Wikipedia](https://en.wikipedia.org/wiki/Standard_cell)
- [ABC (logic synthesis) on Wikipedia](https://en.wikipedia.org/wiki/ABC_(logic_synthesis))
- [BLIF on Wikipedia](https://en.wikipedia.org/wiki/Berkeley_Logic_Interchange_Format)
- [Yosys on GitHub](https://github.com/YosysHQ/yosys)
- [ABC on GitHub](https://github.com/berkeley-abc/abc)
- [EDA Wiki Index](/Users/ccc/Desktop/ccc/project/eda4/_wiki/index.md)
- [Synthesis](/Users/ccc/Desktop/ccc/project/eda4/_wiki/synthesis.md)
- [FPGA](/Users/ccc/Desktop/ccc/project/eda4/_wiki/fpga.md)
- [Standard Cell](/Users/ccc/Desktop/ccc/project/eda4/_wiki/standard_cell.md)
- [ASIC](/Users/ccc/Desktop/ccc/project/eda4/_wiki/asic.md)

## 延伸閱讀

1. Keutzer, K. (1987). "DAGON: Technology binding and local optimization by DAG covering". *24th ACM/IEEE Design Automation Conference*.
2. Cong, J. and Ding, Y. (1994). "FlowMap: an optimal technology mapping algorithm for delay optimization in lookup-table based FPGA designs". *IEEE Transactions on Computer-Aided Design of Integrated Circuits and Systems*, 13(1), 1-12.
3. De Micheli, G. (1994). *Synthesis and Optimization of Digital Circuits*. McGraw-Hill.
4. Mishchenko, A., Chatterjee, S., and Brayton, R. (2006). "DAG-aware AIG rewriting: a fresh look at technology independent synthesis". *Proceedings of the 43rd ACM/IEEE Design Automation Conference*.
5. Mishchenko, A., Cho, S., Chatterjee, S., and Brayton, R. (2007). "Combinational and sequential mapping with priority cuts". *IEEE/ACM International Conference on Computer-Aided Design*.
6. Brayton, R. and Mishchenko, A. (2010). "ABC: An Academic Industrial-Strength Verification Tool". *Proceedings of the 22nd International Conference on Computer Aided Verification*.
