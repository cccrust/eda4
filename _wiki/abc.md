# ABC — 序列合成與驗證系統

ABC 是加州大學柏克萊分校開發的開源邏輯合成、最佳化與驗證工具。自 2005 年發布以來，ABC 已成為學術界與開源社群中標準的邏輯合成系統，廣泛應用於數位電路設計、FPGA 合成與形式驗證。

## 什麼是 ABC

ABC 是一個以 **AIG（And-Inverter Graph）** 為核心表示法的邏輯合成工具。它讀入網表（BLIF、Verilog、AIGER 格式），執行一系列邏輯最佳化與技術映射（technology mapping）操作，最後輸出最佳化後的網表。

ABC 提供一個**互動式命令列介面**，使用者可以逐個執行合成指令，也可以編寫腳本來自動化整個流程。此外，ABC 也提供 C 語言 API，可以嵌入到其他工具中（最著名的例子是 yosys 的 `abc` 指令）。

## 歷史沿革

ABC 的發展繼承了多年來的邏輯合成研究成果：

- **SIS (1990s)**：由柏克萊開發的邏輯合成系統，使用 **SOP（Sum-of-Products）** 和 **BCD（Boolean Cover Diagram）** 作為核心表示。SIS 是 1990 年代邏輯合成的標準工具，但面臨可擴展性問題。
- **MVSIS (2000s)**：在多值邏輯（multi-valued logic）方向的延伸，但未獲得廣泛採用。
- **ABC (2005+)**：Alan Mishchenko 在柏克萊主導開發，從零開始重新設計，採用 AIG 作為核心表示法。ABC 在設計之初就將**可擴展性**作為首要目標，使其能夠處理數百萬閘級別的電路。

ABC 的發展至今仍持續進行，包含新的最佳化演算法、驗證技術和對新興硬體架構的支援。

## 核心表示：AIG（And-Inverter Graph）

AIG 是 ABC 的核心資料結構，也是其高效能的主要原因。

### AIG 的結構

- **節點（node）**：每個非輸入節點代表一個 2 輸入的 AND 閘。
- **反相器（inverter）**：以邊上的屬性（attribute）表示，而非獨立的 NOT 節點。這意味著反相器不消耗任何節點，僅在邊上做標記。
- **輸入**：主輸入（primary input, PI）和暫存器輸出（latch/FF output）均作為 AIG 的輸入節點。
- **輸出**：主輸出（primary output, PO）和暫存器輸入（latch/FF input）從 AIG 內部節點引出。
- **時序邊界**：暫存器在 AIG 中被視為輸入/輸出對——暫存器的輸出是 AIG 的輸入，暫存器的輸入是 AIG 的輸出。這將時序電路的驗證和最佳化問題簡化為組合電路的多次迭代。

### AIG 的優點

- **結構性雜湊（Structural Hashing）**：透過 `strash` 指令，ABC 自動合併結構上等價的 AND 節點。這使得 AIG 在**結構化**意義上具有正規化特性（儘管不等同於 BDD 的函數正規化）。
- **緊湊性**：每個布林函數可以用約 1–2 個 AIG 節點來表示。例如：
  - AND：1 個節點
  - OR：2 個節點（NOT 不計）
  - XOR：3 個節點
  - MUX：4 個節點
- **操作速度**：AIG 的操作（如節點替換、子圖匹配）非常快，因為不涉及 BDD 的遞迴操作或 SOP 的展開。
- **可擴展性**：AIG 可以輕鬆處理數百萬閘的設計，而 BDD 在這種規模下通常會耗盡記憶體。

### AIG vs BDD vs SOP

| 特性 | AIG | BDD | SOP |
|------|-----|-----|-----|
| 表示形式 | DAG | DAG | 覆蓋表 |
| 正規化 | 結構性（strash） | 函數性 | 否 |
| 空間效率 | 極好（百萬閘級） | 好（有限輸入） | 中等 |
| 操作效率 | 極好 | 好 | 好（Espresso） |
| 適用規模 | 極大規模 | 中等規模 | 中等規模 |
| 重寫/重組 | 極快 | O(BDD大小) | O(SOP大小) |
| FPGA 映射 | 標準 | 適合 LUT | 不直接 |

AIG 之所以成為現代邏輯合成的首選表示法，是因為它在**可擴展性**和**操作效率**之間取得了最佳平衡。BDD 雖然函數操作更強大，但無法擴展到超大規模；SOP 雖然對某些最佳化（如 espresso）很高效，但缺乏正規化支援。

