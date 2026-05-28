# Lattice iCE40 FPGA 架構

## 概述

Lattice iCE40 是一系列超低功耗 FPGA（現場可程式化閘陣列），由 Lattice Semiconductor 公司推出，定位於可攜式消費性電子、物聯網、感測器介面、嵌入式膠合邏輯、以及軟體定義硬體加速等應用。iCE40 系列以其極低的靜態功耗（最低可達 100 µA 以下）、小封裝尺寸（最小僅 1.4mm x 1.4mm）、以及足夠的邏輯密度（從約 1K 到約 8K LUT），在市場上佔有一席之地。該系列主要分為三個子產品線：iCE40 LP（低功耗，Low Power）、iCE40 HX（高效能，High Performance）、以及 iCE40 UP（超密度，UltraPlus）。

iCE40 的架構於 2015 年經由 Project IceStorm 專案被 Claire Wolf 完整逆向工程，成為少數擁有完整開放原始碼工具鏈支援的 FPGA 架構之一。這使得 iCE40 在 FPGA 教育、學術研究、開放硬體專案（如 open source ASIC 流程中的原型驗證）中廣泛被採用。eda4 專案是一個完全以 Rust 語言實作的 EDA 工具鏈，鎖定 iCE40 系列為目標架構，提供從 Verilog 合成（synthesis）、佈局繞線（place-and-route, PNR）、位元流打包（bitstream packing）到燒錄（programming）的純 Rust 實現，無需依賴 Yosys 或 nextpnr 等外部工具。

iCE40 系列的設定方式基於 SRAM 技術，每次上電需從外部 SPI Flash 載入設定資料至 CRAM（Configuration RAM）。部份型號支援 NVCM（Non-Volatile Configuration Memory），允許在晶片內部非揮發性地儲存設定，斷電後不遺失，開機即可使用。此外，Warm Boot 功能支援在運行中動態切換不同的設定映像，實現動態重配置。

## 裝置規格與產品變體

eda4 專案支援五款 iCE40 裝置，涵蓋 LP、HX、UP 三個子系列。這些裝置的核心幾何參數定義於 `v2f-db` crate 的 `v2f-db/src/ice40.rs` 中，以 `Ice40Device` 列舉表示：

| 裝置 | num_rows | num_cols | 邏輯單元 (LUT4 + DFF) | 系列 | 推出年份 |
|---|---|---|---|---|---|
| HX1K | 30 | 16 | 1,280 (約 1K) | HX | 2012 |
| LP1K | 30 | 16 | 1,100 (約 1.1K) | LP | 2012 |
| HX4K | 40 | 20 | 3,520 (約 3.5K) | HX | 2012 |
| UP5K | 40 | 22 | 5,280 (約 5.2K) | UP | 2015 |
| HX8K | 70 | 28 | 7,680 (約 7.6K) | HX | 2012 |

HX 系列針對效能最佳化，提供更高的最大運作頻率與更多的 I/O 引腳；LP 系列則針對低功耗最佳化，靜態與動態功耗更低，適合電池供電的裝置；UP 系列在功耗與效能之間取得平衡，並增加了 DSP、BRAM、SPRAM 等硬體加速單元，是最多功能的型號。

除上述五款外，Lattice 還推出了 iCE40 LM 系列（內建 NVCM）與 iCE40 Ultra 系列（感測器集線器專用），但它們的核心 FPGA 架構與標準 LP/HX/UP 相同。eda4 專案目前專注於上述五款主流裝置的支援。

### 封裝選項

iCE40 提供多種封裝尺寸以適應不同的應用場景：

- **TQ144 (144-pin LQFP)：** 最大型封裝，提供最多的使用者 I/O（約 112 個），適合需要大量外部連接的設計。用於 HX4K、HX8K。
- **CT256 (256-ball caBGA)：** 高密度 BGA 封裝，I/O 數量最多。用於 HX8K 等大容量裝置。
- **CM36 (36-ball WLCSP)：** 超小型封裝，僅 2.5mm x 2.5mm，I/O 數量有限，適合空間極度受限的可穿戴裝置。用於 LP1K。
- **SG48 (48-pin QFN)：** 中型封裝，在 I/O 數量與面積之間取得平衡。用於 LP1K、HX1K。
- **SG24 (24-pin QFN)：** 最小型封裝，用於最簡單的膠合邏輯應用。

### 邏輯容量計算

