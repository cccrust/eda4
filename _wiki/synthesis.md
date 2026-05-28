# 邏輯合成與技術映射

- 作者：eda4 團隊
- 對應程式碼：`verilog2fpga/v2f-synth/src/`

## 什麼是邏輯合成

邏輯合成（Logic Synthesis）是將硬體描述語言（HDL，如 Verilog、VHDL）所描述的電路行為，轉換為由基本邏輯閘（primitive cells）組成的閘級網表（gate-level netlist）的過程。這是數位電路設計流程中銜接高階設計與實體晶片／FPGA 的關鍵步驟。

在 EDA 領域中，合成器扮演的角色是：將設計師撰寫的 `assign`、`always` 等高層次結構，逐步拆解為 `AND`、`OR`、`XOR`、`NOT`、`DFF` 等閘極單元，再將這些通用邏輯對應到目標裝置（如 iCE40 FPGA）實際可用的硬體單元（如 LUT、Carry Chain、暫存器）。

eda4 專案的 `v2f-synth` crate 實現了一套完整的純 Rust 合成管線：**Tokenizer → Parser → Elaboration → Technology Mapping**，最終輸出 Yosys 相容的 JSON 格式網表，可直接饋入後續的 PNR（佈局繞線）與 Bitstream 打包流程。

合成器本身可被視為一個編譯器：輸入是 Verilog 原始碼（一種硬體描述語言），輸出是閘級網表（一種由基本邏輯閘與連線組成的圖形結構）。與軟體編譯器不同的是，合成器的輸出不是機器碼，而是描述硬體電路拓撲的結構化資料。

## 合成流程總覽

一個典型的邏輯合成流程包含以下階段：

1. **語法解析** — 將 Verilog 原始碼轉換為抽象語法樹（AST）。此階段又分為詞法分析（Tokenizer）與語法分析（Parser），前者將原始碼切割為 tokens，後者依據文法規則建構樹狀結構。

2. **展開（Elaboration）** — 將 AST 轉換為含有實際連線關係的暫存器傳輸級（RTL）網表。此階段需要解析表達式中的訊號名稱、計算常數值、展開向量為單一位元、綁定訊號寬度，並為每個位元分配唯一的識別碼。

3. **邏輯最佳化** — 對布林表達式進行化簡，移除冗餘邏輯、合併等效節點、傳播常數以減少電路面積。業界常用的技術包含卡諾圖化簡、奎因-麥克拉斯基演算法、以及 espresso 啟發式演算法。

4. **技術映射（Technology Mapping）** — 將最佳化後的通用邏輯單元綁定到目標製程或 FPGA 架構的特定單元庫。例如將一個 5 輸入布林函數對應到 iCE40 的 4-LUT 加上多工器來實現。

v2f-synth 目前實作了階段 1、2、4，而邏輯最佳化階段僅包含基本的常數折疊，尚未引入完整的布林化簡流程。

## RTL 合成

RTL（Register-Transfer Level）合成的核心任務是分析 HDL 中的 `always` 區塊，判斷設計師描述的是循序邏輯（sequential logic）還是組合邏輯（combinational logic），並分別產生對應的硬體單元。

### 敏感列表分析

`always @(posedge clk)` 中的敏感列表告訴合成器何時觸發區塊內的邏輯運算。v2f-synth 目前僅支援 `posedge` 觸發（即正緣觸發），對於 `negedge` 或混合邊緣觸發會引發執行期錯誤。合成器掃描敏感列表中的每個事件，決定是否產生 D 型正反器（DFF）。

### 狀態機提取

在進階的 RTL 合成中，合成器會嘗試找出 `always` 區塊內的有限狀態機（FSM）模式：當區塊中包含 `case` 或 `if-else` 結構，且賦值目標是 `reg` 型別時，合成器可推斷出狀態暫存器與下一狀態邏輯。v2f-synth 目前尚未實作專用的 FSM 提取器，而是將所有 `always` 區塊統一生成 DFF 單元。

### Latch 推斷

