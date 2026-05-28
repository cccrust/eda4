# Verilog HDL 介紹

## 什麼是 Verilog

Verilog 是一種硬體描述語言 (Hardware Description Language, HDL)，用於建模、設計、模擬和合成數位電路。它由 IEEE 1364 標準所規範，與 VHDL 並列為電子設計自動化 (EDA) 領域兩大主流 HDL。Verilog 的語法類似 C 語言，使其對熟悉程式設計的工程師相對容易上手，但其語意核心與軟體程式語言有根本差異——Verilog 描述的是硬體連線與並行行為，而非指令的順序執行。

Verilog 支援從行為級 (behavioral) 到暫存器傳輸級 (RTL)、再到閘級 (gate-level) 和開關級 (switch-level) 的多層次抽象。設計者可以在同一個模組中混用不同抽象層級，這在大型專案的逐步細化設計流程中至關重要。合成工具 (synthesis tool) 會將可合成的 Verilog 程式碼轉換成對應的邏輯閘網路，最終實現於 ASIC 或 FPGA 中。

在 eda4 專案中，Verilog 處於核心位置：verilog2fpga 子專案使用純 Rust 實現的 Verilog-2005 解析器與合成器，直接將 Verilog 原始碼轉換為 FPGA 位元流；verilog2rust 子專案則將同樣的 Verilog 原始碼轉譯為 Rust HDL 程式碼，提供軟體模擬與硬體驗證的替代路徑。

## 歷史沿革

Verilog 的歷史始於 1984 年，由 Gateway Design Automation 公司開發，最初作為專有硬體描述語言與模擬器使用。當時市場上缺乏成熟的 HDL 標準，Verilog 以其近似 C 語言的語法迅速在工程社群中獲得關注。

1989 年，Cadence Design Systems 收購 Gateway Design Automation，取得 Verilog 的所有權。Cadence 於 1990 年公開 Verilog 語言，並開始推動標準化進程。1995 年，IEEE 正式通過 IEEE 1364-1995 標準，這是 Verilog 的第一個官方標準版本。

2001 年，IEEE 發布 1364-2001 標準（又稱 Verilog-2001），這是語言的一次重大更新，加入了 `generate` 區塊、有符號運算、逗號分隔的埠列表、`always @*` 敏感列表簡寫等關鍵功能，大幅提升了表達力和可用性。2005 年的 IEEE 1364-2005 標準則主要是小幅修訂與勘誤，並未引入重大語言變更。此後，Verilog 的後續發展以 SystemVerilog (IEEE 1800) 的形式繼續演進，SystemVerilog 在 2009 年將 Verilog 相容性整合為標準的一部分。

eda4 專案鎖定 Verilog-2005 子集，聚焦於可合成的 RTL 設計模式，不包含 SystemVerilog 的物件導向特性、`assertion`、`interface` 等進階功能。

## 核心概念

### 模組與埠

模組 (module) 是 Verilog 的基本建構單元，類似 C 語言的函式或電路設計中的黑箱。每個模組由埠 (port) 定義與外界的連線介面，內部則包含電路結構和行為的描述。典型模組定義如下：

```verilog
module counter(
    input  wire       clk,
    input  wire       rst_n,
    output reg  [7:0] count
);
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n)
            count <= 8'd0;
        else
            count <= count + 1'b1;
    end
endmodule
```

埠有三種方向：`input`、`output` 和 `inout`（雙向）。埠的資料類型可以是 `wire` 或 `reg`，其中 `input` 和 `inout` 必須是 `wire` 類型，`output` 可以是 `wire` 或 `reg`。

### 連線與暫存器

`wire` 類型代表連續驅動的連線，類似實際電路中的導線。它的值由連續賦值 (continuous assignment) 或模組埠的連線決定。`reg` 類型則代表儲存單元，用於程序區塊 (procedural block) 內部賦值，但不一定對應到實際的暫存器——在組合邏輯中 `reg` 可能只是變數。

```verilog
wire        a;          // 單 bit 連線
wire [7:0]  bus;        // 8-bit 匯流排
reg         clk;        // 暫存器
reg  [31:0] data;       // 32-bit 暫存器
```

### 連續賦值

`assign` 語句用於驅動 `wire` 類型的訊號，當右側表達式的任何變數發生變化時，左側的 `wire` 會立即更新。這模擬了組合邏輯的行為：

```verilog
assign sum = a ^ b;
assign carry = a & b;
```

### always 區塊