每個 Logic Tile 包含 8 個 PLB（Programmable Logic Block），亦即 8 組 LUT+DFF 對。因此理論邏輯單元數可透過網格尺寸計算，以 HX8K 為例：

- 網格 70x28，邊界 I/O Tile 佔去外圍一圈，故 Logic Tile 區域為 68x26 = 1,768 個 tile
- 每個 tile 8 個 PLB = 1,768 x 8 = 14,144 個 LUT+DFF 對
- 但規格表宣稱為 7,680 個邏輯單元

實際上 iCE40 的時脈網路、全域佈線、以及部份 tile 內的特殊功能會佔用部份 PLB，因此可用邏輯單元約為理論值的一半左右。這也是 FPGA 產業的常見現象——廠商公佈的「邏輯單元數」通常低於原始晶片上的實體 LUT 總數，因為部份資源被保留用於晶片內部管理功能。

## 可程式化邏輯單元 (PLB)

iCE40 的基本邏輯單元稱為 PLB（Programmable Logic Block）。每一個 PLB 包含三個主要子單元：4-input LUT、D-type 正反器（DFF）、以及進位邏輯（carry logic）。每個 Logic Tile 包含 8 個 PLB 與區域繞線資源。

### 4-input LUT

LUT（Look-Up Table，查找表）是 FPGA 實現組合邏輯的核心元件。iCE40 使用 4 輸入 LUT，其內部為一個 16x1 的 SRAM——16 個位元儲存真值表，4 個位址輸入（通常標記為 A、B、C、D）選擇其中一個位元輸出。任何 4 輸入的布林函數——從簡單的 AND、OR、XOR 到複雜的多項式或多工器——皆可在此單一 LUT 內實現。

由於 LUT 本質上是 SRAM，它也可以被配置為分散式 RAM（Distributed RAM）或移位暫存器（shift register）。在分散式 RAM 模式下，LUT 的 16 個位元可作為一個 16x1 的單埠 RAM（或 8x2、4x4 等變體），透過 LUT 的輸入引腳進行讀寫操作。移位暫存器模式則將 LUT 配置為 16 位元的靜態移位暫存器，常用於管線延遲、FIFO 同步器等場景。

### D-type 正反器 (DFF)

每個 PLB 包含一個可程式化的 DFF，用於實現同步邏輯（暫存器、狀態機等）。DFF 的功能特性包括：

- **時脈極性選擇：** 可配置為 rising-edge 觸發或 falling-edge 觸發。
- **時脈致能 (Clock Enable)：** 可選擇性地啟用或禁用 DFF，在不需要更新暫存器值時節省動態功耗。
- **同步重置 (Synchronous Reset)：** 支援同步重置輸入，在時脈邊到來時將輸出重置為 0 或 1（可配置）。
- **旁路模式 (Bypass)：** 當僅需組合邏輯時，DFF 可被旁路，LUT 輸出直接連接到 PLB 輸出。
- **初始化值：** DFF 的初始輸出值可在 CRAM 中設定，決定上電或重置後的初始狀態。

DFF 的資料輸入通常來自同一個 PLB 的 LUT 輸出，亦可來自其他 PLB 或繞線資源。這種「LUT + DFF」的配對結構是 FPGA 的基本建構單元，對應於硬體描述語言中「組合邏輯 + 暫存器」的 coding 風格。

### 進位邏輯 (Carry Chain)

為了高效實現算術運算（加法器、計數器、比較器等），每個 PLB 內建快速進位邏輯。進位鏈透過專用的硬體路徑在相鄰 PLB 之間垂直傳遞進位信號：

- **進位產生 (Generate)：** 當兩個輸入位元均為 1 時，無論低位進位為何，該位元必然產生進位。
- **進位傳播 (Propagate)：** 當兩個輸入位元中至少一個為 1 時，該位元的進位輸出等於低位進位輸入。
- **進位消滅 (Kill)：** 當兩個輸入位元均為 0 時，該位元不產生進位且不傳播進位。

LUT 與進位邏輯的協作方式如下：LUT 的 4 個輸入中的 2 個用於 A/B 輸入（資料），另 2 個用於控制進位模式。LUT 的輸出作為「和 (sum)」或「差值 (difference)」，而進位邏輯則獨立計算進位輸出。

進位路徑完全獨立於通用繞線網絡，因此加法器等算術電路可以實現非常低的傳播延遲，不受繞線擁塞的影響。典型加法器的進位鏈延遲約為每 bit 數十皮秒，遠優於使用通用繞線實現的等效電路。

