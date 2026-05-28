# Netlist 與 Yosys-JSON 格式

## 什麼是 Netlist

Netlist（網路表）是數位電路的圖論表示法。相較於 RTL（Register Transfer Level）以行為層級描述電路功能，netlist 則以元件（cell/gate）和連線（wire/net）的拓撲結構來描述電路。每個節點代表一個邏輯閘或基本單元，每條邊代表一條連接這些元件的導線。

在 EDA 流程中，netlist 是合成（synthesis）階段的輸出，同時也是後續佈局（placement）與繞線（routing）階段的輸入。它處於抽象行為描述與具體物理實現之間的核心位置。

Netlist 有兩種常見的表示形式：文字格式（如 Verilog netlist、EDIF）與記憶體中的資料結構（IR，Intermediate Representation）。EDA 工具通常會先將原始碼解析為 AST（抽象語法樹），再綜合轉換為 netlist 形式的 IR，最後才進行後端處理。

在 eda4 專案中，v2f-synth crate 負責將 Verilog 原始碼綜合為內部 netlist 表示，然後序列化成 Yosys-JSON 格式；v2f-pnr crate 則讀取此 JSON 並執行佈局與繞線。兩個 crate 透過標準化的 Yosys-JSON 格式作為銜接介面。

## 層級結構：平坦 vs 階層式 Netlist

Netlist 可以分為平坦（flat）與階層式（hierarchical）兩種結構。

平坦 netlist 將整個設計展開成一個單層的元件集合，所有實體化（instantiation）都被遞迴展開，直到只剩下基本邏輯閘為止。這種結構簡單直接，適合進行邏輯最佳化、技術映射和後端實體設計。然而，當設計規模龐大時，平坦 netlist 會導致記憶體使用量過大，且難以保留原始設計的結構資訊。

階層式 netlist 則保留原始設計中的模組層級。頂層模組包含對子模組的參考，每個子模組內部又可能包含更下層的元件。這種結構與原始 Verilog 的模組層級相對應，有利於團隊協作與 IP 重複使用。缺點是後端工具需要遞迴處理所有層級才能進行全局最佳化。

Yosys-JSON 格式採用階層式結構：JSON 的最上層是一個 `modules` 物件，其鍵值為模組名稱，值為對應的模組定義。每個模組內部包含自己的 `ports`、`cells` 和 `netnames`。這種設計讓 Yosys 可以保留 `hierarchy` 指令處理後的模組邊界，同時也允許 `flatten` 指令將所有層級合併為單一模組。

eda4 專案中的 v2f-synth 當前僅處理單一頂層模組，產生的 Yosys-JSON 只包含一個模組條目。實際上 v2f-synth 的 `elaborate` 函式接受單一 `Module` 結構（從 v2f-synth 的 parser 解析而來），並直接產出平坦 netlist。這意味著在 v2f-synth 目前的實作中，所有層級都會在合成前先被展開。

## Yosys-JSON 交換格式

Yosys-JSON 格式是 Yosys 綜合工具定義的一種結構化 JSON 規格，用於在 Yosys 與 nextpnr（或其它後端工具）之間傳遞 netlist 資料。它本質上是 Yosys 內部 `RTLIL::Module` 結構的序列化表示。

### 頂層結構

JSON 的根物件包含以下欄位：

```json
{
  "creator": "v2f-synth v0.3",
  "modules": {
    "module_name": { ... }
  }
}
```

- `creator`：選填，標示產生此 JSON 的工具名稱與版本。
- `modules`：物件，鍵為模組名稱，值為模組定義。每個模組名稱在整個 JSON 中必須唯一。

### 模組定義

每個模組是一個 JSON 物件，包含三個主要欄位：`ports`、`cells` 和 `netnames`。

#### Ports（埠）

`ports` 是一個物件，鍵為埠名稱，值為埠定義。每個埠定義包含：

- `direction`：字串，值為 `"input"`、`"output"` 或 `"inout"`。
- `bits`：整數陣列，表示此埠所連接的位元 ID 列表。每個整數是全局唯一的 BitId。

範例：
```json
"clk": { "direction": "input", "bits": [1] },
"led": { "direction": "output", "bits": [2] }
```

#### Cells（元件）

`cells` 是一個物件，鍵為元件名稱（通常以 `$` 開頭表示內部產生的名稱），值為元件定義。每個元件定義包含：