## ABC 核心指令

ABC 的互動式命令列提供了豐富的指令集。以下是最常用的指令分類：

### 輸入/輸出

- **read**：讀入 BLIF 格式的設計檔案。
- **read_aiger**：讀入 AIGER 格式（二進位或 ASCII）。
- **read_verilog**：讀入 Verilog 檔案（需要編譯時啟用 Verilog 解析器）。
- **write_blif**：輸出 BLIF 格式的網表。
- **write_verilog**：輸出 Verilog 格式的網表。
- **write_aiger**：輸出 AIGER 格式。
- **print_stats**：列印當前網表的統計資訊（節點數、層級數、輸入/輸出數等）。

### AIG 建構與基本操作

- **strash**：執行結構性雜湊（structural hashing），建構 AIG 並合併等價節點。這是讀入設計後的第一步。
- **balance**：對 AIG 進行結構重組以減少邏輯深度。使用結合律和交換律將 AND 樹重新平衡為最淺的形式。
- **cleanup**：移除 unreachable 節點和冗餘節點。
- **sweep**：移除未被使用的節點和驅動常數的節點。

### 邏輯最佳化

- **rewrite**：AIG 重寫（rewriting）。使用預先計算的「最佳」子圖模式來替換 AIG 中的子圖，以減少節點計數或深度。這是 ABC 最具特色的最佳化技術之一。
- **refactor**：布林重構（Boolean refactoring）。對節點進行較大規模的函數重組，可能改變電路的拓撲結構。與 rewrite 相比，refactor 的變動更大。
- **resub**：重代入（resubstitution）。檢查一個節點是否可以用已經存在的節點來表示，從而減少節點數。這是克服局部最佳化限制的關鍵技術。
- **dc2**：基於無關項（don't-care）的最佳化。利用 satisfiability don't care（SDC）和 observability don't care（ODC）來簡化邏輯。
- **compress**：結合 rewrite、refactor、resub 和 balance 的綜合最佳化腳本，使用多輪迭代來最大限度減少節點數。
- **compress2**：更積極的壓縮腳本，適用於對面積要求嚴格的場景。

### FPGA 技術映射

- **if**：FPGA 技術映射器（technology mapper），將 AIG 映射到 K-LUT（K 輸入查閱表）。
  - 支援不同的 LUT 大小，如 `if -K 4` 將映射到 4-LUT。
  - 支援混合 LUT 大小，如 `if -K 4:6` 允許使用 4 到 6 輸入的 LUT。
  - 支援面積最佳化模式、深度最佳化模式和混合模式。
  - 支援暫存器重定時（register retiming）整合。
- **lutpack**：LUT 打包，將多個小 LUT 合併為一個大 LUT。這對於提高 LUT 利用率和減少路徑延遲很有幫助。
- **choice**：透過建立「選擇節點（choice node）」來表示多種等價實現，讓後續的映射器可以從中選擇最佳方案。
- **fx**：XOR 主導邏輯的萃取與合成。對算術電路（加法器、乘法器）特別有效，因為這類電路大量使用 XOR。

### 驗證

- **cec**：組合等效檢查（Combinational Equivalence Checking）。比較兩個電路是否在組合層次上等價。使用 `cec -s` 進行順序等效檢查（透過暫存器配對）。
- **dprove**：序列驗證（sequential verification），使用 PDR（Property Directed Reachability）/IC3 演算法。可以驗證時序屬性（如安全屬性、LTL 屬性）。
- **pdr**：與 dprove 類似，專注於 PDR/IC3 演算法。
- **fraig**：基於 SAT 的錯誤類比（fault simulation）與冗餘識別。

### 其他指令

- **synth**：執行標準合成腳本（`resub; resub; rewrite; refactor; rewrite; resub; rewrite; balance; if` 等步驟的組合）。
- **sop**：SOP 化簡，使用類似 Espresso 的演算法。
- **fm**：扇出最佳化（fanout optimization）。
- **mfs**：最小扇出分離（minimum fanout separation）。
- **dch**：分解（decomposition），用於大扇入節點的分解。

## ABC 標準合成流程

一個典型的 ABC 合成腳本如下：

```
# 步驟 1：讀入設計
read input.blif
# 步驟 2：建構 AIG 並執行結構性雜湊
strash
# 步驟 3：多輪邏輯最佳化
balance
rewrite
refactor
rewrite
resub
rewrite
balance
# 步驟 4：FPGA 技術映射到 4-LUT
if -K 4
# 步驟 5：LUT 打包
lutpack
# 步驟 6：輸出
write_blif output.blif
```