## 拼塊架構 (Tile Architecture)

iCE40 晶片的基礎建構單元為拼塊（tile），每一種類型的 tile 在 CRAM 中佔用不同數量的 Frame。Tile 的類型與 Frame 計數定義在 `v2f-db/src/tile.rs` 的 `TileType` 列舉中：

| Tile 類型 | Frame 數量 | 用途 |
|---|---|---|
| Logic | 7 | 實現組合與同步邏輯 |
| I/O | 3 | 外部引腳介面 |
| BRAM | 14 | 區塊記憶體（僅 iCE40UP） |
| DSP | 7 | 數位訊號處理（僅 iCE40UP） |

### 邏輯拼塊 (Logic Tile)

每個 Logic Tile 佔用 7 個 Frame，包含 8 個 PLB。7 個 Frame 的內容分配如下：

- Frame 0-1：LUT 真值表（每個 PLB 的 16-bit LUT 需要跨 Frame 儲存）
- Frame 2-3：DFF 配置（時脈極性、致能、重置等控制位元）
- Frame 4-5：多工器選擇（MUX 用於將多個 LUT/DFF 輸出組合）
- Frame 6：繞線連接與其他雜項控制

每個 Frame 包含 33 words x 40 bits = 1,320 bits，因此一個 Logic Tile 的設定資訊共 7 x 1,320 = 9,240 bits。

### I/O 拼塊 (I/O Tile)

I/O Tile 位於晶片四週的邊界區域，僅佔用 3 個 Frame。每個 I/O Tile 包含多個 I/O 緩衝器（IOB, I/O Buffer），可獨立配置為輸入、輸出（含三態）、或雙向模式。I/O 的配置參數包括驅動強度（2 mA / 4 mA / 8 mA / 12 mA）、slew rate（快/慢）、上拉/下拉電阻、施密特觸發輸入（Schmitt trigger）等。

I/O Tile 的 Frame 數量（3）遠少於 Logic Tile，因為 I/O Tile 不含 LUT 真值表或 DFF 組合邏輯，其設定主要為多工器選擇、電氣參數控制、以及繞線連接。

### BRAM 拼塊（iCE40UP）

BRAM（Block RAM）拼塊佔用 14 個 Frame，恰好為 Logic Tile 的兩倍。每個 BRAM 拼塊提供 4 Kbit 的同步雙埠記憶體：

- **雙埠架構：** 擁有兩組完全獨立的位址匯流排、資料匯流排（可各自設定寬度）、以及時脈，支援兩個連接埠同時但非同步地存取記憶體陣列。
- **可配置資料寬度：** 支援 x1、x2、x4、x8、x16 等多種資料寬度模式，位址深度隨之調整。
- **ROM 模式：** 可預先載入初始化資料，作為唯讀記憶體使用。
- **FIFO 支援：** 透過額外的外部邏輯可實現同步 FIFO。

4 個 BRAM 拼塊總計提供 16 Kbit 的區塊記憶體。對於需要更大容量記憶體的設計，可使用分散式 RAM（LUT 實現）或 SPRAM（iCE40UP 專有）來補充。

### DSP 拼塊（iCE40UP）

DSP（Digital Signal Processing）拼塊同樣佔用 7 個 Frame，與 Logic Tile 相同。每個 DSP 拼塊包含一個硬體乘法器與累加器：

- **16x16 乘法器：** 支援有號（signed）與無號（unsigned）乘法操作，輸出為 32-bit 乘積。
- **32-bit 累加器：** 可將乘法結果與前一運算結果累加，實現乘加運算（multiply-accumulate, MAC）。
- **級聯支援：** 多個 DSP 拼塊可級聯以實現更高精度的運算或更高階的濾波器。
- **管線暫存器：** 內部包含可選擇的管線暫存器，用於提升最高運作頻率。

UP5K 內建 4 個 DSP 拼塊，適合實現 FIR 濾波器、FFT 運算、數位控制迴路等 DSP 應用。

## 拼塊網格與拓撲

iCE40 的晶片佈局為一個規律的二維網格，由 rows 列與 cols 行組成。網格的最外圍（第一列 row=0、最後一列 row=rows-1、第一行 col=0、最後一行 col=cols-1）均配置為 I/O Tile，內部網格則為 Logic Tile（並在 iCE40UP 中混合放置 BRAM 與 DSP Tile）。

