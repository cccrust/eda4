# RTL (Register Transfer Level)

## 什麼是 RTL

RTL（Register Transfer Level，暫存器傳輸層級）是數位電路設計中的一種抽象層級，將電路模型描述為資料在暫存器之間流動，並經由組合邏輯進行運算。在這個抽象層級中，設計人員專注於**每個時脈週期內**暫存器之間的資料轉換與傳輸，而非底層的邏輯閘或電晶體實現。

RTL 的核心概念是：

- **暫存器（Register）**：由正反器（Flip-Flop）構成，在時脈邊緣儲存狀態。
- **組合邏輯（Combinational Logic）**：在暫存器之間進行布林運算、算術運算、多工選擇等。
- **時脈邊界（Clock Boundary）**：每個暫存器更新都與時脈邊緣同步，形成明確的週期邊界。

## 抽象層級階層

數位電路設計可按 Gajski-Kuhn Y-Chart（伽斯基-庫恩 Y 圖）劃分為多個抽象層級：

| 層級 | 說明 | 範例 |
|------|------|------|
| 電晶體層級（Transistor） | 開關級電路，NMOS/PMOS 網路 | SPICE 網表 |
| 邏輯閘層級（Gate） | 基本邏輯閘（AND、OR、NOT、NAND） | 邏輯網表 |
| **RTL（Register Transfer）** | **暫存器 + 組合邏輯，時脈週期精確** | **Verilog/VHDL 合成子集** |
| 行為層級（Behavioral） | 純演算法描述，無週期精確性 | 未合成的 Verilog |
| 演算法層級（Algorithmic） | 系統層級功能規格 | SystemC、C++ 模型 |

RTL 恰好位在抽象層級的中間：它比邏輯閘層級更高階（因此設計效率更高），又比行為層級更具體（因此可以預測地合成為邏輯閘）。

Y-Chart 的三個維度分別是：

1. **行為域（Behavioral Domain）**：描述電路做什麼。
2. **結構域（Structural Domain）**：描述電路由什麼元件組成。
3. **物理域（Physical Domain）**：描述電路的幾何佈局。

RTL 在行為域中對應「暫存器傳輸行為」，在結構域中對應「暫存器 + 組合邏輯方塊」。

## RTL 的組成元件

### 暫存器（Register）

暫存器由 D 型正反器（D Flip-Flop）實現，在時脈正緣（posedge）或負緣（negedge）將輸入取樣並儲存：

```verilog
always @(posedge clk) begin
    q <= d;
end
```

暫存器的關鍵參數包括：

- **建立時間（Setup Time, tsu）**：資料在時脈邊緣前需穩定的最短時間。
- **保持時間（Hold Time, th）**：資料在時脈邊緣後需穩定的最短時間。
- **時脈到輸出延遲（Clock-to-Q Delay, tcq）**：時脈邊緣到輸出變化的延遲。
- **非同步重置（Asynchronous Reset）**：不受時脈控制的強制重置。

### 組合邏輯（Combinational Logic）

組合邏輯的輸出僅取決於當前輸入，不具記憶性。RTL 中常見的組合邏輯元件包括：

- 基本邏輯閘：AND、OR、XOR、NOT、NAND、NOR
- 多工器（Multiplexer, MUX）
- 解碼器（Decoder）、編碼器（Encoder）
- 算術邏輯單元（ALU）：加法器、減法器、比較器
- 移位器（Shifter）

### 資料路徑（Datapath）

資料路徑負責實際的資料運算與傳輸，通常由以下元素構成：

- 暫存器檔案（Register File）
- 算術單元（加法器、乘法器）
- 匯流排（Bus）與多工器網路
- 管線暫存器（Pipeline Register）

### 控制單元（Control Unit / FSM）

控制單元是一個有限狀態機（FSM），根據當前狀態與輸入條件決定下一個狀態，並產生控制信號驅動資料路徑。

## RTL 設計風格：Verilog 合成子集

RTL 設計對應硬體描述語言（HDL）中**可合成的語法子集**。並非所有 Verilog/VHDL 語法都可以對應到實際硬體；例如 `$display`、`$monitor`、`initial` 區塊（除了 testbench）、`force`、`wait` 等僅用於模擬，無法合成。

Verilog 的 RTL 合成子集主要包含：

