# Place & Route (PNR) 佈局與繞線

- 作者：eda4 團隊
- 對應程式碼：`verilog2fpga/v2f-pnr/src/`

## 什麼是 Place & Route

Place & Route（PNR，佈局與繞線）是 FPGA 設計流程中銜接邏輯合成與位元流產生的關鍵階段。合成器將 HDL 原始碼轉換為由基本邏輯閘組成的網表（netlist）後，PNR 的任務是將這些邏輯閘**放置**到 FPGA 晶片上的實體位置，並在 FPGA 的可程式化繞線資源中**繞線**（route）來連接它們。

PNR 的本質是一個大型組合最佳化問題。給定一個由邏輯單元（cells）與連線（nets）組成的電路，以及一個由可程式化磚塊（tiles）與繞線通道組成的 FPGA 架構，PNR 必須同時滿足兩個相互依存的目標：
- 為每個邏輯單元分配一個實體磚塊（placement）
- 在磚塊之間找到可行的物理路徑來實現所有連線（routing）

eda4 專案的 `v2f-pnr` crate 實現了一套純 Rust 的 PNR 管線，採用**模擬退火**（Simulated Annealing）進行佈局最佳化，並以**A\* 路徑搜尋**進行繞線。它接收 Yosys 相容的 JSON 網表作為輸入，最終輸出 ASC 格式的佈局結果，可繼續餵入 `v2f-bitstream` 打包為 FPGA 位元流。

## 佈局（Placement）

佈局是 PNR 的第一個階段，目標是為網表中的每個邏輯單元分配一個特定的 FPGA 磚塊座標 `(row, col)`。在 iCE40 系列 FPGA 上，磚塊分為兩大類：
- **邏輯磚塊（Logic Tile）**：位於晶片內部網格，容納 LUT、正反器、進位鏈等可程式化邏輯
- **I/O 磚塊（IO Tile）**：位於晶片邊緣，負責晶片腳位與內部邏輯的訊號溝通

佈局問題可表述為：給定 `N` 個邏輯單元與 `M` 個可用的邏輯磚塊（通常 `M >= N`），尋找一個從單元到磚塊的映射關係，使得某個成本函數最小化。這本質上是**二次指派問題**（Quadratic Assignment Problem, QAP）的一種形式，屬於 NP-hard。

在 `v2f-pnr` 中，佈局的初始狀態採用**隨機放置**：每個單元被隨機分配到一個邏輯磚塊上。雖然這會產生一個品質極差的初始解，但模擬退火演算法有能力在迭代過程中逐步改善它。

隨機放置的實作位於 `place.rs` 的 `random_placement()` 函數中。它從 `ArchGraph::logic_tiles()` 取得所有可用邏輯磚塊的座標列表，然後為每個單元隨機選取一個座標，並初始計算總成本。

### 模擬退火演算法

模擬退火是一種基於物理退火過程的通用啟發式最佳化演算法。其核心概念是：在高溫時允許接受較差的解（以逃離局部最佳值），隨著溫度逐漸降低，接受劣解的概率也隨之減少，最終收斂到一個近似全域最佳的解。

`v2f-pnr` 的模擬退火實作於 `place()` 函數中，參數如下：

- **起始溫度（`t_start`）**：100.0
- **終止溫度（`t_end`）**：1.0
- **降溫係數（`cooling`）**：0.9
- **每步迭代次數（`iters_per_step`）**：`max(num_cells * 50, 100)`
- **最大停滯次數（`stall`）**：5 — 若連續 5 個溫度步無改善則提前終止

每個溫度步的迭代邏輯如下：
1. 隨機選取一個單元 `i`
2. 記錄其當前位置 `old`
3. 將其移動到一個隨機的邏輯磚塊 `new_coord`
4. 計算移動後的新成本 `new_cost`
5. 計算成本差異 `delta = new_cost - cost`
6. 若 `delta < 0`（成本下降），接受此移動
7. 若 `delta >= 0`，以概率 `exp(-delta / temp)` 決定是否接受
8. 若拒絕，將單元移回原位

這種隨機擾動與機率性接受機制讓演算法能夠在早期階段探索廣闊的解空間，並在後期階段專注於局部微調。

## 成本函數

成本函數是引導模擬退火搜尋方向的關鍵。`v2f-pnr` 的成本計算實作於 `cost.rs`，由 `PlacerCost` 結構體封裝兩個加權項：