此拓撲結構在 `v2f-pnr/src/arch.rs` 的 `ArchGraph::new()` 中有完整實作。`ArchGraph` 結構記錄了裝置類型、所有 tile 的座標與類型、以及邏輯區域的尺寸（logic_rows = total_rows - 2, logic_cols = total_cols）。以 HX8K 為例：

- 總網格：70 rows x 28 cols = 1,960 個 tile
- I/O Tile：70 x 2（左右邊界）+ 26 x 2（上下邊界，扣除四角重複）= 140 + 52 = 192 個
- Logic Tile：68 x 26 = 1,768 個

在佈局階段（placement），`ArchGraph::is_valid_placement()` 方法確保邏輯單元被放置在 Logic Tile 上，而 I/O 單元（如 SB_IO、SB_GB_IO）則被放置在 I/O Tile 上。這確保了佈局結果與晶片實體結構一致。

## 設定記憶體 (CRAM)

CRAM（Configuration RAM）是 iCE40 的核心設定儲存體，所有 LUT 真值表、繞線多工器選擇、DFF 配置、I/O 電氣參數等皆儲存於此。CRAM 的組織方式如下：

### Frame 結構

- **每個 Frame = 33 words x 40 bits = 1,320 bits = 165 bytes**
- 40 bits 的 word 寬度對應到 iCE40 CRAM 的實體陣列寬度
- 33 words 的深度構成了 Frame 的垂直維度

這些常數定義於 `Ice40Device` 的關聯常數中（`BITS_PER_FRAME=1320`, `WORDS_PER_FRAME=33`, `BITS_PER_WORD=40`, `FRAME_BYTES=165`）。

### 每列 Frame 數計算

由於 CRAM 採用 Row-major 組織方式，每一列的 Frame 數量由該列的 tile 配置決定：

```
每列 Frame 數 = (num_cols × 7) + io_left_frames + io_right_frames
```

其中 `io_left_frames` 與 `io_right_frames` 均為 3，對應於左右邊界 I/O Tile 的 Frame 數量。以 HX1K 為例：`num_cols=16`，每列 Frame 數 = 16x7 + 3 + 3 = 118。

### 總 Frame 數

```
總 Frame 數 = num_rows × frames_per_row
```

各裝置的總 Frame 數計算如下：

| 裝置 | rows | cols | frames_per_row | total_frames | CRAM 總位元數 |
|---|---|---|---|---|---|
| HX1K / LP1K | 30 | 16 | 118 | 3,540 | 4,672,800 bits (571 KB) |
| HX4K | 40 | 20 | 146 | 5,840 | 7,708,800 bits (941 KB) |
| UP5K | 40 | 22 | 160 | 6,400 | 8,448,000 bits (1,031 KB) |
| HX8K | 70 | 28 | 202 | 14,140 | 18,664,800 bits (2,279 KB) |

CRAM 的總位元數即為 FPGA 設定的最小資訊量，這也對應於燒錄檔案（BIN）的理論最小尺寸（在壓縮前）。

## CRAM 位址對映 (Address Mapping)

eda4 的 `v2f-db/src/cram_addr.rs` 實現了 CRAM 位址對映的核心邏輯，由 `CramAddrMap` 結構提供。該結構封裝了裝置參數，提供兩個關鍵方法：

### tile_start_frame()：定位 Tile 起始 Frame

給定 tile 座標 `(row, col)` 與 tile 類型，計算該 tile 在 CRAM 中的起始 Frame 編號：

```
start_frame = row × frames_per_row + col × tile.num_frames()
```

其中 `row × frames_per_row` 為列偏移量（跳過前面所有列的所有 tile），`col × tile.num_frames()` 為行偏移量（跳過該列中前面的所有 tile）。

### resolve()：定位到具體位元

給定 tile 座標、tile 類型、tile 內 Frame 偏移（frame_within_tile）、word 索引、bit 索引，組合成完整的 `CramAddr`：

```
CramAddr {
    frame: start_frame + frame_within_tile,
    word: word,
    bit: bit,
}
```

`CramAddr` 的三個字段（frame, word, bit）即可唯一標識 CRAM 中的任何一個設定位元。在 `v2f-bitstream` crate 的 ASC 解析與 BIN 打包過程中，即利用此對映將每個設定位元寫入正確的位置。

## 繞線資源