- `type`：字串，表示元件類型。Yosys 使用 `$_` 前綴表示標準邏輯閘（如 `$_AND_`、`$_DFF_P_`），而 `$` 前綴表示內部單元（如 `$add`）。技術映射後的 FPGA 元件則使用廠商名稱（如 `ICESTORM_LC`）。
- `parameters`：物件，選填。鍵為參數名稱，值為對應的 JSON 值。例如 LUT 的初始化值 `LUT_INIT`。
- `port_directions`：物件，選填。鍵為埠名稱，值為 `"input"` 或 `"output"`。這相當於元件各埠的方向資訊。
- `connections`：物件，鍵為埠名稱，值為對應的 BitId 陣列。表示元件這個埠連接到哪些位元。

範例：
```json
"$0": {
  "type": "$_AND_",
  "port_directions": { "A": "input", "B": "input", "Y": "output" },
  "connections": { "A": [10], "B": [11], "Y": [12] }
}
```

#### Netnames（網路名稱）

`netnames` 是一個物件，鍵為網路（導線）名稱，值為網路定義。每個網路定義包含：

- `bits`：整數陣列，表示此命名網路所包含的 BitId。
- `hide_name`：整數，選填。若為 `1` 則表示此網路名稱為內部生成，不應顯示在除錯輸出中。預設值為 `0`。

Netnames 的作用是保留原始設計中的訊號名稱，使得除錯與分析更為直觀。即使經過多階段的處理，只要 bit 的對應關係沒有改變，原始的名稱就能被追蹤。

## 範例：Blinky 模組的 Yosys-JSON

以下是一個 blinky 模組（含計數器與 LED 輸出）的完整 Yosys-JSON 表示：

```json
{
  "creator": "v2f-synth v0.3",
  "modules": {
    "blinky": {
      "ports": {
        "clk": { "direction": "input", "bits": [1] },
        "led": { "direction": "output", "bits": [2] }
      },
      "cells": {
        "$0": { "type": "$_INPUT_", "connections": { "Y": [1] } },
        "$1": { "type": "$_OUTPUT_", "connections": { "A": [94], "Y": [2] } },
        "$2": { "type": "$_DFF_P_", "connections": { "D": [4], "Q": [3] } },
        "$3": { "type": "$_ADD_", "connections": { "A": [3], "B": [5], "Y": [4] } },
        "$4": { "type": "$_ONE_", "connections": { "Y": [5] } },
        "$5": { "type": "$_DFF_P_", "connections": { "D": [7], "Q": [6] } },
        "$6": { "type": "$_ADD_", "connections": { "A": [6], "B": [8], "Y": [7] } },
        "$7": { "type": "$_ZERO_", "connections": { "Y": [8] } },
        "$8": { "type": "$_AND_", "connections": { "A": [9], "B": [9], "Y": [94] } },
        "$9": { "type": "$_DFF_P_", "connections": { "D": [11], "Q": [9] } }
      },
      "netnames": {
        "clk": { "bits": [1] },
        "led": { "bits": [2] },
        "counter.0": { "bits": [3], "hide_name": 1 },
        "_dff_2": { "bits": [4], "hide_name": 1 },
        "counter.1": { "bits": [6], "hide_name": 1 },
        "_dff_6": { "bits": [7], "hide_name": 1 }
      }
    }
  }
}
```

此例中可以看到：
- 輸入埠 `clk` 連接到 BitId 1，輸出埠 `led` 連接到 BitId 2。
- DFF 元件與加法器元件串接形成計數器鏈。
- `$_ONE_` 元件產生常數 1 作為加法器的另一個輸入。
- `$_AND_` 元件（以 A 與 B 接同一訊號的形式）做為 buffer 使用，驅動輸出埠 `led`。
- Netnames 保留 `counter.0`、`counter.1` 等原始訊號名稱，方便追蹤。

## v2f-synth 的內部 Netlist IR

v2f-synth crate 定義了一套記憶體中的 netlist IR（Intermediate Representation），位於 `v2f-synth/src/netlist.rs`。這套 IR 是介於 Verilog AST 與 Yosys-JSON 之間的中間表示，設計輕量且易於操作。

### BitId：全域唯一的位元識別碼

`BitId` 是 `u64` 的型別別名，代表電路中每一個信號位元的全域唯一整數 ID。在 `Netlist` 結構中，`next_bit` 欄位是一個不斷遞增的計數器，每次呼叫 `alloc_bit()` 就會配置一個新的 BitId。BitId 從 1 開始編號（0 保留未使用）。

BitId 的設計原因在於：Verilog 訊號名稱可長可短，且包含陣列索引（如 `counter[25]`）、部分選取（如 `data[3:0]`）等複雜結構。直接使用字串進行路由表的查找與比對效率低落。BitId 將所有訊號平面化為整數索引，使得位元層級的連接關係可以在常數時間內查詢，並且在後續的 PNR 流程中作為唯一的參考依據。