| 語法結構 | 用途 | 是否可合成 |
|----------|------|-----------|
| `assign` | 組合邏輯連續賦值 | 是 |
| `always @(posedge clk)` | 序向邏輯（暫存器） | 是 |
| `always @*` | 組合邏輯程序塊 | 是 |
| `if` / `else` | 條件賦值 | 是（須完整） |
| `case` / `casez` | 多路分支 | 是（須完整） |
| `for` 迴圈（常數範圍） | 重複結構生成 | 是 |
| `generate` / `genvar` | 參數化結構生成 | 是 |
| `function` | 純組合邏輯函式 | 是 |
| `$display`, `$monitor` | 除錯輸出 | 否 |
| `initial` | 初始化（testbench） | 否（僅部分支援） |
| `force` / `release` | 強制賦值 | 否 |
| `wait` | 事件等待 | 否 |

## 同步設計

RTL 設計的基礎假設是**同步設計（Synchronous Design）**——所有暫存器都由同一個時脈信號的同一邊緣觸發。這帶來了關鍵優勢：

1. **可預測的時序**：所有路徑都是從一個暫存器到另一個暫存器，時序分析只需要計算「組合邏輯延遲 + 走線延遲 < 時脈週期」。
2. **靜態時序分析（STA）可行**：EDA 工具可以自動檢查是否滿足建立時間與保持時間。
3. **避免競爭危害**：非同步電路中的競賽條件（Race Condition）在同步設計中大幅減少。

同步設計的限制條件：

```
Tclk >= Tc-q + Tcomb + Tsu
```

其中 `Tclk` 為時脈週期，`Tc-q` 為暫存器時脈到輸出延遲，`Tcomb` 為組合邏輯最大延遲，`Tsu` 為暫存器建立時間。

## always 區塊的使用規則

### 序向邏輯（Sequential Logic）

```verilog
always @(posedge clk or negedge rst_n) begin
    if (!rst_n)
        q <= 0;
    else
        q <= d;
end
```

- 敏感列表包含 `posedge clk`（時脈正緣）和可選的非同步重置信號。
- 使用**非阻塞賦值（<=）**。

### 組合邏輯（Combinational Logic）

```verilog
always @(*) begin
    case (sel)
        2'b00: y = a;
        2'b01: y = b;
        2'b10: y = c;
        default: y = d;
    endcase
end
```

- 敏感列表使用 `@(*)`（自動推斷所有輸入）。
- 使用**阻塞賦值（=）**。
- 必須覆蓋所有分支，否則會產生鎖存器（Latch）。

```verilog
// 連續賦值同樣用於組合邏輯
assign y = sel ? a : b;
```

## 阻塞賦值與非阻塞賦值的 RTL 編碼規則

這是 Verilog RTL 設計最重要的規則：

| 用途 | 賦值方式 | 說明 |
|------|---------|------|
| 序向邏輯（always @(posedge clk)） | `<=`（非阻塞） | 所有賦值同時更新，模擬正反器行為 |
| 組合邏輯（always @* 或 assign） | `=`（阻塞） | 順序執行，模擬邏輯閘傳遞延遲 |
| 混用（絕對要避免） | — | 同一個 always 區塊內不可混用 |

非阻塞賦值的模擬語意：

```verilog
always @(posedge clk) begin
    a <= b;  // 讀取 b 的舊值，排程在事件佇列中
    b <= a;  // 讀取 a 的舊值，與上行同時更新
end
```

這正確地模擬了正反器的行為：所有暫存器在同一個時脈邊緣同時從輸入取樣，同時更新輸出。如果使用阻塞賦值，則上述兩個賦值會順序執行，模擬結果與實際硬體不符。

## 為什麼 RTL 是業界標準

RTL 之所以成為數位設計的主流抽象層級，原因如下：

1. **合成工具可預測映射**：現代邏輯合成工具（如 Yosys、Synopsys DC、Cadence Genus）可以將 RTL 描述可靠地映射為邏輯閘網表，結果接近最優。

2. **靜態時序分析自然適用**：每條路徑都以暫存器為端點，STA 可以直接計算路徑延遲，無需複雜的非同步路徑分析。

3. **設計生產力與可控性平衡**：RTL 比邏輯閘層級高出約 5-10 倍生產力，同時保留了對硬體行為的精確控制——設計人員清楚知道每個時脈週期發生了什麼。

4. **標準化與工具生態系**：Verilog（IEEE 1364）和 VHDL（IEEE 1076）都是國際標準，擁有完整的 EDA 工具鏈支援。

5. **IP 可攜性**：RTL 級別的 IP（Intellectual Property）可以跨製程、跨工具、跨廠商重複使用。

## 有限狀態機（FSM）的 RTL 實現

### Moore 狀態機

輸出僅取決於當前狀態（與輸入無關）：