當 `always` 區塊中的組合邏輯路徑未完整覆蓋所有分支（例如 `if` 缺少 `else`），合成器會推斷出鎖存器（latch）。v2f-synth 在實作上對 `Stmt::If` 的處理是遞迴走訪 then 與 else 分支，若缺少 else 分支，則不會生成任何單元，從而在下游可能導致不完整的邏輯。

## 邏輯最小化

邏輯最小化（Logic Minimization）旨在降低布林函數的實現成本，包括減少使用的邏輯閘數量、降低邏輯層級深度、以及縮小晶片面積。

常見的化簡技術：

- **布林代數化簡**：直接套用布林代數的公理進行手動或自動化簡，例如 `A & A = A`、`A & (~A) = 0`、`A | (A & B) = A`。

- **卡諾圖（Karnaugh Map）**：一種圖形化方法，將真值表重新排列為格雷碼順序的二維表格，相鄰的 1 可以合併為較大的乘積項。適合變數數較少（小於等於 6 個）的情況。

- **奎因-麥克拉斯基演算法（Quine-McCluskey）**：系統化的布林函數最小化方法，可以處理任意數量的變數。先找出所有質含項（prime implicants），再透過涵蓋表格選擇最小的質含項集合。缺點是計算複雜度隨變數數量呈指數增長。

- **espresso 演算法**：基於啟發式最佳化的迭代簡化演算法，能在合理的時間內處理數十個變數的函數。Yosys 內部整合了 espresso，v2f-synth 目前尚未引入等效的布林化簡引擎。

## 技術映射

技術映射的目的是將通用邏輯表示對應到目標裝置的實體單元庫。對於 iCE40 FPGA 系列，主要的邏輯單元是 **邏輯單元（Logic Cell, LC）**，又稱為 **iCE40 Logic Cell（ICESTORM_LC）**。

一個 ICESTORM_LC 包含：
- 一個 4 輸入查找表（4-LUT），可實現任意 4 變數布林函數
- 一個可選的 D 型正反器
- 一個進位邏輯（Carry Chain）用於算術運算
- 一個多工器用於 LUT 合併

技術映射的核心挑戰在於：將任意 n 輸入的布林函數「包裝」到 4-LUT 中。若函數輸入數超過 4 個，則需要將其拆分為多個 LUT 並以多工器串接，這個過程稱為 **LUT 映射**。

v2f-synth 的技術映射實作在 `techmap.rs` 中，透過 `map_cell` 函數將內部 `CellKind` 列舉逐個對應到 Yosys 標準單元名稱。

## Netlist 資料結構

內部網表（Netlist）由 `netlist.rs` 中的 `Netlist` 結構體定義，是所有後續階段的共通行表示。其核心欄位包括：

- **`top: String`** — 頂層模組名稱
- **`ports: Vec<NetPort>`** — 埠列表，每個埠包含名稱、方向、以及對應的位元 ID 向量
- **`cells: Vec<Cell>`** — 所有邏輯單元，每個單元包含名稱、種類（`CellKind`）、輸入埠與輸出埠（均為 `(String, Vec<BitId>)` 對）
- **`bits: Vec<BitInfo>`** — 所有位元的資訊，包含唯一 ID、可選的名稱、是否為埠
- **`next_bit: BitId`** — 下一個可用的位元 ID（單調遞增）

`BitId` 為 `u64` 型別的全域唯一識別碼，由 `alloc_bit()` 分配。每個位元代表電路中一條單一位元寬度的連線。多位元訊號（如 `[3:0] a`）會被展開為多個 `BitId` 並以 `Signal` 結構統一管理。

## v2f-synth 管線詳解

### Tokenizer（詞法分析器）

詞法分析實作在 `parser.rs` 的 `tokenize()` 函數中。其輸入為 Verilog 原始碼字串，輸出為 `(Tok, usize)` 向量，其中 `Tok` 為 token 型別、`usize` 為該 token 在原始碼中的起始位置。