`always` 區塊是 Verilog 中描述時序邏輯和組合邏輯的主要方式。所有 `always` 區塊概念上並行執行——這是 HDL 與軟體程式語言最關鍵的區別之一。`always` 區塊由敏感列表 (sensitivity list) 觸發執行：

```verilog
always @(posedge clk)             // 時序邏輯，clk 上升沿觸發
    q <= d;

always @*                         // 組合邏輯，所有輸入變化觸發
    y = a & b | c;
```

## 並行性與程序邏輯

Verilog 語言最根本的特性是其內建的並行性。在一個模組中，所有的 `always` 區塊、`assign` 語句和模組實例化在模擬時都同時參與運算，不存在傳統軟體程式的 main 函式或順序執行起點。

這種並行模型直接反映了數位硬體的本質：電路中的每個閘和觸發器都在同時工作。EDA 工具（模擬器與合成器）的責任就是正確處理這種並行性，確保結果與實際硬體行為一致。

與並行性相對的是程序區塊內部的順序邏輯。在單一 `always` 區塊內部，程式碼由上而下依序執行，這使得設計者能夠在區域範圍內使用 `if/else`、`case` 等控制結構描述複雜的邏輯。模擬器透過事件驅動 (event-driven) 模型實現並行性：當訊號變化時，觸發依賴該訊號的所有區塊，區塊執行完畢後可能產生新的訊號變化，進而觸發更多區塊，直到系統收斂。

eda4 的 verilog2fpga 合成器在解析 Verilog 後，會將所有並行結構轉換為內部 netlist 表示，再透過技術映射 (techmap) 將邏輯映射到 iCE40 FPGA 的邏輯單元 (logic cell) 中。

## 四值邏輯系統

Verilog 使用四值邏輯系統，這與數位電路的實際行為緊密相關：

- **0** — 邏輯低電位 (false / low)
- **1** — 邏輯高電位 (true / high)
- **X** — 未知狀態 (unknown)，表示無法確定的邏輯值，通常用於模擬初期未初始化的暫存器
- **Z** — 高阻抗 (high-impedance)，表示訊號未被驅動，常見於三態匯流排

所有 Verilog 的表達式運算都遵循預先定義的四值邏輯真值表。例如，`X & 0` 的結果是 `0`（因為與閘只要有一輸入為 0，輸出即為 0），但 `X & 1` 的結果是 `X`。