```verilog
always @(posedge clk or negedge rst_n) begin
    if (!rst_n)
        state <= IDLE;
    else
        state <= next_state;
end

always @(*) begin
    case (state)
        IDLE:  if (start) next_state = RUN;
        RUN:   if (done)  next_state = IDLE;
        default: next_state = IDLE;
    endcase
end

always @(*) begin
    case (state)
        IDLE:  out = 0;
        RUN:   out = 1;
        default: out = 0;
    endcase
end
```

### Mealy 狀態機

輸出同時取決於當前狀態與輸入：

```verilog
always @(*) begin
    case (state)
        IDLE: if (start) begin next_state = RUN; out = 1; end
        RUN:  if (done)  begin next_state = IDLE; out = 0; end
        default: begin next_state = IDLE; out = 0; end
    endcase
end
```

### 狀態編碼方式

| 編碼方式 | 說明 | 暫存器數 | 組合邏輯 | 適用場景 |
|---------|------|---------|---------|---------|
| 二進制（Binary） | 狀態以二進制編號 | log2(N) | 較多 | 狀態數多（>10） |
| 獨熱碼（One-Hot） | 每個狀態一個正反器 | N | 較少（解碼簡單） | FPGA 最佳化 |
| 格雷碼（Gray） | 相鄰狀態僅一位元不同 | log2(N) | 中等 | 降低功耗 |

FPGA 架構因為每個 Logic Cell 都有正反器，獨熱碼通常是最有效率的選擇——雖然使用更多暫存器，但大大簡化了組合邏輯。

## 資料路徑 RTL

資料路徑的 RTL 描述直接使用算術與邏輯運算子：

```verilog
// 加法器
assign sum = a + b;

// 乘法器
assign product = a * b;

// 比較器
assign a_lt_b = (a < b);
assign a_eq_b = (a == b);

// 移位器
assign shifted = data_in << shamt;

// 多工器
assign result = sel ? data_a : data_b;

// ALU
always @(*) begin
    case (alu_op)
        3'b000: alu_out = a + b;
        3'b001: alu_out = a - b;
        3'b010: alu_out = a & b;
        3'b011: alu_out = a | b;
        3'b100: alu_out = a ^ b;
        3'b101: alu_out = a << b[3:1];
        default: alu_out = 0;
    endcase
end
```

合成工具會自動從 `+`、`*`、`<` 等運算子推斷出對應的硬體結構（如進位鏈加法器、華勒斯樹乘法器）。

## 控制 RTL

控制 RTL 本質上就是將控制邏輯實作為 FSM，描述設計的控制流程。在一個完整的處理器中：

- **資料路徑**負責執行算術運算、資料搬移。
- **控制單元**負責解碼指令、產生控制信號（register write enable、ALU opcode、MUX select、memory read/write）。

兩者的互動方式為：控制 FSM 根據當前指令與狀態發出控制信號，資料路徑根據這些信號進行運算，並回傳狀態信號（如 zero flag、carry flag）給控制 FSM。

典型的控制 FSM 範例（CPU 指令擷取階段）：

```verilog
typedef enum logic [2:0] { FETCH, DECODE, EXECUTE, MEM_ACCESS, WRITEBACK } state_t;

always @(posedge clk or negedge rst_n) begin
    if (!rst_n)
        state <= FETCH;
    else
        state <= next_state;
end

always @(*) begin
    next_state = state;
    pc_write     = 0;
    ir_write     = 0;
    reg_write    = 0;
    mem_read     = 0;
    mem_write    = 0;
    alu_sel      = 0;

    case (state)
        FETCH: begin
            mem_read = 1;
            ir_write = 1;
            next_state = DECODE;
        end
        DECODE: begin
            next_state = EXECUTE;
        end
        EXECUTE: begin
            // 根據解碼後的指令執行
            next_state = MEM_ACCESS;
        end
        // ...
    endcase
end
```

## RTL 合成流程

RTL 合成（Logic Synthesis）是將 RTL 描述轉換為邏輯閘網表的過程，典型流程如下：

```
RTL Verilog/VHDL
    |
    v
Elaboration（解析、展開、實體化）
    |
    v
HDL 最佳化（常數傳播、死邏輯消除）
    |
    v
邏輯最佳化（Logic Optimization）
    - 布林代數簡化（Boolean Minimization）
    - 技術無關最佳化（Technology-Independent Optimization）
    |
    v
技術映射（Technology Mapping）
    - 映射到目標製程的標準單元庫
    - 考慮面積、延遲、功耗
    |
    v
閘層級網表（Gate-Level Netlist）
    |
    v
實體設計（Physical Design）：佈局與繞線
```