支援的 token 類別包括：
- 關鍵字：`module`、`endmodule`、`input`、`output`、`inout`、`wire`、`reg`、`assign`、`always`、`posedge`、`negedge`、`if`、`else`、`begin`、`end`、`case`、`endcase`
- 運算子：`+`、`-`、`*`、`/`、`&`、`|`、`^`、`~`、`==`、`!=`、`<`、`>`、`<=`、`>=`、`<<`、`>>`
- 數字：支援十進位（`123`）、基數格式（`4'b1010`、`8'hFF`、`32'd100`）
- 識別字：以字母或底線開頭，後接字母、數字或底線
- 特殊符號：括號、大括號、逗號、分號、冒號、點號
- 註解：`//` 行註解（不支援 `/* */` 區塊註解）

Tokenizer 在遇到無法解析的字元時會觸發 `panic`，這在處理不完整的 Verilog 子集時需要留意。

### Parser（語法分析器）

Parser 採用遞迴下降（recursive descent）解析法，實作於 `Parser` 結構體中。公開入口為 `parse_module()`，其解析順序為：

1. 消耗 `module` 關鍵字，讀取模組名稱
2. 解析埠列表（port list），處理方向宣告（`input`/`output`/`inout`）、範圍（`[msb:lsb]`）與名稱
3. 進入模組主體，依序處理：
   - `input`/`output`/`inout` 埠宣告
   - `wire` 線網宣告
   - `reg` 暫存器宣告
   - `assign` 連續賦值
   - `always` 程序區塊
   - 模組實例化（module instance）
4. 遇到 `endmodule` 時結束解析

表達式解析使用 **Pratt Parser**（優先順序解析法），將每個運算子綁定一個最小優先權（min binding power），透過 `parse_expr_bp()` 遞迴處理運算子優先順序與結合性。支援的二元運算包含算術運算（Add、Sub、Mul、Div）、位元運算（And、Or、Xor）、比較運算（Eq、Neq、Lt、Gt、Le、Ge）及移位運算（Shl、Shr）。

### Elaboration（展開）

展開階段實作於 `elab.rs`，核心資料結構為 `Ctx`，其中維護 `HashMap<String, Signal>` 將訊號名稱映射到位元向量。

`elaborate()` 函數的處理流程：

1. **埠展開**：遍歷模組的 `ports`，為每個埠的每個位元分配 `BitId`，建立 `NetPort` 並存入 `sigs` 映射表。
2. **內部訊號註冊**：遍歷 `items`，為 `wire` 與 `reg` 宣告分配位元。
3. **表達式遞迴解析**：`resolve_expr()` 將 AST 表達式節點轉換為 `Vec<BitId>`，透過模式匹配處理：
   - `Expr::Number`：逐位元檢查是 0 還是 1，生成 `Const0` 或 `Const1` 單元
   - `Expr::Ident`：從 `sigs` 映射查找訊號位元
   - `Expr::BitSel` / `Expr::Range`：從父表達式的位元向量中選取特定位元或區間
   - `Expr::Concat`：依序展開子表達式後串接
   - `Expr::Binary`：展開左右運算元，對齊寬度後逐位元生成邏輯閘
   - `Expr::Unary`：展開子表達式後逐位元處理
4. **語句處理**：對 `always` 區塊內的賦值語句生成 DFF 單元；對 `assign` 連續賦值生成傳遞邏輯。
5. **埠綁定**：為輸入埠生成 `$_INPUT_` 單元、為輸出埠生成 `$_OUTPUT_` 單元。

### Cell Generation（單元生成）

在展開過程中，v2f-synth 會根據運算類型動態生成以下主要單元：

- **`CellKind::And`** — 布林 AND，用於 `&`、連續賦值、以及寬度對齊填充
- **`CellKind::Or`** — 布林 OR，用於 `|`
- **`CellKind::Xor`** — 布林 XOR，用於 `^`
- **`CellKind::Not`** — 布林 NOT，用於 `~` 與 `-`（負號）
- **`CellKind::Add`** — 加法器，用於 `+`
- **`CellKind::Sub`** — 減法器，用於 `-`
- **`CellKind::Dff`** — D 型正反器，用於 `always @(posedge clk)` 中的賦值
- **`CellKind::Const0` / `CellKind::Const1`** — 常數 0 與常數 1 產生單元
- **`CellKind::Lut`** — 查找表（包含初始值 `init: u16`），尚未由展開階段直接生成
- **`CellKind::Carry`** — 進位鏈單元
- **`CellKind::Mux2`** — 2 選 1 多工器