### Netlist 結構

```rust
pub struct Netlist {
    pub top: String,        // 頂層模組名稱
    pub ports: Vec<NetPort>, // 所有埠
    pub cells: Vec<Cell>,    // 所有元件
    pub bits: Vec<BitInfo>,  // 所有位元的中繼資料
    pub next_bit: BitId,     // 下一個可用的位元 ID
}
```

### Cell 與 CellKind

每個 `Cell` 代表一個邏輯元件，包含名稱、類型（`CellKind` 列舉）、輸入連線列表與輸出連線列表。輸入與輸出都以 `(String, Vec<BitId>)` 的形式儲存，字串為埠名稱（如 `"A"`、`"B"`、`"Y"`）。

`CellKind` 列舉定義了 v2f-synth 支援的所有基本元件類型：

- `Input` / `Output`：模組的輸入與輸出埠。
- `And` / `Or` / `Xor` / `Not`：基本邏輯閘。
- `Dff`：D 型正邊緣觸發正反器。
- `Lut { init: u16 }`：查找表，`init` 為 16 位元的 LUT 初始化值（對應 iCE40 的 4-LUT）。
- `Carry`：進位鏈元件。
- `Mux2`：2-to-1 多工器。
- `Const0` / `Const1`：常數 0 與常數 1。
- `Add` / `Sub`：加法器與減法器。

### BitInfo

每個配置出去的 BitId 都有對應的 `BitInfo` 記錄，包含該位元的 ID、可選的名稱（`name: Option<String>`）與是否為埠的標記（`is_port`）。名稱資訊主要用於產生 Yosys-JSON 中的 `netnames`，讓最終的 JSON 保留原始訊號名稱以便除錯。

### NetPort

`NetPort` 記錄模組埠的資訊：名稱、方向（`PortDir::Input` / `Output` / `Inout`）以及所連接的 BitId 列表。Ports 與一般的 `Cell` 是分開儲存的，因為 PNR 階段需要獨立處理埠的物理位置分配。

## 從 AST 建構 Netlist：Elaboration 過程

Elaboration（詳細化）是將解析後的 Verilog AST 轉換為 netlist IR 的過程，實作在 `v2f-synth/src/elab.rs` 的 `elaborate` 函式中。

### 步驟一：分配埠與訊號的 BitId

函式首先遍歷模組的埠列表，為每個埠分配所需的 BitId 數量。例如 `input [3:0] a` 會分配 4 個 BitId，並將它們命名為 `a.0`、`a.1`、`a.2`、`a.3`。這些 BitId 同時存入 `sigs`（符號表）與 `net.ports` 中。

接著處理 `wire` 與 `reg` 宣告，為內部訊號分配 BitId。已經存在於 `sigs` 中的名稱（如已處理的埠）會被跳過，避免重複配置。

### 步驟二：處理 continuous assignment

對於 `assign` 語句，`elaborate` 函式會遞迴解析賦值目標（target）與賦值來源（value）的運算式樹。解析過程中遇到二元運算（如 `+`、`&`、`|`）時，對每個位元建立對應的 `Cell`。例如 `assign y = a & b` 會對每一位建立一個 `CellKind::And` 元件。

每個運算元都會先透過 `resolve_expr` 遞迴展開，直到碰到基本識別碼（`Ident`）或常數（`Number`）為止。常數產生的方式較為特別：對於 `Expr::Number(v, w)`，每個位元若為 1 則建立 `Const1` 元件，否則建立 `Const0` 元件。

### 步驟三：處理 always block

`always` 區塊的處理目前僅支援 `posedge` 觸發的同步邏輯。對於 `Nonblocking` 或 `Blocking` 賦值（如 `counter <= counter + 1`），會為每個目標位元建立一個 `CellKind::Dff` 元件。DFF 的 D 輸入連接到賦值來源的運算結果，Q 輸出則透過一個 buffer（`And` 單元，A 與 B 接同一輸入）連接到實際的目標訊號。

### 步驟四：新增 Port Cell

最後，為每個輸出埠與 inout 埠建立 `CellKind::Output` 元件，為每個輸入埠建立 `CellKind::Input` 元件。這些特殊的 cell 讓 PNR 階段能夠識別哪些訊號是模組的邊界埠。

這種 elaboration 設計的關鍵特性在於：它是一次性的單向處理。elaborate 函式接受一個已解析的 `Module`，回傳一個完整的 `Netlist`，中間不涉及迭代最佳化或反向傳播。這保持了實作的簡單性，但同時也意味著未來若要支援更複雜的 Verilog 語法（如多層級實體化、generate 區塊、function/task），需要擴充 elaboration 的邏輯。