這個流程對應於 `synth` 指令的標準行為。

### 從粗糙邏輯到最佳化網表的步驟

1. **讀入與 AIG 化**：將 BLIF/Verilog 讀入，轉換為 AIG，透過 `strash` 實現結構性雜湊。
2. **邏輯最佳化**：透過多輪的 `rewrite`、`refactor`、`resub` 和 `balance` 來減少節點數量和邏輯深度。這個階段通常迭代多次以達到收斂。
3. **技術映射**：使用 `if` 將 AIG 映射到目標技術庫（標準細胞或 LUT）。
4. **後處理**：包含 `lutpack`、`choice`、`fm` 等進一步最佳化。

## Yosys 如何使用 ABC

Yosys 是目前最流行的開源 Verilog 合成工具。它內部整合了 ABC 作為其後端最佳化引擎：

### 標準細胞映射

```tcl
# Yosys 腳本：使用 ABC 進行標準細胞映射
synth -top top_module
abc -g AND,OR,XOR,NAND,NOR,NOT,BUF,MUX,FA
```

`abc` 指令的 `-g` 參數指定了目標標準細胞庫中的閘類型。ABC 會將 AIG 映射到這些閘的組合上。

### FPGA LUT 映射

```tcl
# Yosys 腳本：使用 ABC 進行 LUT 映射
synth -top top_module
abc -lut 4:6
```

`-lut 4:6` 告訴 ABC 使用 4 到 6 輸入的 LUT。這對於 FPGA 合成非常關鍵——yosys 自身的 LUT 映射能力有限，主要依賴 ABC 的 `if` 映射器。

### ABC 指令傳遞

```tcl
# 將自訂 ABC 命令傳遞給 ABC
abc -script "strash; if -K 4; lutpack"
```

透過 `-script` 參數，使用者可以將任意 ABC 指令傳遞給 ABC 執行。

## ABC 在 eda4 中的角色

在 eda4 專案的 `verilog2fpga` 中，ABC 通過 yosys 間接使用：

### 使用 yosys 後端（--backend yosys）

當使用者指定 `--backend yosys` 時，`v2f` CLI 的流程如下：

1. `v2f-synth` 將 Verilog 轉換為中間表示。
2. 呼叫 yosys 執行 Verilog 解析與初步合成。
3. yosys 內部呼叫 ABC 進行 AIG 最佳化與 LUT 映射。
4. ABC 輸出最佳化後的網表（BLIF 格式）。
5. `v2f-pnr` 讀入 BLIF 進行佈局佈線。

### 使用純 Rust 後端（--backend pure-rust）

當使用純 Rust 後端時，流程為：

1. `v2f-synth` 直接對 Verilog 進行解析和技術映射，生成網表。
2. 技術映射是基於規則的——逐個 Verilog 原語（assign、always、case 等）映射到 iCE40 邏輯單元。
3. LUT init 的計算使用真值表直接建構，而非透過 AIG/BDD 最佳化。
4. 生成的 JSON 網表直接傳送給 `v2f-pnr` 進行佈局佈線。

### 純 Rust 與 yosys+ABC 的比較

| 特性 | 純 Rust（v2f-synth） | yosys + ABC |
|------|----------------------|-------------|
| 邏輯最佳化 | 無多層次最佳化 | 多層次 AIG 重寫 |
| LUT 映射 | 直接規則映射 | 最佳化 AIG 覆蓋 |
| 面積效率 | 中等 | 好（~20–40% 更少 LUT） |
| 性能 | 快速（無最佳化開銷） | 較慢（最佳化耗時） |
| 依賴 | 無外部依賴 | 需要 yosys、ABC |
| 正確性參考 | 需要與 yosys 比對 | 業界標準 |

純 Rust 合成的設計目標是**功能等效性**與**低依賴性**，而非最佳化強度。yosys+ABC 管線則作為生產環境的參考實作，提供最佳的合成品質。

## ABC 範例：blinky 最佳化

以下是一個針對 `blinky` 設計的簡單 ABC 腳本：

```
# read the synthesized AIG
read blinky.blif
# structural hashing
strash
# print initial stats
print_stats
# AIG rewriting (multi-pass)
balance
rewrite -z  # -z = zero-cost mode (no area increase)
refactor -z
rewrite -z
balance
# print optimized stats
print_stats
# FPGA technology mapping into 4-LUTs
if -K 4 -a  # -a = area-oriented mapping
# output
write_blif blinky_opt.blif
```