值得注意的是，展開階段對賦值的處理採用 `And` 單元來實現線路連結：`ctx.net.add_cell(CellKind::And, vec![("A", vec![v]), ("B", vec![v])], vec![("Y", vec![t])])`。這相當於 `t = v & v`，在布林邏輯中等價於 `t = v`，是一種利用現有單元實現繞線連接的技巧。

### Technology Mapping（技術映射）

技術映射實作於 `techmap.rs`，主要函數 `techmap_to_json()` 接收 `&Netlist`，輸出 `HashMap<String, YosysModule>` 供序列化為 JSON。

`map_cell()` 函數負責將每個 `Cell` 映射到 Yosys 相容的單元格式，包含三個輸出：
1. **cell_type** — Yosys 標準單元名稱字串
2. **parameters** — 參數表（如 LUT 的 `LUT_INIT`）
3. **connections** — 埠連線映射（如 `"A": [1, 2, 3]`）

最終產出的 JSON 遵循 Yosys JSON 後端格式，結構為：

```json
{
  "creator": "v2f-synth v0.3",
  "modules": {
    "top": {
      "ports": { "clk": { "direction": "input", "bits": [1] } },
      "cells": {
        "$0": { "type": "$_INPUT_", "connections": { "Y": [1] } }
      },
      "netnames": { "clk": { "bits": [1], "hide_name": 0 } }
    }
  }
}
```

此 JSON 可直接被 `v2f-pnr` 讀取進行佈局繞線。

## Yosys 單元類型

v2f-synth 產生的 Yosys 相容單元類型如下：

| 內部 `CellKind` | Yosys 單元類型 | 功能 |
|---|---|---|
| `Input` | `$_INPUT_` | 頂層輸入埠 |
| `Output` | `$_OUTPUT_` | 頂層輸出埠 |
| `And` | `$_AND_` | 2 輸入 AND |
| `Or` | `$_OR_` | 2 輸入 OR |
| `Xor` | `$_XOR_` | 2 輸入 XOR |
| `Not` | `$_NOT_` | 反向器 |
| `Dff` | `$_DFF_P_` | 正緣觸發 D 型正反器 |
| `Const0` | `$_ZERO_` | 常數 0 |
| `Const1` | `$_ONE_` | 常數 1 |
| `Add` | `$_ADD_` | 加法器 |
| `Sub` | `$_SUB_` | 減法器 |
| `Mux2` | `$_MUX_` | 2 選 1 多工器 |
| `Lut` | `ICESTORM_LC` | iCE40 邏輯單元（含 LUT_INIT 參數） |
| `Carry` | `$_CARRY_` | 進位鏈 |

這些單元名稱與 Yosys 內部 `techlibs/ice40` 的定義一致，確保 v2f-synth 的輸出可直接用於 nextpnr 的後續處理。

## LUT 映射

LUT 映射是技術映射中最關鍵的環節。一個 4-LUT 可以實現任意 4 輸入的布林函數，其行為由 `LUT_INIT` 參數決定。`LUT_INIT` 是一個 16 位元的數值，對應到函數的真值表：對於輸入組合 `i3 i2 i1 i0`，輸出為 `LUT_INIT[ {i3, i2, i1, i0} ]`。

以一顆 4 輸入 AND 為例：`y = i0 & i1 & i2 & i3`，其 `LUT_INIT` 值為 `0x8000`（即只有在所有輸入皆為 1 時輸出 1）。若為區段式映射（cascade mapping），則透過將多個 LUT 搭配 MUX 串接來實現超過 4 輸入的函數。

v2f-synth 的 `CellKind::Lut { init }` 攜帶 `u16` 的初始值，經 `map_cell` 映射為 `ICESTORM_LC` 單元並將 `init` 寫入 `LUT_INIT` 參數。目前 v2f-synth 尚未實作將任意組合邏輯自動打包為 LUT 的演算法（即自動推導 `init` 值），此功能為未來開發方向。

## DFF 推斷