## 從內部 Netlist 映像到 Yosys-JSON

`v2f-synth/src/techmap.rs` 中的 `techmap_to_json` 函式負責將內部的 `Netlist` 結構轉換為 Yosys-JSON 格式的 `HashMap<String, YosysModule>`。這個過程稱為技術映射（technology mapping）的一環。

### Cell 類型對應

`map_cell` 函式將 `CellKind` 對應到 Yosys 標準的 cell 類型字串：

| CellKind | Yosys 類型 | 說明 |
|---|---|---|
| `Input` | `$_INPUT_` | 輸入埠 |
| `Output` | `$_OUTPUT_` | 輸出埠 |
| `And` | `$_AND_` | 2 輸入 AND |
| `Or` | `$_OR_` | 2 輸入 OR |
| `Xor` | `$_XOR_` | 2 輸入 XOR |
| `Not` | `$_NOT_` | 反向器 |
| `Dff` | `$_DFF_P_` | 正邊緣觸發 DFF |
| `Const0` | `$_ZERO_` | 常數 0 |
| `Const1` | `$_ONE_` | 常數 1 |
| `Add` | `$_ADD_` | 加法器 |
| `Sub` | `$_SUB_` | 減法器 |
| `Mux2` | `$_MUX_` | 2-to-1 多工器 |
| `Lut` | `ICESTORM_LC` | iCE40 LUT（附 LUT_INIT 參數） |
| `Carry` | `$_CARRY_` | 進位鏈 |

每個 cell 的連接關係會按照 Yosys 的慣例進行映像：`And` 的輸入為 `A` 與 `B`、輸出為 `Y`；`Dff` 的輸入為 `D` 與 `C`（時脈）、輸出為 `Q`；`Lut` 的輸入為 `I0` 到 `I3`、輸出為 `O`。

### Netnames 的產生

`techmap_to_json` 會將 `BitInfo` 中的名稱資訊反向彙整為 `netnames`。它建立一個 `bit_to_name` 雜湊表，記錄每個 BitId 對應的名稱。若有兩個位元共享同一個名稱（如匯流排的各個位元），它們會被合併到同一個 `netnames` 條目中。埠的名稱會蓋過任何內部產生的名稱，確保埠名稱在 JSON 中保持正確。內部生成的訊號（不屬於任何埠的）會被標記為 `hide_name: 1`。

### LUT 打包

在 iCE40 架構中，邏輯是以 4 輸入查找表（4-LUT）為基本單位實現的。`ICESTORM_LC` 類型對應到 iCE40 的邏輯單元，包含一個可設定初始值的 LUT（`LUT_INIT` 參數）。這 16 位元的遮罩值編碼了 LUT 的真值表行為。

將多層的 gate（And/Or/Xor/Not）打包進單一 LUT 是技術映射中的關鍵最佳化步驟。雖然 v2f-synth 目前的 `CellKind::Lut` 已經定義了 `init` 欄位，但實際的 LUT 打包演算法是在更高的抽象層次進行的。工具鏈可以選擇將互連的小型邏輯閘樹合併為單一 `ICESTORM_LC` 元件，以減少 LUT 的使用量並改善路徑延遲。

## v2f-pnr 如何讀取 JSON Netlist

v2f-pnr crate 的 `v2f-pnr/src/pnr.rs` 定義了 `parse_json` 函式，負責將 Yosys-JSON 格式的字串解析為 PNR 階段使用的內部表示。

### 資料結構

`SynthJson` 是 JSON 解析的根結構，包含選填的 `creator` 與 `modules`。解析時使用 `serde_json::from_str` 並依賴結構的自動推導（`#[derive(Deserialize)]`）。

`ModuleJson` 對應 Yosys-JSON 中的模組定義，包含 `ports`、`cells` 與 `netnames`。`CellJson` 使用 `#[serde(rename = "type")]` 屬性將 JSON 中的 `type` 欄位對應到 Rust 的 `cell_type` 欄位（因為 `type` 是 Rust 的保留關鍵字）。

### 解析流程

`parse_json` 的執行流程如下：