```rust
pub struct PlacerCost {
    pub w_wire: f64,    // 線長權重（預設 1.0）
    pub w_cong: f64,    // 擁塞權重（預設 0.5）
}
```

總成本公式為：`cost = w_wire * wire_length + w_cong * congestion_penalty`

### 線長（Wire Length）

線長採用**半周長線長**（Half-Perimeter Wire Length, HPWL）作為估計。對於一個連接 `k` 個磚塊的網路，HPWL 定義為：

```
HPWL = (max_row - min_row) + (max_col - min_col)
```

這相當於覆蓋所有連接磚塊的最小矩形周長的一半。HPWL 是佈局階段最廣泛使用的線長估計模型，雖然它忽略了實際繞線路徑的曲折，但作為一種快速可計算的啟發式指標，它與實際繞線長度具有高度正相關。

`v2f-pnr` 的 `total_wire_length()` 函數對所有網路（net）的 HPWL 進行加總。對於只有一個終端（single-pin net）的網路，線長計為 0，因為這類網路不需要實際的繞線。

### 擁塞懲罰（Congestion Penalty）

擁塞懲罰用於避免多個單元被分配到同一個磚塊上——這在物理上是不可能的，因為每個 iCE40 邏輯磚塊只能容納一個邏輯單元。

`congestion_penalty()` 函數透過一個 `HashSet` 來檢測是否有重複的座標：
- 若所有單元都位於不同的磚塊上，懲罰為 0
- 若有任何重疊，懲罰為 1000.0（一個極大的數值，確保模擬退火會極力避免）

## 繞線（Routing）

繞線是 PNR 的第二個階段，目標是在給定的佈局結果上，為每個網路找到一條或多條穿過 FPGA 繞線資源的物理路徑。與 PCB 繞線或 ASIC 繞線不同，FPGA 繞線的通道與交換資源在晶片製造時就已固定，繞線器只能在預先存在的繞線網格上選擇可用的路徑。

在 iCE40 架構中，每個磚塊之間存在垂直與水平的繞線通道，以及可程式化的連接點（switch box / connection box）。繞線器的任務就是在這個網格圖中為每個網路找出一條連接其所有終端的路徑。

`v2f-pnr` 採用迭代式 A\* 繞線，實作於 `route.rs`。其核心流程如下：

### A\* 路徑搜尋

A\*（A-star）是一種經典的啟發式路徑搜尋演算法，使用 `f = g + h` 作為節點評價函數，其中：
- `g` 為從起點到當前節點的實際路徑成本
- `h` 為從當前節點到終點的估計成本（啟發式函數）

`v2f-pnr` 的 A\* 實作特點：
- 採用曼哈頓距離（Manhattan distance）作為啟發式函數 `h`
- 使用二元堆積（`BinaryHeap`）作為優先佇列，以 `f` 值排序
- 透過 `came_from` 映射追蹤路徑，在找到目標後回溯重建完整路徑
- `neighbors()` 函數產生上下左右四個方向的相鄰磚塊

曼哈頓距離計算是 `manhattan()` 函數：`|a.row - b.row| + |a.col - b.col|`。由於 A\* 要求啟發式函數不能高估實際成本（admissible），而曼哈頓距離在網格圖中確實是下界，因此保證能找到最短路徑。

### 擁塞感知繞線

實際繞線中最大的挑戰不是找出一條路徑，而是同時為所有網路找出路徑而不互相衝突。單一網路的最短路徑往往會與其他網路的最短路徑爭奪相同的繞線資源。

`v2f-pnr` 透過**疊代式解除繞線與重新繞線**（rip-up and reroute）來處理擁塞：

1. 初始化一個空的擁塞圖（`congestion: HashMap<TileCoord, u32>`）
2. 對每個網路依序以 A\* 繞線，將使用到的磚塊的擁塞計數加 1
3. 重複最多 5 次（`max_iters = 5`）：
   - 清除所有先前的繞線路徑
   - 使用當前的擁塞資訊重新繞線所有網路
   - 若所有網路皆成功繞線，提前結束
   - 每次迭代後將所有擁塞計數減 1（衰減機制）

在 A\* 的成本計算中，繞過擁塞磚塊的額外成本為 `congestion_penalty * 10`。這意味著若某個磚塊被多個網路爭奪，後續的繞線迭代會傾向於繞道而行，從而將流量分散到不同的路徑上。

### 繞線結果