正反器推斷邏輯位於 `elab.rs` 的 `process_always_stmt()` 函數中。當合成器遇到 `always @(posedge clk)` 區塊內的賦值語句時，會為每個目標位元生成一個 `CellKind::Dff` 單元：

```
輸入資料 D    → DFF 的 D 埠
時脈訊號 C    → DFF 的 C 埠
輸出資料 Q    → DFF 的 Q 埠
```

處理邏輯為：
1. 展開賦值的目標表達式（`target`）與數值表達式（`value`）
2. 對逐位元對齊的每個位元對，分配一個新的 `BitId` 作為 `q`
3. 生成 `CellKind::Dff`：`D → value_bits[i]`、`Q → q`
4. 透過 `And` 單元將 `q` 接回目標位元

目前 `Dff` 單元的 `C` 埠在生成時為空向量，這是因為 v2f-synth 尚未將時脈訊號明確傳遞到 DFF 單元中。`$_DFP_P_` 的埠方向定義中包含 `"C"` 作為輸入，此處需要未來補全。

### 語句處理的遞迴結構

`process_always_stmt()` 採用遞迴方式處理 `always` 區塊內的語句結構：

- `Stmt::Nonblocking` 與 `Stmt::Blocking`：生成 DFF 單元的終端節點
- `Stmt::If`：先處理條件表達式，再分別遞迴處理 then 分支與 else 分支
- `Stmt::Block`：對區塊內的所有語句依序遞迴處理

這種處理方式對於巢狀 `if-else` 結構與 begin-end 區塊皆能正確對應。但由於 v2f-synth 不追蹤條件表達式的布林值來進行路徑收斂，對於同一個目標訊號在多個分支中被賦值的情況，可能會生成多個並列的 DFF 單元連接到同一位元上。

### 合成測試案例

`synth.rs` 內含三個測試案例，可用於驗證合成管線的正確性：

1. **`test_synth_simple_wire`**：最簡單的 `assign y = a`，驗證輸入輸出埠的正確映射。
2. **`test_synth_blinky`**：經典的 LED 閃爍電路，包含 `always @(posedge clk)` 中的計數器遞增與 DFF 生成驗證。
3. **`test_synth_adder`**：4 位元加法器，驗證多位元運算與 `$_ADD_` 單元的生成。

這些測試可透過 `cargo test -p v2f-synth` 執行。

## 常數傳播

常數傳播（Constant Propagation）是在展開表達式時，若能確定某個子表達式的值為常數，則直接以 `Const0` 或 `Const1` 單元取代，而非產生完整邏輯閘。

v2f-synth 實作於 `resolve_expr()` 對 `Expr::Number` 的處理中：對常數字面量的每個位元檢查其值為 0 或 1，分別生成對應的常數單元。這避免了「將常數傳入邏輯閘後再計算」的冗餘步驟。

但更進階的常數傳播——例如偵測 `a + 0` 應化簡為 `a`、或 `a & 1` 化簡為 `a`——目前尚未實作。這在 Yosys 中由 `opt` 系列 pass（`opt_const`、`opt_merge` 等）處理。

## 已知限制

v2f-synth 是 eda4 專案為展示純 Rust EDA 管線可行性而開發的合成器，與 yosys 等業界級工具相比有以下限制：

- **無區域最佳化**：不執行自動的電路面積與延遲最佳化，無法合併冗餘邏輯或共用子表達式。
- **無暫存器重定時（Retiming）**：無法跨暫存器邊界移動邏輯以改善時序。
- **無 FSM 提取與編碼最佳化**：不偵測有限狀態機模式，也不進行狀態編碼最佳化。
- **無正式的布林最小化**：未實作 espresso 或 Quine-McCluskey 演算法。
- **無形式驗證**：不支援等效性檢查（equivalence checking）或模型檢驗（model checking）。
- **無法處理完整 Verilog**：僅支援 Verilog-2005 撰寫風格子集，不支援 `while`、`for`、`function`、`task`、`generate`、`tri`、`supply0`/`supply1` 等結構。
- **錯誤處理不完善**：遇到不支援的語法結構時直接 `panic`，而非回報友善的診斷訊息。
- **無 DFF 時脈連線**：產生的 `$_DFF_P_` 單元缺少 C 埠連線。