1. **收集 cell 名稱與類型**：遍歷所有 cell，將其名稱與類型分別推入 `cell_names` 與 `cell_types` 向量。
2. **加入 port cell**：埠被視為特殊的 `"PORT"` 類型元件加入列表，名稱加上 `"port_"` 前綴以避免與既有 cell 衝突。
3. **建立 bit 到 cell 的反向索引**：遍歷所有 cell 與 port 的連接關係，建立 `bit_to_cells: HashMap<u64, Vec<usize>>`，記錄每個 BitId 被哪些 cell 索引使用。
4. **根據 netnames 建立連線群組**：遍歷 `netnames`，對於每個網路名稱，取出其包含的所有 BitId，查詢 `bit_to_cells` 找出所有相關的 cell 索引。這些 cell 被收集為一個群組（net group），存入 `net_conns` 向量。`processed` 集合確保每個 cell 只能屬於一個群組。
5. **處理未連接的 cell**：任何不在 `processed` 中的 cell 會被新增為獨立的單 cell 群組。

最終產出的 `PnrNetlist` 結構包含三個向量：
- `cell_names`：所有元件的名稱列表。
- `cell_types`：所有元件的類型列表，兩者透過索引對應。
- `net_conns`：連線群組，每個 `Vec<usize>` 包含一組互相連接的 cell 索引。

### PNR 主流程

`run_pnr` 函式整合了完整的 PNR 流程：
1. 呼叫 `parse_json` 解析 netlist。
2. 建立 `ArchGraph`（架構圖，包含 FPGA 的邏輯單元位置與路由資源）。
3. 執行隨機初始佈局，然後透過模擬退火演算法進行最佳化（`place::random_placement` + `place::place`）。
4. 執行繞線（`route::route`）。
5. 輸出 ASC 格式的佈局結果（`asc_out::write_asc`）。

這個流程展示了 JSON netlist 在 EDA 工具鏈中的核心角色：作為合成階段的輸出，同時也是佈局與繞線階段的輸入，是整個工具鏈的資料交換中樞。

## Netnames 的作用：保留原始訊號名稱

在數位電路設計中，原始訊號名稱不僅是人類工程師理解電路的關鍵，也是除錯與驗證的重要依據。Yosys-JSON 格式中的 `netnames` 就是為了保留這些名稱而設計的。

當 v2f-synth 對 Verilog 進行綜合時，`elaborate` 函式會在建立 BitId 後立即為其命名。例如 `counter[25]` 這個訊號會被命名為 `counter.25`。這些名稱雖然在內部運算時不會被使用（內部使用 BitId 作為唯一標識），但在序列化成 JSON 時會被記錄在 `netnames` 中。

在後續的 PNR 流程中，`parse_json` 利用 `netnames` 將原本依賴 BitId 的連線關係分組為邏輯上相關的 net group。雖然 PNR 階段的實體佈局與繞線主要依賴 cell 與 net 的拓撲結構，但保留的名稱可以讓產出的 ASC 檔案包含原始訊號名稱，從而在 FPGA 編輯器（如 nextpnr-gui）或波形觀察工具中顯示可讀的訊號名稱。

`hide_name` 標記用於區分使用者定義的名稱與工具內部產生的名稱。內部產生的名稱（如 `_dff_2`、`$0`、`$1`）通常對使用者沒有意義，因此在序列化時被標記為 `hide_name: 1`，讓前端工具可以選擇性地隱藏這些訊號。

## 導線位元順序：Little-Endian 慣例

Yosys-JSON 格式與 v2f-synth 皆採用 little-endian 的位元順序，亦即位元 0（bit 0）對應最低有效位元（LSB）。

在 Elaboration 階段，當處理 `input [3:0] a` 這類匯流排宣告時，`alloc_bits` 分配的 BitId 陣列順序為 `[a_bit0, a_bit1, a_bit2, a_bit3]`，其中索引 0 對應到 LSB（`a[0]`）。同樣地，`assign y[7:0] = a[7:0]` 的範圍選取中，`Range` 表達式展開時會保持 LSB 在索引 0 的位置。

當 Yosys-JSON 序列化時，埠的 `bits` 陣列也遵循此順序。例如 4 位元輸入埠的 `"bits": [5, 6, 7, 8]` 表示 bit 0（LSB）對應 BitId 5，bit 3（MSB）對應 BitId 8。

這種慣例與 iCE40 實體元件的接腳對應方式一致，也與 Verilog 語言中 `[3:0]` 宣告的位元索引方式相符。保持一致的位元順序對於確保工具鏈各階段之間資料的一致性至關重要：如果合成階段與 PNR 階段對位元順序的理解不一致，將導致電路功能錯誤。

## 延伸閱讀

- [Netlist (Wikipedia)](https://en.wikipedia.org/wiki/Netlist)
- [Yosys (Wikipedia)](https://en.wikipedia.org/wiki/Yosys_(software))
- [Logic Synthesis (Wikipedia)](https://en.wikipedia.org/wiki/Logic_synthesis)
- [Boolean Algebra (Wikipedia)](https://en.wikipedia.org/wiki/Boolean_algebra)