繞線結果由 `Routing` 結構體表示，包含 `net_paths: Vec<RoutingPath>`，其中每個 `RoutingPath` 記錄了一條由連續磚塊座標組成的路徑。若某個網路繞線失敗（A\* 無法找到路徑），則使用僅包含源頭座標的退化路徑作為佔位。

## ASC 輸出

ASC（ASCII）格式是 nextpnr 與 icestorm 工具鏈使用的文字格式，用於描述佈局與繞線結果。`v2f-pnr` 的 ASC 序列化實作於 `asc_out.rs` 的 `write_asc()` 函數中。

ASC 檔案包含三大區段：

### .device 指令

指定目標裝置型號，字串對應關係為：
- `HX1K` → `"HX1K-TQ144"`
- `HX4K` → `"HX4K-TQ144"`
- `HX8K` → `"HX8K-CT256"`
- `LP1K` → `"LP1K-CM36"`
- `UP5K` → `"UP5K-SG48"`

### .logic_tile 區段

對每個已放置的邏輯單元，寫入一個 `.logic_tile` 區塊：

```
.logic_tile <col> <row>
  .sym <row*100 + col> 0 0 0 0 "<cell_name>"
```

`<row*100 + col>` 作為該磚塊內單元的唯一識別碼。每個 `.sym` 條目對應一個單元的名稱與其在 FPGA 磚塊網格中的位置。

### .wiring 區段

對每個繞線路徑中的相鄰磚塊對，寫入一條 `.wiring` 指令：

```
.wiring <from_col> <from_row> <to_col> <track>
```

其中 `track = net_index % 8`，作為一個簡化的繞線通道分配策略。在真實的 iCE40 繞線模型中，每個繞線軌道（track）代表不同的垂直或水平繞線資源，此處的 `% 8` 是一種教學用途的簡化處理。

## PNR Netlist 與 JSON 解析

`v2f-pnr` 的輸入是 Yosys 相容的 JSON 格式。解析實作於 `pnr.rs`，定義了一系列 Serde 反序列化型別：

### 資料結構

- **`SynthJson`**：頂層結構，包含 `creator`（工具來源）與 `modules`（多個模組）
- **`ModuleJson`**：單一模組，包含 `ports`、`cells`、`netnames`
- **`CellJson`**：單一邏輯單元，包含 `type`（單元種類）、`parameters`（參數表）、`port_directions`（埠方向）、`connections`（連線映射）
- **`NetJson`**：一個命名網路，包含 `bits`（位元 ID 列表）與 `hide_name`（是否隱藏名稱）
- **`PortJson`**：頂層埠，包含 `direction`（"input"/"output"）與 `bits`

### PnrNetlist 內部表示

`parse_json()` 函數將 JSON 解析為 `PnrNetlist` 結構體：

```rust
pub struct PnrNetlist {
    pub cell_names: Vec<String>,
    pub cell_types: Vec<String>,
    pub net_conns: Vec<Vec<usize>>,
}
```

轉換邏輯：
1. 從 JSON 的 `cells` 區段收集所有單元名稱與型別
2. 將 `ports` 轉換為特殊的 `"PORT"` 型別單元（前綴 `port_`）
3. 建立 `bit_to_cells` 反向索引表，將每個位元 ID 對應到其所連接的單元索引
4. 從 `netnames` 區段提取網路分組——每個 `netname` 定義一組應相連的單元
5. 尋找未連接的孤立單元，為其建立獨立的單元網路

### Cell 型別約定

`v2f-pnr` 對單元型別的處理使用簡單的啟發式規則：
- 型別以 `"SB_"` 開頭的單元（如 `"SB_IO"`、`"SB_GB_IO"`）只能放置在 IO 磚塊
- 型別以 `"$_INPUT_"` 或 `"$_OUTPUT_"` 開頭的單元也只能放置在 IO 磚塊
- 所有其他型別的單元放置在邏輯磚塊

這些規則實作於 `ArchGraph::is_valid_placement()` 方法中。

## 架構模型（ArchGraph）

`v2f-pnr` 的 FPGA 架構模型由 `arch.rs` 定義，核心型別如下：

### ArchGraph

`ArchGraph` 是整個 FPGA 晶片的抽象表示：

```rust
pub struct ArchGraph {
    pub device: Device,
    pub ice40: Ice40Device,
    pub tiles: Vec<ArchTile>,
    pub logic_rows: u32,
    pub logic_cols: u32,
}
```