iCE40 的可程式化互連網絡（programmable interconnect）是實現晶片內部任意連接的關鍵。繞線資源位於 tile 之間的水平與垂直通道中。

### 繞線層次

iCE40 的繞線分為多種長度類別：

- **單 tile 繞線 (Single / Direct)：** 毗鄰 tile 之間的直接連接，延遲最小，用於局部邏輯連接。
- **雙 tile 繞線 (Double)：** 跨越兩個 tile 距離的中等長度繞線，平衡了延遲與靈活性。
- **長線 (Long Line / Span)：** 可跨越整個晶片的高度或寬度，用於時脈、重置、全域致能等高扇出訊號。長線具有較低的 RC 延遲，適合長距離傳輸。

### 開關盒 (Switch Box)

每個 tile 的交叉點設有開關盒，由 CRAM bit 控制的可程式化傳輸閘（pass transistor）構成。開關盒決定了水平繞線與垂直繞線之間的連通性：

- 一個開關點可連接來自四個方向（北、南、東、西）的繞線
- 每種連接組合由一個獨立的 CRAM bit 控制
- 典型拓撲為 disjoint、Wilton、或 universal 開關盒（iCE40 使用基於這三種的子集）

### 連接盒 (Connection Box)

連接盒位於 PLB 輸入/輸出引腳與繞線通道之間。每個 PLB 的輸入引腳可透過連接盒從多條相鄰的繞線中選擇訊號來源：

- 輸入 MUX：每個 PLB 輸入可從若干條水平與垂直繞線中選擇
- 輸出 MUX：PLB 輸出可驅動多條相鄰繞線，向外傳播訊號

連接盒的配置同樣由 CRAM bit 控制，構成繞線設定資訊的主要組成部份。

### 時脈網絡

iCE40 擁有專門的時脈分佈網絡，將時脈訊號低偏移（low skew）地分配到所有 DFF：

- **全域時脈緩衝器 (Global Clock Buffer)：** 位於晶片四角的 SB_GB 元件，可將外部時脈輸入或 PLL 輸出分配至全域時脈網路。
- **時脈樹 (Clock Tree)：** H 型分佈的時脈樹，確保時脈到達各 DFF 的延遲差異最小。
- **時脈域：** iCE40 支援多個獨立的時脈域，每個時脈域可由不同的時脈來源驅動。

### 繞線與 CRAM 位元的關係

繞線設定是 CRAM 中佔用位元數最多的部份，遠超過 LUT 真值表。每一個開關點與連接點都對應到一個 CRAM bit。典型的 FPGA 設計中，繞線位元約佔總 CRAM 容量的 70-80%，這也是 FPGA 架構的普遍特徵——可程式化互連的代價即為大量的設定位元。

## 輸入輸出 (I/O)

iCE40 的 I/O 電路提供豐富的可程式化選項，使其能與各種外部電路介接。

### I/O 標準

每個 I/O bank 可獨立設定供電電壓（VCCIO），支援的電壓標準包括：

- **LVCMOS33 (3.3V)：** 最常見的標準，相容於 3.3V 邏輯
- **LVCMOS25 (2.5V)：** 用於 DDR 記憶體等 2.5V 介面
- **LVCMOS18 (1.8V)：** 低壓邏輯介面
- **LVCMOS15 (1.5V)：** 超低電壓邏輯
- **LVCMOS12 (1.2V)：** 極低電壓邏輯（部份型號支援）

### 差分訊號

特定 I/O 對可配置為差分訊號對：

- **LVDS (Low-Voltage Differential Signaling)：** 高速低功耗差分傳輸，資料速率可達數百 Mbps
- **Sub-LVDS：** 更低擺幅的差分標準，用於影像感測器介面

### I/O 電氣特性

- **驅動強度：** 可選擇 2 mA、4 mA、8 mA、12 mA，平衡訊號完整性與功耗
- **Slew Rate：** 快速模式（fast）用於高速訊號，慢速模式（slow）用於減小 EMI
- **上拉/下拉電阻：** 內部可程式化的 pull-up 與 pull-down 電阻（約 50-100 kΩ）
- **施密特觸發：** 可啟用 Schmitt trigger 輸入以增強雜訊容忍度

### PLL（鎖相迴路）

HX 與 UP 系列內建 PLL，提供時脈管理功能：