## Yosys 後備方案

考量到純 Rust 合成器的功能限制，eda4 的 `v2f` CLI 工具支援多重後端模式：

- `--backend pure-rust`：強制使用 v2f-synth，適合簡單設計與示範用途
- `--backend yosys`：呼叫外部 yosys 進行合成，使用 `synth_ice40` 指令產生 JSON 輸出
- `--backend pnr-only`：跳過合成階段，直接讀取現有 JSON 網表進行 PNR
- `--backend auto`（預設）：優先嘗試 yosys，若找不到則自動回退至 v2f-synth

這種架構設計讓 eda4 既能以純 Rust 完成端到端流程（適合 CI、教學、與無外部依賴的環境），又能在需要進階合成能力時無縫銜接 yosys。

對於需要完整 Verilog 支援、多層級邏輯最佳化、或大規模設計的場景，建議使用 `--backend yosys`。而 v2f-synth 的定位在於提供一個可嵌入、無外部依賴、且能清晰展示合成原理的實作參考，同時作為 eda4 純 Rust 工具鏈的示範標竿。

### 位元 ID 與連線模型

理解 v2f-synth 的位元模型有助於掌握整個管線的設計哲學。在 `Netlist` 中，`BitId` 是連線的唯一識別碼，類似於 SPICE 網路列表中的節點編號。單元（cell）之間的連線不透過明確的 wire 物件，而是透過共享相同的 `BitId` 來隱式表示。例如，若單元 A 的輸出埠 `Y` 連接到 `BitId 5`，單元 B 的輸入埠 `A` 也連接到 `BitId 5`，則代表 A 的輸出直接饋入 B 的輸入。

這種模型簡化了網表的建構與遍歷，但代價是缺少對「連線名稱」的原生支援。為了解決這個問題，`BitInfo` 中的 `name: Option<String>` 欄位可以為位元附加可讀名稱，主要用於 JSON 輸出中的 `netnames` 區段。

### 合成輸出範例

以一個最簡單的反向器電路 `module inv(input a, output y); assign y = ~a; endmodule` 為例，v2f-synth 會產生以下 JSON 結構：

```json
{
  "creator": "v2f-synth v0.3",
  "modules": {
    "inv": {
      "ports": {
        "a": { "direction": "input", "bits": [1] },
        "y": { "direction": "output", "bits": [4] }
      },
      "cells": {
        "$0": { "type": "$_INPUT_", "connections": { "Y": [1] } },
        "$1": { "type": "$_NOT_", "connections": { "A": [1], "Y": [2] } },
        "$2": {
          "type": "$_OUTPUT_",
          "port_directions": { "A": "input", "Y": "output" },
          "connections": { "A": [2], "Y": [4] }
        }
      },
      "netnames": {
        "a": { "bits": [1], "hide_name": 0 },
        "y": { "bits": [4], "hide_name": 0 }
      }
    }
  }
}
```

三個單元分別對應輸入埠、反向邏輯、與輸出埠，透過 `BitId 1`、`2`、`4` 串聯。此 JSON 格式與 Yosys 的 `write_json` 輸出相容，可直接由 nextpnr 或 v2f-pnr 載入進行實體佈局。

### 與 Yosys JSON 格式的相容性

v2f-synth 的 JSON 輸出設計目標是與 Yosys 的 JSON 後端格式保持相容，以利於 `v2f-pnr` 無縫處理。Yosys JSON 格式的關鍵慣例包括：

- 頂層物件包含 `creator` 字串與 `modules` 物件
- 每個模組包含 `ports`、`cells`、`netnames` 三個頂層鍵
- 單元名稱以 `$` 前綴表示內部自動生成名稱
- 單元型別使用 `$_[NAME]_` 命名慣例（如 `$_AND_`、`$_DFF_P_`）
- `port_directions` 為選擇性欄位，提供給下游工具埠方向資訊
- `hide_name` 為 0 時表示該網路為使用者命名（如埠），為 1 時表示內部自動生成名稱

v2f-synth 完全遵循上述慣例，確保與既有 Yosys 生態系工具（如 nextpnr）的相容性。