在 eda4 的 verilog2fpga 中，`v2f-synth` crate 實作了純 Rust 的合成管線，將可合成的 Verilog RTL 解析後，經過技術映射轉換為 iCE40 邏輯單元（Logic Cell）的網表。

## RTL 模擬

RTL 模擬（又稱功能模擬或前模擬）是在合成之前驗證 RTL 程式碼功能的正確性。模擬器直接執行 RTL 描述，無需經過合成工具。

RTL 模擬的優點：

- **速度快**：比閘層級模擬快 10-100 倍。
- **除錯容易**：可以直接觀察暫存器值、信號波形。
- **早期驗證**：在投入合成與佈局繞線之前發現功能錯誤。

標準的模擬流程：

```verilog
// testbench 範例
module tb_adder;
    reg  [7:0] a, b;
    wire [8:0] sum;

    adder uut (.a(a), .b(b), .sum(sum));

    initial begin
        $monitor("a=%d b=%d sum=%d", a, b, sum);
        a = 0; b = 0;
        #10 a = 5;  b = 10;
        #10 a = 255; b = 1;
        #10 $finish;
    end
endmodule
```

在 eda4 的 verilog2rust 中，RTL 模擬採用了一種獨特的方式：將 Verilog 轉換為 Rust 程式碼，然後用 rustc 編譯執行，利用 Rust 的型別系統和效能優勢進行模擬。

## RTL 與行為層級的區別

| 面向 | RTL | 行為層級（Behavioral） |
|------|-----|----------------------|
| 時脈週期精確性 | 是，每個 always @(posedge clk) 定義一個週期 | 否，描述演算法流程 |
| 合成能力 | 可合成 | 通常不可合成（或合成結果不可預測） |
| 範例 | `always @(posedge clk) count <= count + 1;` | `for (i=0; i<N; i++) sum += a[i];` |
| 設計人員控制 | 精確控制硬體結構 | 交由工具決定硬體結構 |
| 時序分析 | STA 可直接進行 | 需要高階綜合（HLS）工具 |

`c = a + b` 在行為描述中只是一個算術運算，不指定何時發生；在 RTL 描述中，它代表一個特定的組合邏輯路徑，其結果將在某個時脈邊緣被寫入暫存器。

## RTL 在 eda4 中的實作

### verilog2fpga（v2f-synth）

`v2f-synth` crate 處理可合成的 Verilog RTL 子集。其管線從 Verilog 原始碼開始：

1. **詞法分析與解析**：將 Verilog 原始碼轉換為 AST。
2. **Elaboration**：展開模組實體化、解析參數、計算訊號寬度。
3. **合成**：將 AST 中的 `always` 區塊、`assign` 語句轉換為邏輯表達式。
4. **技術映射**：將邏輯表達式映射到 iCE40 的邏輯單元（LUT + 正反器）。
5. **輸出 JSON 網表**：提供給後續的 PNR 與位元流打包。

此管線專注於 FPGA 合成，因此只支援完整的合成子集，不支援 `$display` 等模擬專用結構。

### verilog2rust

`verilog2rust` 的目標是模擬（Simulation），因此它處理的 Verilog 子集更廣泛：

- 支援 `$display`、`$monitor`、`$finish` 等模擬系統任務。
- 支援 `initial` 區塊用於初始化與測試序列。
- 支援 `#delay` 延遲控制。
- 將 Verilog 轉換為 Rust 程式碼，使用 `rhdl` 執行階段庫模擬正反器與連線。

兩個專案共用同一個 Verilog 解析器核心，但後端的處理邏輯不同。

## RTL 設計的重要規則

### 避免鎖存器（Latch）

鎖存器（Level-Sensitive Latch）通常是 RTL 設計中非預期的產物。當組合邏輯區塊（`always @*`）沒有覆蓋所有輸入條件時，合成工具會推斷出鎖存器：

```verilog
// 不好的寫法：會產生鎖存器
always @(*) begin
    if (en)
        q = d;
    // 缺少 else 分支，en=0 時 q 要保持原值 => 鎖存器
end

// 正確寫法：所有條件都有明確賦值
always @(*) begin
    if (en)
        q = d;
    else
        q = 0;  // 或 q = q（但通常用預設值）
end
```

```verilog
// case 不完整也會產生鎖存器
always @(*) begin
    case (sel)
        2'b00: y = a;
        2'b01: y = b;
        // 缺少 2'b10, 2'b11
    endcase
end

// 正確：加入 default
always @(*) begin
    case (sel)
        2'b00: y = a;
        2'b01: y = b;
        default: y = 0;
    endcase
end
```