- **頻率合成：** 將外部參考時脈乘以可程式化的倍率（M）再除以分頻係數（N），產生目標頻率
- **多個輸出：** 每個 PLL 可產生多個不同頻率與相位的輸出時脈
- **相位偏移：** 支援 0、90、180、270 度的相位調整，或更精細的延遲調整
- **鎖定指示：** PLL 鎖定位（PLL_LOCK）指示時脈已穩定

HX 系列通常配備 1 個 PLL，UP5K 配備 2 個 PLL。LP 系列則不包含 PLL，僅能使用外部時脈或內部 RC 振盪器。

### I/O 映射與佈局

在佈局繞線階段，`ArchGraph::is_valid_placement()` 確保 `SB_IO`、`SB_GB_IO` 等 I/O 元件僅被放置在 I/O Tile 上，而一般的邏輯元件（`$_DFF_P_`、`$and` 等）則應放置在 Logic Tile 上。違反此規則的佈局會被 PNR 引擎拒絕。

## 特殊功能

### SPI Flash 設定

iCE40 使用 SPI 協定從外部序列 Flash 載入設定資料。支援的設定模式包括：

- **Master SPI：** FPGA 作為 SPI master，主動從 SPI Flash 讀取設定資料。這是最常用的模式，上電後 FPGA 自動透過 SCK、SI、SO、CS 等引腳讀取 Flash。
- **Slave SPI：** FPGA 作為 SPI slave，由外部處理器（MCU、SoC）將設定資料寫入 FPGA。此模式適合系統動態重配置的場景。
- **寬度模式：** 支援標準 SPI（1-bit）與雙倍資料率（Dual SPI）模式。

### Warm Boot（動態重配置）

Warm Boot 是 iCE40 的重要功能之一，允許在不斷電的情況下重新載入 FPGA 設定：

- **雙映像支援：** 外部 SPI Flash 可儲存兩份設定映像（image A 與 image B），分別位於不同的位址範圍。
- **切換機制：** 透過特定的 FPGA 引腳（BOOT）或內部邏輯觸發重新設定，並選擇載入哪一個映像。
- **應用場景：** 系統現場升級（field upgrade）時，寫入新設定至備用區域，確認成功後再切換；或在多模式系統中根據運行環境切換功能。

Warm Boot 的配置位元同樣儲存在 CRAM 中，因此每次載入映像時可選擇不同的 Warm Boot 行為。

### 晶片內建 RC 振盪器

iCE40 內建 RC 振盪器（On-Chip Oscillator, OSC），標稱頻率約 10 MHz（實際頻率因製程與溫度而變化，約在 6-18 MHz 範圍）。此振盪器的用途包括：

- 提供初始設定載入時的時脈來源
- 為低速邏輯提供時脈，無需外部晶振
- 作為系統啟動時的暫態時脈，等待 PLL 鎖定

OSC 可透過 CRAM 配置為啟用或停用，其輸出可經由分頻後饋入內部時脈網路。

### RGB LED 驅動器（UP 系列）

iCE40UP 系列內建 RGB LED 驅動器，可直接驅動外部 RGB LED（紅、綠、藍），無需外部驅動 IC 或 PWM 控制器：

- 三個 PWM 通道分別控制 R、G、B 的亮度
- PWM 頻率與解析度可程式化
- 驅動強度經過最佳化，可直接連接常見的 LED

此功能使得 iCE40UP 特別適合用於 LED 指示、裝飾燈效、使用者互動介面等消費性電子應用。

### NVCM（非揮發性設定記憶體）

部份 iCE40 型號（如 LP8K 與 LM 系列）內建 NVCM（Non-Volatile Configuration Memory），可在晶片內部永久儲存設定：

- **快閃技術：** 基於 Flash 記憶體，斷電後資料不遺失
- **即時啟動：** 開機後直接從 NVCM 載入設定，無需等待外部 SPI Flash 初始化
- **重寫次數：** 支援有限次數的重新燒錄（通常約 1,000 次）
- **應用場景：** 要求即時啟動的系統（如汽車電子、工業控制），或對外部 SPI Flash 成本敏感的設計

NVCM 與 CRAM 的關係是：NVCM 儲存原始設定數據，開機時自動複製到 CRAM 中執行。NVCM 的內容可透過 JTAG 或 SPI 介面燒錄。

## 電源架構

iCE40 使用多個獨立的電源域以優化功耗與效能：