這個腳本可以減少約 15–30% 的 LUT 數量（取決於原始設計的品質）。

## ABC 驗證：等效檢查與序列驗證

### 組合等效檢查（cec）

```abc
# 讀入兩個電路進行比較
read circuit1.blif
strash
# 將 circuit1 存為備份
miter circuit2.blif  # 建立 miter 電路
cec
```

`cec` 指令透過建立 miter 電路（兩個電路的輸出進行 XOR）並檢查其是否可被滿足來判斷等效性。如果 miter 電路產生 1 的可能性被證明不存在，則兩個電路等效。

### 序列等效檢查（dprove）

```abc
read sequential1.blif
strash
dprove sequential2.blif
```

`dprove` 使用 PDR/IC3 演算法進行序列驗證。它可以處理帶有暫存器的時序電路，驗證兩個時序電路是否在相同的輸入序列下產生相同的輸出序列。

## 為什麼 ABC 對了解 eda4 很重要

1. **正確性參考**：yosys→ABC 管線是驗證純 Rust 合成結果正確性的黃金參考。任何對 `v2f-synth` 的修改都應與 yosys+ABC 的輸出進行等效檢查。

2. **效能基準**：ABC 的最佳化品質為 `v2f-synth` 提供了追求目標。雖然純 Rust 合成目前沒有實現高階最佳化，但 ABC 的輸出展示了在面積和延遲方面可以達到的上限。

3. **架構理解**：理解 ABC 的 AIG 最佳化技術（rewrite、refactor、resub、dc2）有助於未來在 `v2f-synth` 中實作類似的 Rust 原生最佳化。

4. **未來整合**：如果未來 eda4 需要更高品質的合成輸出，可以考慮直接整合 ABC 的 C 函式庫（透過 FFI 綁定），而不是依賴外部 yosys 程序呼叫。

## ABC 指令實戰範例

以下展示幾個常見的 ABC 使用場景。這些範例可以在 ABC 互動式命令列中執行。

### 場景一：讀入並了解一個設計

```
abc> read design.blif
abc> strash
abc> print_stats
design           : i/o =  8/ 1  lat = 0  nd = 127  edge = 0
area = 127.00  delay = 14.00  level = 7
```

`print_stats` 輸出顯示：8 個輸入、1 個輸出、0 個暫存器、127 個 AIG 節點、邏輯深度為 7。

### 場景二：逐步觀察最佳化效果

```
abc> read design.blif
abc> strash
abc> print_stats          # 初始：127 節點
abc> balance
abc> print_stats          # balance 後：115 節點
abc> rewrite -z
abc> print_stats          # rewrite 後：98 節點
abc> refactor -z
abc> print_stats          # refactor 後：87 節點
abc> balance
abc> print_stats          # final：82 節點（最佳化 35%）
```

對 AIG 節點計數的逐步觀察有助於理解每個指令的實際效果。

### 場景三：LUT 映射與面積比較

```
abc> read design.blif
abc> strash; balance; rewrite; balance
abc> if -K 4 -a           # 面積最佳化映射
abc> print_stats
LUT mapping:  lut = 28  level = 5
abc> if -K 4 -d           # 深度最佳化映射
abc> print_stats
LUT mapping:  lut = 35  level = 3
```

面積最佳化得到 28 個 LUT、深度 5；深度最佳化則用更多 LUT（35 個）換取了更小的深度（3）。這是典型的 area-delay tradeoff。

### 場景四：等效檢查

```
abc> read golden.blif
abc> strash
abc> write_aiger golden.aig
abc> read revised.blif
abc> strash
abc> write_aiger revised.aig
abc> cec golden.aig revised.aig
Networks are equivalent.
```

如果兩個電路不等效，`cec` 會輸出反例（counterexample），包含一組能區分兩個電路的輸入賦值。

### 場景五：使用腳本進行批次處理

```sh
# 命令列批次模式
abc -c "read design.blif; strash; balance; rewrite; if -K 4; write_blif opt.blif"

# 或使用腳本檔案
abc -f synth_script.abc
```

其中 `synth_script.abc` 內容為 ABC 指令序列，每行一條。

## ABC 與其他工具的比較

### ABC vs Yosys