在合成階段，`X` 和 `Z` 的處理方式與模擬不同：`X` 通常被合成工具視為「不在乎」(don't care) 條件，可用於邏輯最佳化；`Z` 則對應到實際的三態緩衝器或雙向 I/O 引腳。

eda4 的 verilog2rust 在模擬運行時 (runtime) 中實作了這套四值邏輯系統，對應的型別定義於 `verilog2rust/src/rhdl/` 中，包含 `Signal` 結構和相關運算子重載。

## 資料型別

Verilog-2005 提供多種內建資料型別，以下是最常用的幾種：

**wire** — 表示硬體連線，由連續賦值或閘輸出驅動。是 Verilog 中最基本的資料型別。在 eda4 的解析器中，`wire` 宣告被記錄於符號表中，並在 netlist 生成階段轉換為對應的連線節點。

**reg** — 表示程序區塊中的賦值目標，可對應到暫存器或組合邏輯輸出。需要注意 `reg` 並不總是合成為 flip-flop——只有在 `always @(posedge clk)` 區塊中賦值的 `reg` 才會成為觸發器。

**integer** — 有號 32-bit 整數，常用於迴圈計數器和測試平台中的計數變數。`integer` 在合成中通常不被支援，主要用於行為描述和驗證。

**parameter** — 編譯期常數，用於參數化模組設計：

```verilog
parameter WIDTH = 8;
reg [WIDTH-1:0] data;
```

**genvar** — Verilog-2001 引入，用於 `generate` 區塊中的迴圈計數變數，在編譯期由合成工具展開：

```verilog
generate
    genvar i;
    for (i = 0; i < 8; i = i + 1) begin : gen_ff
        dff inst(.clk(clk), .d(in[i]), .q(out[i]));
    end
endgenerate
```

eda4 的 verilog2fpga 嚴格區分可合成與不可合成的資料型別，對於 `time`、`real`、`realtime` 等主要用於模擬的型別不提供支援。

## 運算子

Verilog 的運算子種類豐富，涵蓋數位電路設計所需的各種運算：

**位元運算子**：`&` (AND)、`|` (OR)、`^` (XOR)、`~` (NOT)、`~^` / `^~` (XNOR)。對兩個向量逐位元進行邏輯運算。

**歸約運算子**：`&` (歸約 AND)、`|` (歸約 OR)、`^` (歸約 XOR)、`~&`、`~|`、`~^`。將向量的所有位元進行單一邏輯運算，結果為 1-bit：

```verilog
wire all_zero = ~|data_bus;   // 檢查是否全為 0
wire parity   = ^data_bus;    // 計算奇同位
```

**算術運算子**：`+`、`-`、`*`、`/`、`%`。Verilog-2001 支援 `signed` 關鍵字和 `$signed()` 系統函式，使有號數運算更加可靠。

**關係運算子**：`>`、`<`、`>=`、`<=`、`==`、`!=`。結果為 1-bit 的 Boolean 值（0、1 或 X）。

**移位運算子**：`<<` (左移)、`>>` (右移)、`<<<` (算術左移)、`>>>` (算術右移)。算術右移會保留符號位元，這在有號運算中很重要。

**串接與複製運算子**：`{a, b}` 將多個訊號串接為更寬的向量；`{n{expr}}` 將表達式複製 n 次：

```verilog
wire [7:0] byte = {msb, nibble, 3'b0};
wire [7:0] mask = {4{2'b10}};      // 結果: 10101010
```

**條件運算子**：`cond ? expr_true : expr_false`。對應到多工器 (multiplexer)：

```verilog
assign max = (a > b) ? a : b;
```

eda4 的 Pratt 表達式解析器處理所有上述運算子的優先級與結合性，並在內部表示為抽象語法樹 (AST) 節點，供後續合成或程式碼生成使用。

## 程序區塊與敏感列表

### 時序邏輯

時序邏輯使用 `always @(posedge clk)` 或 `always @(negedge clk)` 定義，必要時加入非同步重置訊號：

```verilog
always @(posedge clk or negedge rst_n) begin
    if (!rst_n)
        q <= 0;
    else
        q <= d;
end
```

敏感列表中的 `posedge` 和 `negedge` 指定觸發邊沿，合成工具會根據此列表推導出觸發器類型（D flip-flop）與時鐘使能邏輯。

### 組合邏輯

Verilog-2001 引入的 `always @*`（或 `always @(*)`）自動推導敏感列表，包含區塊內讀取的所有訊號：

```verilog
always @* begin
    case (sel)
        2'b00: y = a;
        2'b01: y = b;
        2'b10: y = c;
        default: y = d;
    endcase
end
```

使用 `always @*` 是良好的設計習慣，可避免因手動遺漏敏感列表訊號而導致的模擬與合成不一致 (simulation-synthesis mismatch)。

### 敏感列表的邊沿觸發邊際情況

`always` 區塊支援的敏感列表格式包括：
- `always @(event1 or event2 or ...)` — 用 `or` 分隔多個事件
- `always @(event1, event2, ...)` — Verilog-2001 允許用逗號分隔
- `always @*` / `always @(*)` — 自動推導組合邏輯敏感列表

每個事件可以是 `posedge signal`、`negedge signal`，或單純的 `signal`（電位敏感，用於電平敏感鎖存器 — 應盡量避免）。

## 阻塞與非阻塞賦值

Verilog 程序區塊中的賦值分為兩類，理解其差異對於正確設計至關重要：

**阻塞賦值 (=)**：使用 `=` 運算子，賦值立即生效。在同一 `always` 區塊中，後面的語句會看到前面語句的最新賦值結果。阻塞賦值主要用於組合邏輯建模：

```verilog
always @* begin
    tmp = a & b;
    y   = tmp | c;
end
```

**非阻塞賦值 (<=)**：使用 `<=` 運算子，賦值在時間步 (time step) 結束時同時更新。所有非阻塞賦值在區塊執行期間讀取舊值，在區塊結束後才寫入新值。非阻塞賦值用於時序邏輯建模：

```verilog
always @(posedge clk) begin
    q1 <= d;
    q2 <= q1;      // q2 得到的是 q1 的舊值，模擬 D flip-flop 的行為
end
```

如果將非阻塞賦值用於組合邏輯，或將阻塞賦值用於時序邏輯，都可能導致模擬結果與實際硬體行為不一致。這在業界被稱為「阻塞/非阻塞賦值誤用」的常見設計錯誤。eda4 的 verilog2rust 會在遷移過程中保持這兩種賦值語意的正確性，確保 Rust 模擬與原始 Verilog 行為一致。

## 控制流程

Verilog 的程序區塊支援多種控制結構，與 C 語言相似但帶有硬體設計的特殊語意：

**if/else** — 條件分支，對應到多工器或優先編碼器：

```verilog
always @* begin
    if (sel == 0)
        y = a;
    else if (sel == 1)
        y = b;
    else
        y = c;
end
```

**case/endcase** — 多路分支，對應到多工器或解碼器：

```verilog
always @* begin
    case (state)
        IDLE:     next = READ;
        READ:     next = COMPUTE;
        COMPUTE:  next = WRITE;
        WRITE:    next = IDLE;
        default:  next = IDLE;
    endcase
end
```

**for 迴圈** — 主要用於 `generate` 區塊或 `always` 區塊中的重複邏輯。合成工具通常要求 for 迴圈的邊界是編譯期常數：

```verilog
always @* begin
    for (i = 0; i < 8; i = i + 1)
        parity[i] = ^data[i*8 +: 8];
end
```

**forever 迴圈** — 無窮迴圈，主要用於測試平台中產生時鐘訊號：

```verilog
initial begin
    clk = 0;
    forever #5 clk = ~clk;
end
```

**begin/end** — 將多條語句組合成順序區塊，相當於 C 語言的大括號。

在 eda4 的 Verilog 解析器中，所有控制結構都被轉換為 AST 節點，並在語意分析階段進行類型檢查和可合成性驗證。

## 模組實例化

Verilog 模組可以像晶片一樣被其他模組引用和連接，這個過程稱為實例化 (instantiation)。模組實例化支援兩種埠連接方式：

**位置連接 (positional)** — 依埠宣告順序連接，簡潔但容易出錯：

```verilog
adder u1(a, b, sum, cout);   // 假設 adder 的埠順序為 a, b, sum, cout
```

**命名連接 (named)** — 明確指定埠名稱，更安全且可讀性更高：

```verilog
adder #(.WIDTH(8)) u1 (
    .a     (a),
    .b     (b),
    .sum   (sum),
    .cout  (cout)
);
```

參數化模組透過 `#(...)` 語法傳遞參數，使模組可重用於不同的位寬或配置。eda4 的 verilog2fpga 在 netlist 生成階段會展開所有模組實例化，建立完整的階層式連接圖。

## 內建閘級原語

Verilog 語言內建了一組閘級原語 (gate-level primitives)，用於在結構化描述中直接實體化邏輯閘：

- **and** — 多輸入與閘：`and (out, in1, in2, ...)`
- **or** — 多輸入或閘：`or (out, in1, in2, ...)`
- **not** — 反相器：`not (out, in)`
- **xor** — 多輸入互斥或閘：`xor (out, in1, in2, ...)`
- **nand** — 多輸入反及閘
- **nor** — 多輸入反或閘
- **xnor** — 多輸入互斥反或閘
- **buf** — 緩衝器：`buf (out, in)`

閘級原語的埠連接順序固定：第一個埠為輸出，其餘為輸入。這些原語提供了最底層的抽象，常用於技術映射 (technology mapping) 階段的輸出或手動最佳化的關鍵路徑。

```verilog
and g1(sum, a, b);
and g2(carry, a, b);
```

eda4 的 verilog2fpga 合成器最終會將高層次的 RTL 描述映射到 iCE40 的邏輯單元，但也可以接受閘級網表作為輸入。

## 編譯器指令

Verilog 的編譯器指令 (compiler directives) 以反引號 `` ` `` 開頭，在編譯階段由預處理器處理：

**`include** — 檔案包含，類似 C 語言的 `#include`：

```verilog
`include "defines.v"
`include "adder.v"
```

**`define** — 文字巨集定義，可用於常數定義和條件編譯：

```verilog
`define CLK_PERIOD 10
`define SIMULATION
```

**`ifdef / `ifndef / `else / `endif** — 條件編譯，用於在測試平台中區分模擬與合成配置：

```verilog
`ifdef SIMULATION
    initial $monitor("clk=%b state=%b", clk, state);
`endif
```

eda4 的 Verilog 預處理器實作支援巢狀的 `` `ifdef`` 和 `` `define `` 巨集展開，與標準 Verilog 工具的行為保持一致。

## 系統任務與函式

Verilog 提供以 `$` 開頭的系統任務和函式，主要用於測試平台和除錯：

**$display** — 在模擬過程中列印格式化訊息，類似 C 語言的 `printf`：

```verilog
$display("time=%0t a=%b b=%b sum=%b", $time, a, b, sum);
```

**$monitor** — 持續監控訊號變化，每當訊號改變時自動列印：

```verilog
initial $monitor("At %t: clk=%d, count=%d", $time, clk, count);
```

**$finish** — 結束模擬：

```verilog
initial #100 $finish;
```

**$readmemh / $readmemb** — 從檔案載入記憶體初始化數據，十六進位和二進位格式：

```verilog
reg [7:0] memory [0:255];
initial $readmemh("program.hex", memory);
```

**$time / $realtime** — 返回當前模擬時間。

eda4 的 verilog2rust 對系統任務的支援聚焦於可合成的功能上；純模擬性質的 `$display` 和 `$monitor` 在轉譯時被映射到 Rust 的 `println!` 和對應的追蹤機制。

## 測試平台

測試平台 (testbench) 是用於驗證 Verilog 模組功能的頂層包裝，它被設計成不可合成的，專注於模擬驗證。典型的測試平台結構如下：

```verilog
module tb_counter;
    reg        clk;
    reg        rst_n;
    wire [7:0] count;

    counter uut (.clk(clk), .rst_n(rst_n), .count(count));

    initial begin
        clk = 0;
        forever #5 clk = ~clk;
    end

    initial begin
        rst_n = 0;
        #20 rst_n = 1;
        #200 $finish;
    end

    initial begin
        $monitor("time=%0t count=%d", $time, count);
        #10 rst_n = 0;
        #10 rst_n = 1;
    end
endmodule
```

`initial` 區塊在模擬開始時執行一次，與 `always` 區塊一樣並行存在。`#` 延遲控制是測試平台中控制時間進展的主要手段——`#10` 表示等待 10 個時間單位。

eda4 的 verilog2rust 也支援將測試平台轉譯為 Rust 測試程式，利用 Rust 的 `#[test]` 屬性來進行自動化的硬體驗證。

## Verilog 與 VHDL 比較

Verilog 和 VHDL 是 HDL 領域的兩大主流語言，各有其設計哲學與適用場景：

| 面向 | Verilog | VHDL |
|------|---------|------|
| 語法基礎 | 類似 C 語言 | 類似 Ada（源自美國國防部需求） |
| 標準化 | IEEE 1364 | IEEE 1076 |
| 表達力 | 簡潔，程式碼量較少 | 嚴謹，型別系統更嚴格 |
| 學習曲線 | 相對平緩 | 相對陡峭 |
| 低位元抽象 | 支援開關級與閘級 | 主要為 RTL 及以上 |
| 型別系統 | 鬆散，隱式轉換 | 嚴謹，需顯式轉換 |
| 主要應用 | 美國/亞洲 IC 設計主流 | 歐洲軍工/航太領域盛行 |

兩者都能描述相同的硬體行為，選擇主要取決於團隊傳統、生態系統和專案需求。eda4 選擇 Verilog 作為主要輸入語言，主要考量是 Verilog 在 FPGA 開發中的廣泛使用以及其語法對 Rust 程式碼生成的便利性。但要強調的是，eda4 並不直接支援 VHDL 輸入。

## eda4 支援的 Verilog-2005 子集

eda4 專案實現了一個可合成的 Verilog-2005 子集，涵蓋日常 FPGA 開發中最常用的語言功能：

**已支援**：
- 模組宣告與埠定義（`input`、`output`、`inout`）
- `wire` 與 `reg` 類型（含向量）
- `parameter` 與區域參數 (`localparam`)
- `assign` 連續賦值
- `always @(posedge/negedge)` 時序邏輯
- `always @*` 組合邏輯
- `if/else`、`case/endcase` 控制結構
- `for` 迴圈（合成時展開）
- `generate` 區塊（`for` 與 `if/else` 形式）
- 算術、位元、邏輯、移位、串接、條件運算子
- 模組實例化（命名與位置連接）
- 多維陣列
- 有號數 (`signed`) 支援
- `function` 與 `task`

**未支援**：
- 使用者定義原語 (UDP, User-Defined Primitives)
- `specify` 區塊與 `$setup`/`$hold` 時序檢查
- 開關級原語 (`tran`、`tranif0`、`rtran` 等)
- `force`/`release` 程序性賦值
- `wait` 語句
- `event` 命名事件
- `initial` 區塊中的非固定延遲
- SystemVerilog 擴展 (`always_ff`、`always_comb`、`interface`、`class` 等)

這些限制確保 eda4 的語法分析器和後端工具能夠高效且可靠地處理實際的 FPGA 設計。

## eda4 中 verilog2fpga 的解析器實作

verilog2fpga 子專案的 Verilog 解析器 (`v2f-synth` crate) 採用遞迴下降 (recursive descent) 解析策略，結合 Pratt 解析器處理表達式的優先級和結合性。

解析流程分為三個階段：

1. **詞法分析 (Lexer)**：將原始 Verilog 文字轉換為記號串流 (token stream)，處理識別字、關鍵字、數值字面量、運算子、註解和空白。詞法分析器同時負責處理編譯器指令（`` `include``、`` `define`` 等）的展開。

2. **語法分析 (Parser)**：遞迴下降解析器根據 Verilog 的上下文無關文法 (CFG) 建構抽象語法樹 (AST)。模組宣告、埠列表、訊號宣告、`always` 區塊、`assign` 語句、控制結構等都有對應的解析函式。表達式層級使用 Pratt 解析技術，為每個運算子分配優先級和解析函式，以簡化表達式層級的處理。

3. **語意分析與 Netlist 生成**：遍歷 AST 建立符號表、解析型別和位寬、進行名稱解析和連線檢查，最終將 Verilog 設計轉換為與具體 FPGA 架構無關的 netlist 表示。

這個純 Rust 實作的解析器不需要外部工具依賴，與專案的「純 Rust EDA 工具鏈」理念一致。

## eda4 中 verilog2rust 的程式碼生成

verilog2rust 子專案利用與 verilog2fpga 相同的詞法分析和語法分析基礎，但在語意分析階段走向不同的路徑——它不是產生 netlist，而是生成 Rust 原始碼。

轉譯的核心策略是將 Verilog 的硬體並行模型映射到 Rust 的軟體執行模型：

- **模組** → Rust 結構體 (`struct`)，包含所有內部訊號作為欄位
- **並行區塊** → 模擬執行時的求值迴圈，按拓撲順序遍歷所有 `always` 區塊和 `assign` 語句
- **非阻塞賦值** → 使用雙緩衝機制（舊值/新值切換）模擬時間步更新
- **四值邏輯** → `Signal` 類型列舉，支援 `0`、`1`、`X`、`Z` 四種狀態及對應的邏輯運算
- **時序邏輯** → 邊沿檢測函式，追蹤訊號的上一個值和當前值以偵測 `posedge`/`negedge`
- **測試平台** → Rust 的 `#[test]` 函式，配合 `#` 延遲的模擬實現

verilog2rust 的執行流程如下：

1. 解析 Verilog 原始碼產生 AST
2. 符號解析與型別檢查
3. 生成 Rust 模組（包含 `Signal` 類型的結構體和求值函式）
4. 編譯 Rust 程式碼並執行（直接產生二進位或透過 `rustc` 即時編譯）
5. 輸出模擬結果或與測試向量比對

這種方法允許設計者在純 Rust 環境中模擬和驗證 Verilog 設計，利用 Rust 的所有權系統和型別安全特性來避免傳統 Verilog 模擬中常見的初始化順序和競爭條件問題。

## 結語

Verilog 作為 IEEE 標準的硬體描述語言，已經在數位電路設計領域累積超過三十年的使用歷史。從 ASIC 到 FPGA、從簡單的狀態機到複雜的系統單晶片，Verilog 始終是硬體工程師描述和驗證設計的核心工具。eda4 專案透過 verilog2fpga 和 verilog2rust 兩個子專案，從 Rust 生態系統的角度重新詮釋了 Verilog——前者實現了純 Rust 的 FPGA 合成工具鏈，後者則搭建了 Verilog 與 Rust 軟體世界之間的橋樑。這樣的設計不僅為硬體設計提供了新的工具選擇，也為 Rust 在 EDA 領域的應用開創了新的可能性。

## 延伸閱讀

- [Verilog (Wikipedia)](https://en.wikipedia.org/wiki/Verilog)
- [VHDL (Wikipedia)](https://en.wikipedia.org/wiki/VHDL)
- [Hardware Description Language (Wikipedia)](https://en.wikipedia.org/wiki/Hardware_description_language)
- [IEEE 1364 (Wikipedia)](https://en.wikipedia.org/wiki/IEEE_1364)
- [Digital Signal Processing (Wikipedia)](https://en.wikipedia.org/wiki/Digital_signal_processing)