- **VCC (1.2V)：** 核心電壓，供給所有內部邏輯（LUT、DFF、繞線開關）與 CRAM。1.2V 的較低電壓有效降低動態功耗與靜態漏電。
- **VCCIO (1.2V / 1.5V / 1.8V / 2.5V / 3.3V)：** I/O 銀行電壓，每個 I/O bank 可獨立設定。VCCIO 的電壓值決定了 I/O 引腳的邏輯準位。
- **VPP (2.5V)：** NVCM 燒錄電壓，僅在具有 NVCM 的型號中需要，僅在燒錄時供電。
- **VCCT (2.5V)：** 輔助電壓，用於 PLL 類比電路等內部類比模組。
- **VCCSPI (1.8V / 2.5V / 3.3V)：** SPI 設定介面的專用電壓域，可與核心及其他 I/O 電壓獨立。

iCE40 LP 系列的靜態功耗可低至 100 µA 以下（典型條件下），這得益於其 1.2V 核心電壓與先進的低功耗製程技術。HX 系列的靜態功耗稍高（約 1-2 mA），但提供更高的運作頻率。

## 時脈架構

iCE40 的時脈分佈系統確保低偏移與高可靠性的時脈傳輸：

### 時脈來源

可作為時脈來源的訊號包括：

- 外部時脈輸入引腳（GBINx, GBIx）
- PLL 輸出（僅 HX/UP 系列）
- 內部 RC 振盪器
- 內部邏輯產生的時脈（時脈閘控）

### 全域時脈緩衝器

SB_GB（Global Buffer）元件是 iCE40 的全域時脈驅動器。位於 I/O Tile 或角落位置的 SB_GB 可被分配至佈局繞線工具，以驅動整個晶片的時脈網路。

### 時脈區域

iCE40 的時脈網路將晶片劃分為多個時脈區域。每個時脈區域內的 DFF 可共用同一時脈源。時脈區域的數量與劃分方式取決於裝置尺寸。

### 重置網絡

除時脈網路外，iCE40 也提供全域重置（global reset）網路，用於將所有 DFF 初始化為已知狀態。重置信號可來自外部重置引腳、PLL 鎖定指示器、或內部邏輯。

## iCE40UP 先進功能

iCE40 UltraPlus (UP) 系列在基本架構上增加了多項硬體單元，是 iCE40 家族中功能最先進的子系列。

### 硬體乘法器與 DSP

UP5K 的 4 個 DSP 拼塊提供固定功能的數位訊號處理能力。每個 DSP 拼塊包含：

- 16x16 硬體乘法器，單週期輸出 32-bit 乘積
- 32-bit 累加器，支援累加與加減法
- 可程式化的管線深度（0-2 級），平衡延遲與吞吐量
- 級聯支援，多 DSP 可串接實現更高精度

典型用途包括 FIR 濾波器（係數 8-16 階可單晶片實現）、IIR 濾波器、DDS（直接數位合成）、數位控制振盪器（NCO）等。

### 區塊記憶體 (BRAM)

BRAM 提供效率遠高於分散式 RAM（LUT 實現）的儲存方案：

- 每個 BRAM 容量 4 Kbit，總計 16 Kbit
- 真正的雙埠（True Dual-Port），兩個連接埠可同時獨立存取
- 連接埠 A 與 B 可設定不同的資料寬度（如 A 為 x8、B 為 x16）
- 可選的註冊輸出（registered output）以改善時序

### SPRAM（單埠 SRAM）

UP5K 內建 1 個 256 Kbit 的 SPRAM，遠大於 BRAM 的總容量：

- **單埠架構：** 單一連接埠，每次一個存取操作
- **高速存取：** 最多可達 200 MHz 以上的操作頻率
- **應用場景：** 視訊幀緩衝器、音訊樣本緩衝器、通訊封包緩衝區、嵌入式處理器的工作記憶體

SPRAM 與 BRAM 的互補使用，使得 UP5K 足以勝任需要相當程度晶片內記憶體的應用，如簡單的微控制器系統、資料緩衝器等。

### 更多邏輯與 I/O

UP5K 的 40x22 網格賦予其 5,280 個邏輯單元與豐富的 I/O 資源，在 iCE40 系列中僅次於 HX8K。加上 DSP 與 BRAM，UP5K 可謂「小晶片、大功能」，適合需要在一顆小型 FPGA 內整合邏輯、記憶體、與數位訊號處理的應用。

## 開機設定流程

iCE40 的上電設定流程（Power-Up Sequence）如下：