- `device`：列舉型別（HX1K/HX4K/HX8K/LP1K/UP5K），決定晶片尺寸
- `ice40`：`v2f-db` 提供的 iCE40 裝置資料庫，包含 `num_rows()` 與 `num_cols()` 方法
- `tiles`：所有磚塊的列表
- `logic_rows` / `logic_cols`：邏輯磚塊區域的尺寸（總列數減去上下各一列 IO 行）

### ArchTile 與 TileType

每個磚塊（`ArchTile`）包含座標（`TileCoord`）與型別（`TileType`）：

- **`TileType::Logic`**：邏輯磚塊，可放置 LUT、DFF 等一般邏輯單元
- **`TileType::Io`**：I/O 磚塊，位於晶片邊緣，僅可放置 IO 相關單元

`ArchGraph::new()` 在初始化時遍歷所有 `(row, col)` 座標，根據位置決定磚塊型別：
- 第 0 列或最後一列 → IO
- 第 0 行或最後一行 → IO
- 其餘 → Logic

### 架構查詢方法

- **`logic_tiles()`**：回傳所有邏輯磚塊的座標列表，用於隨機放置與交換
- **`io_tiles()`**：回傳所有 I/O 磚塊的座標列表
- **`is_valid_placement(coord, cell_type)`**：檢查特定單元型別是否能放置在特定座標上

## PNR 的 NP-hard 本質

PNR 之所以困難，在於它所包含的兩個子問題都是計算理論中的經典難題：

### 佈局是 NP-hard

佈局問題可化歸為**二次指派問題**（Quadratic Assignment Problem），這是最經典的 NP-hard 組合最佳化問題之一。給定 `n` 個單元與 `n` 個位置，要找到一個指派使得所有配對單元之間的流量乘積與距離乘積的和最小化，其解空間大小為 `n!`。對於一個含有 1000 個單元的設計，窮舉搜尋是天文數字級別的不可能任務。

即使對問題進行簡化——例如使用線性規劃或解析法求解——精確求解的計算複雜度仍然是指數級的。這解釋了為什麼業界與學術界廣泛採用啟發式演算法（如模擬退火、遺傳演算法、解析法）來尋找近似最佳解。

### 繞線是 NP-hard

繞線問題中的多終端網路繞線（multi-terminal net routing）本質上是**斯坦納樹問題**（Steiner Tree Problem）在網格圖上的變體。即使只考慮兩個終端之間的繞線（最短路徑），在可用的多層繞線資源中分配路徑也需要同時考慮所有網路的全局擁塞，這使得問題更加複雜。

同時繞線所有網路的**全局繞線**（global routing）問題可化歸為圖著色問題的變體，同樣屬於 NP-hard 範疇。`v2f-pnr` 使用的「逐一繞線後疊代修正」方法是實務上最常見的啟發式策略。

## 與 nextpnr 的比較

nextpnr 是 Project IceStorm 生態系的官方 PNR 工具，由 Yosys 開發團隊維護，支援 iCE40、ECP5、以及 Nexus 等 FPGA 系列。`v2f-pnr` 與 nextpnr 的設計目標與實現策略有著本質上的差異：

| 面向 | nextpnr | v2f-pnr |
|------|---------|---------|
| 定位 | 生產級工具 | 教育/實驗性工具 |
| 佈局演算法 | 解析法（analytical placement）+ 合法化 | 模擬退火（模擬退火） |
| 時序驅動 | 支援（STA 引擎） | 不支援 |
| 繞線演算法 | 疊代式迷宮繞線（多層網格） | 疊代式 A\*（單層網格） |
| 架構模型 | 完整的 tile 與 routing 資源描述 | 簡化的邏輯/IO 二分模型 |
| 效能 | 可處理數萬 LUT 的設計 | 適合數百 LUT 的小型設計 |
| 語言 | C++ | Rust |

nextpnr 的解析法佈局使用加權線性系統來求解連續空間中的最佳位置，然後透過合法化（legalization）將單元映射到離散的磚塊上。這種方法在處理大型設計時收斂速度遠快於模擬退火。此外，nextpnr 內建靜態時序分析（STA）引擎，可以根據時序約束來引導佈局與繞線，這是生產級 PNR 不可或缺的功能。