| 特性 | ABC | Yosys |
|------|-----|-------|
| 核心表示 | AIG | RTLIL（內部）+ AIG（經由 ABC） |
| 主要功能 | 邏輯最佳化、映射、驗證 | 完整合成流程（RTL 到網表） |
| Verilog 解析 | 有限支援 | 完整支援（AST 解析） |
| FPGA 映射 | 強大（if 映射器） | 有限（依賴 ABC） |
| 腳本語言 | 指令列 | Tcl |
| 技術庫支援 | 通用 + ABC 格式 | Liberty + ABC 格式 |
| 應用場景 | 最佳化引擎 | 完整合成工具 |

ABC 和 Yosys 並非競爭關係，而是互補關係。Yosys 處理前端（RTL 解析、高階最佳化），ABC 處理後端（閘級最佳化、技術映射）。兩者透過 BLIF 和 AIGER 格式互通。

### ABC vs SIS

SIS 是 1990 年代的邏輯合成系統，代表了傳統的 SOP 為中心的合成方法：

| 特性 | ABC | SIS |
|------|-----|-----|
| 核心表示 | AIG（節點級） | SOP（cube 級） |
| 可擴展性 | 百萬閘 | 萬閘級別 |
| 速度 | 快 | 慢 |
| 演算法 | DAG-aware 重寫 | 基於 BDD/SOP 的化簡 |
| 技術映射 | 基於 cuts | 基於樹覆蓋 |
| 維護狀態 | 活躍開發 | 已停止 |

ABC 相較於 SIS 在可擴展性和效率上有數量級的提升。

## 取得與編譯 ABC

ABC 的原始碼託管於 GitHub：

```
git clone https://github.com/berkeley-abc/abc.git
cd abc
make
```

編譯後的二進位檔案約 1–2 MB，無需安裝即可執行。ABC 也透過包管理器分發：

- macOS：`brew install abc`（第三方 formula）
- Linux：透過 yosys 的依賴安裝（`yosys-abc` 套件）

在 eda4 專案中，ABC 透過 yosys 的 `abc` 指令間接使用。如果系統中安裝了 yosys，ABC 二進位檔案（`yosys-abc` 或 `abc`）通常已可用。

## 進階主題

### 使用 Python 操作 ABC

透過 PyABC 或子程序調用，可以在 Python 中使用 ABC：

```python
import subprocess

def optimize_blif(input_file, output_file):
    cmd = f"abc -c 'read {input_file}; strash; balance; rewrite; if -K 4; write_blif {output_file}'"
    subprocess.run(cmd, shell=True, check=True)
```

這種方式適合將 ABC 整合到 Python 為主的 EDA 工具鏈中。

### ABC 的平行化

ABC 支援平行化執行：

- 多線程技術映射
- 平行 SAT 求解（多核心 PDR）
- 指令層級的平行化（受限）

對於超大規模設計（百萬節點以上），ABC 可以透過 `-T` 參數設定線程數來充分利用多核心 CPU：

```
abc -c "read large.aig; if -K 4 -T 8; write_blif opt.blif"
```

### ABC 的拓展性

ABC 的架構設計允許使用者加入新的指令和演算法：

1. 實作一個新的 C 函數，遵循 ABC 的指令規範。
2. 將函數註冊到 ABC 的指令表中。
3. 在互動式命令列中使用新的指令。

這種拓展性使 ABC 成為一個理想的邏輯合成研究平台——許多頂尖會議（DAC、ICCAD、DATE）的新演算法都以 ABC 為基礎進行開發和比較。

1. A. Mishchenko et al., "ABC: A System for Sequential Synthesis and Verification," Berkeley Logic Synthesis and Verification Group, 2005–2024.
2. R. Brayton, A. Mishchenko, "ABC: An Academic Industrial-Strength Verification Tool," CAV, 2010.
3. A. Mishchenko, S. Chatterjee, R. Brayton, "DAG-Aware AIG Rewriting: A Fresh Look at Combinational Logic Synthesis," DAC, 2006.
4. A. Mishchenko, S. Cho, S. Chatterjee, R. Brayton, "Combinational and Sequential Mapping with Priority Cuts," ICCAD, 2007.
5. R. Brayton et al., "SAT-Based Logic Synthesis and Technology Mapping," FMCAD, 2006.
6. N. Een, A. Mishchenko, R. Brayton, "Efficient Implementation of Property Directed Reachability," FMCAD, 2011.
7. C. Wolf, "Yosys Open Synthesis Suite," 2012–2024.

## 延伸閱讀

其他相關條目：[BDD](./bdd.md)、[邏輯合成](./synthesis.md)、[形式驗證](./formal_verification.md)、[IEEE 1364 Verilog](./verilog.md)、[FPGA](./fpga.md)