1. **上電重置 (POR)：** 核心電壓達到穩定後，POR 電路釋放晶片重置狀態。
2. **設定來源選擇：** 根據外部引腳（如 SS、SPI 等）的電壓準位，決定設定來源（Master SPI、Slave SPI、NVCM、或 JTAG）。
3. **設定載入：** 從選定的來源讀取設定資料，寫入 CRAM。載入過程中所有 I/O 處於高阻抗狀態。
4. **CRAM 驗證：** 可選的 CRC 驗證步驟（取決於 CRAM 中的配置位元），確保設定資料的正確性。
5. **啟用 I/O：** 設定載入完成後，I/O 引腳按照設定中的配置啟用。
6. **進入使用者模式：** FPGA 開始執行使用者邏輯，依使用者模式正常運行。

整個設定流程的時間取決於設定資料量與 SPI 時脈頻率。以 HX1K 為例，CRAM 約 571 KB，在 10 MHz SPI 時脈下約需 5 ms 完成載入。

## 開放原始碼工具鏈與生態系統

iCE40 在開放原始碼 FPGA 工具鏈領域具有里程碑意義：

### Project IceStorm (2015)

Claire Wolf 於 2015 年發起 Project IceStorm，成功逆向工程 iCE40 的 CRAM 位元流格式。該專案釋出了：

- **icepack：** BIN 位元流打包/解包工具
- **iceprog：** SPI Flash 燒錄工具
- **icetime：** 時序分析工具
- **icebram：** BRAM 初始化資料工具
- 完整的 CRAM 位址對映文件

IceStorm 的成果使得第三方工具鏈能夠產生 iCE40 可執行的位元流，無需依賴 Lattice 的專有工具 Diamond 或 iCEcube2。

### Yosys + nextpnr

在此基礎上衍生出兩大關鍵工具：

- **Yosys：** Claire Wolf 開發的 Verilog 合成框架，支援 iCE40 為目標架構的合成（使用 `synth_ice40` pass）。
- **nextpnr-ice40：** David Shah 開發的架構無關開源 PNR 工具，iCE40 是其最初支援的架構。支援 A* 繞線演算法與模擬退火佈局演算法。

### eda4 純 Rust 工具鏈

eda4 專案進一步推進了這一方向，以 Rust 語言從零實作完整的 iCE40 EDA 工具鏈：

- **v2f-synth：** 純 Rust 的 Verilog 合成器，將 Verilog 解析為 JSON 格式的 netlist
- **v2f-pnr：** 佈局繞線引擎，使用模擬退火演算法（simulated annealing）進行佈局，並實現了開關盒繞線
- **v2f-bitstream：** ASC 格式解析與 BIN 位元流打包，基於 v2f-db 的 CRAM 位址對映
- **v2f-programmer：** 支援 mock 模式與選用的 ftdi 功能（透過 FTDI 晶片實現實際燒錄）
- **v2f-db：** iCE40 裝置資料庫，包含所有型號的 geometry、CRAM 位址對映、以及 tile 類型定義
- **v2f-rust：** 創新的 `fpga!` DSL，允許以 Rust 語法直接描述硬體

eda4 支援三種後端模式：自動（auto，優先使用 yosys/nextpnr/icepack，必要時回退至純 Rust）、純 Rust（pure-rust，完全使用 eda4 自帶工具）、以及 yosys 模式。

## 參考資料

- Project IceStorm: https://github.com/YosysHQ/icestorm
- Yosys Open SYnthesis Suite: https://github.com/YosysHQ/yosys
- nextpnr: https://github.com/YosysHQ/nextpnr
- Lattice iCE40 LP/HX Family Data Sheet (DS1040)
- Lattice iCE40 UltraPlus Family Data Sheet (DS1048)
- Lattice iCE40 Programming and Configuration Technical Note (TN1250)
- eda4 原始碼: verilog2fpga/v2f-db/src/

## 延伸閱讀

- [Lattice iCE40 (Wikipedia)](https://en.wikipedia.org/wiki/Lattice_iCE40)
- [Field-Programmable Gate Array (Wikipedia)](https://en.wikipedia.org/wiki/Field-programmable_gate_array)
- [SRAM (Wikipedia)](https://en.wikipedia.org/wiki/Static_random-access_memory)
- [Flip-Flop (Electronics) (Wikipedia)](https://en.wikipedia.org/wiki/Flip-flop_(electronics))