相比之下，`v2f-pnr` 的定位是：
- 提供一個**可讀、可理解、可修改**的純 Rust PNR 參考實作
- 作為 eda4 純 Rust 工具鏈中展示 PNR 原理的示範元件
- 適合嵌入到教學工具或學術研究中
- 對於小於 100 個單元的簡單設計（如 blinky、adder）可以完成端到端的 PNR 流程

## PNR 的反饋迴路

佈局與繞線之間存在密切的相互依賴關係，形成了 EDA 工具鏈中經典的**反饋迴路**：

1. **佈局影響繞線可行性**：若將兩個需要密集連接的單元放置在晶片的兩端，繞線器將需要大量的繞線資源來連接它們，可能導致繞線失敗。

2. **繞線擁塞反饋指導重新佈局**：在更完善的 PNR 流程中，繞線階段的擁塞資訊會被回饋給佈局器，引導其將高互連密度的單元群集放置在相鄰區域，以減少後續的繞線壓力。

3. **疊代收斂**：理想的 PNR 流程應在佈局與繞線之間進行多次疊代，逐步改善整體解決方案。

`v2f-pnr` 目前的實作採取的是單向管線——佈局完成後直接進行繞線，沒有回饋迴路。這簡化了實作但意味著若繞線階段失敗，整個 PNR 流程無法自動調整佈局來改善繞線結果。這是未來可以擴展的改進方向。

## PNR 流程總覽

完整的 PNR 管線流程由 `pnr.rs` 的 `run_pnr()` 函數驅動：

```
run_pnr(json_str, device)
  │
  ├─ 1. parse_json(json_str)
  │     └─ 將 Yosys-JSON 解析為 PnrNetlist
  │
  ├─ 2. ArchGraph::new(device)
  │     └─ 根據裝置型號建立架構圖
  │
  ├─ 3. random_placement(&cell_names, &net_conns, &arch)
  │     └─ 產生隨機初始佈局
  │
  ├─ 4. place(&mut placement, &arch)
  │     └─ 模擬退火最佳化
  │
  ├─ 5. route(&placement, &arch)
  │     └─ 疊代式 A\* 繞線
  │
  └─ 6. write_asc(&placement, &routing, &arch)
        └─ 序列化為 ASC 格式
```

輸出為 ASC 格式的字串，後續由 `v2f-bitstream` crate 讀取，打包為最終的 FPGA 位元流（BIN 格式）。

## 已知限制與未來方向

`v2f-pnr` 作為一個教學與實驗性質的 PNR 實作，存在以下限制：

- **無時序驅動**：不考慮路徑延遲，不支援 STA 引擎。對於需要滿足特定時序約束的設計不適用。
- **簡化的擁塞模型**：僅追蹤各磚塊被使用的次數，不區分不同類型的繞線資源（垂直 vs. 水平通道、長線 vs. 短線）。
- **無層級感知**：不考慮 FPGA 內部的時脈網路、全局緩衝器、PLL 等特殊資源的佈局約束。
- **無合法化後處理**：雖然擺放約束（邏輯/IO）在成本函數中傾向於被遵守，但沒有強制執行的合法化步驟。
- **單一管線**：佈局與繞線間無反饋機制，無法進行多輪疊代改善。

未來的改進方向包括：引入基於時序分析的靜態時序引擎、支援多層繞線資源模型、實作解析法佈局加速器、以及建立佈局-繞線反饋迴路。

## 延伸閱讀

- [邏輯合成與技術映射](synthesis.md) — PNR 的前置階段
- [網表 / Yosys-JSON](netlist.md) — PNR 的輸入格式
- [模擬退火](simulated_annealing.md) — 佈局階段的核心演算法
- [A\* 路徑搜尋](a_star_routing.md) — 繞線階段的基礎演算法
- [iCE40 架構](ice40.md) — PNR 目標平台的硬體架構
- [ASC 格式 / 磚塊架構](asc_tile.md) — PNR 的輸出格式
- [CRAM / Frame / 位元流](cram_bitstream.md) — PNR 的後續階段

## 延伸閱讀

- [Place and Route (Wikipedia)](https://en.wikipedia.org/wiki/Place_and_route)
- [Electronic Design Automation (Wikipedia)](https://en.wikipedia.org/wiki/Electronic_design_automation)
- [FPGA Place and Route (Wikipedia)](https://en.wikipedia.org/wiki/Field-programmable_gate_array#Design)
- [Timing Closure (Wikipedia)](https://en.wikipedia.org/wiki/Timing_closure)