在 FPGA 中，預期的儲存元件是正反器（Flip-Flop），鎖存器不僅浪費資源，而且時序分析更複雜。鎖存器本身並非不好，但在 RTL 設計中，**除非明確需要 Latch，否則應避免非預期的 Latch 推斷**。

### 避免組合迴圈（Combinational Loop）

組合迴圈是指組合邏輯的輸出直接或間接回饋到輸入，中間沒有暫存器：

```verilog
// 危險：組合迴圈
assign y = a & y;  // y 依賴自己
```

組合迴圈會導致：

- **模擬無法收斂**：模擬器無法確定穩定狀態。
- **合成結果不可預測**：工具可能推斷出鎖存器或振盪器。
- **實體電路可能振盪**：如果路徑延遲剛好滿足，電路會產生非預期的振盪。

除非常特定的應用（如環形振盪器），否則應嚴格避免組合迴圈。

### 暫存化所有輸出

模組的輸出信號最好都經過暫存器輸出（Registered Output）。這樣做的好處：

- **降低輸出延遲**：時序收斂更容易。
- **隔離內部組合邏輯**：避免組合邏輯的毛刺（Glitch）傳播到其他模組。
- **簡化時序約束**：所有輸出路徑都以暫存器為終點。

### 同步化非同步輸入

外部輸入（按鍵、開關、其他晶片的中斷信號）與內部時脈不同步，直接使用會導致**亞穩態（Metastability）**問題。標準做法是用兩個（或更多）正反器構成同步器：

```verilog
always @(posedge clk) begin
    sync_ff1 <= async_in;
    sync_ff2 <= sync_ff1;
end

assign sync_out = sync_ff2;
```

第一級正反器可能進入亞穩態，但在第二級正反器取樣之前有一個完整時脈週期讓電壓穩定，大幅降低亞穩態傳播的機率。

## 亞穩態（Metastability）與跨時脈域

### 問題本質

當資料信號的變化時間違反了正反器的建立/保持時間窗口時，正反器的輸出可能進入亞穩態——既不是邏輯 0 也不是邏輯 1，而是一個介於兩者之間的不穩定電壓。亞穩態的持續時間理論上是無限的（雖然實際上因熱雜訊會最終收斂到某個邏輯值）。

### 跨時脈域（Clock Domain Crossing, CDC）

在有多個時脈的設計中，跨時脈域的資料傳輸是 RTL 設計的重大挑戰：

- **單一位元信號**：使用雙級（或多級）同步器。
- **多位元匯流排**：使用非同步 FIFO（Asynchronous FIFO）或握手機制（Handshake）。

```verilog
// 雙級同步器：標準作法
module synchronizer #(
    parameter STAGES = 2
) (
    input  wire clk,
    input  wire async_in,
    output wire sync_out
);
    reg [STAGES-1:0] sync;

    always @(posedge clk) begin
        sync <= {sync[STAGES-2:0], async_in};
    end

    assign sync_out = sync[STAGES-1];
endmodule
```

### MTBF（Mean Time Between Failures）

亞穩態的可靠性用 MTBF 衡量。MTBF 與時脈頻率、資料變化率、製程參數呈指數關係。同步器的級數越多，MTBF 越長。對於現代 FPGA 設計，雙級同步器的 MTBF 通常在數年到數百萬年之間，足以滿足絕大多數應用。

## RTL 設計流程總結

```
需求規格
    |
    v
架構設計（Architecture Design）
    - 劃分為控制單元 + 資料路徑
    - 定義介面與協定
    |
    v
RTL 編碼（RTL Coding）
    - Verilog/VHDL 撰寫
    - 遵守合成子集規範
    |
    v
功能模擬（Functional Simulation）
    - 撰寫 testbench
    - 驗證功能正確性
    |
    v
邏輯合成（Logic Synthesis）
    - RTL → 邏輯閘網表
    - 面積/時序最佳化
    |
    v
靜態時序分析（STA）
    |  + 佈局繞線（Place & Route）
    v
閘層級模擬（Gate-Level Simulation）-- 可選
    |
    v
晶片製造 / FPGA 程式化
```

RTL 是這個流程中最關鍵的一環：上接架構設計，下接合成與實體實現。RTL 的品質直接決定了最終晶片的效能、面積與功耗。掌握 RTL 設計是數位積體電路設計工程師的核心技能，也是 eda4 專案中 verilog2fpga 與 verilog2rust 兩條工具鏈的共同起點。
