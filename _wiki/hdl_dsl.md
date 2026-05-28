# Rust HDL DSL 與 fpga! 巨集

## 什麼是 HDL DSL

嵌入式硬體描述領域特定語言（HDL DSL）是指將硬體描述語言（如 Verilog）的語法與語義嵌入到通用程式語言（Rust）中，讓開發者能在不離開 Rust 生態系的情況下書寫與合成數位電路。eda4 專案在 `v2f-rust` 與 `v2f-rust-macros` 這兩個 crate 中實作了一套此類 DSL，核心是 `fpga!` 程序式巨集（proc-macro）。

## fpga! 巨集

`v2f-rust-macros` crate 定義了一個程序式巨集 `fpga!`，接受 Verilog 風格的語法作為輸入，在編譯期解析並展開為 Rust 程式碼（`macros/src/lib.rs:26-35`）。

### DSL 語法

```rust
use v2f_rust::fpga;

let blinky = fpga! {
    module Blinky {
        input clk: 1,
        output led: 1,
        reg [25:0] counter,
        always(posedge clk) {
            counter <= counter + 1;
        }
        assign led = counter[25];
    }
};
```

支援的語法結構：
- **port 宣告**：`input name: width`、`output name: width`、`inout name: width`
- **reg 宣告**：`reg [msb:lsb] name`（多位元）或 `reg name`（單位元）
- **assign 連續賦值**：`assign target = expression;`
- **always 區塊**：`always(posedge clk) { ... }`，支援 `<=`（非阻塞）與 `=`（阻塞）賦值
- **表達式**：常數、識別符號、索引 `[i]`、範圍 `[msb:lsb]`、`+`、`-`、`&`、`|`、`^`、`~`

### 編譯期展開

`fpga!` 巨集使用自製的手寫遞迴下降解析器（`Cursor` 結構體，`macros/src/lib.rs:264-369`）解析輸入字串。解析結果透過 `generate_code()` 函數（`macros/src/lib.rs:375-430`）展開為 Rust 程式碼，呼叫 `v2f_rust::HdlModule` 的 builder 方法：

```rust
{{
    let mut __m = v2f_rust::HdlModule::new("Blinky");
    __m = __m.input("clk", 1);
    __m = __m.output("led", 1);
    __m = __m.reg("counter", 26);
    __m = __m.dff("counter", v2f_rust::HdlExpr::Add(
        Box::new(v2f_rust::HdlExpr::Ident("counter".to_string())),
        Box::new(v2f_rust::HdlExpr::Const(1, 1))
    ));
    __m = __m.assign("led", v2f_rust::HdlExpr::Index(
        Box::new(v2f_rust::HdlExpr::Ident("counter".to_string())),
        25
    ));
    __m
}}
```

## v2f-rust crate

`v2f-rust` 是非巨集部分的實作（`v2f-rust/src/`）。它定義了中間表示（IR）與後端輸出。

### HdlExpr — 表達式 IR

`HdlExpr` 列舉（`hdl.rs:1-14`）描述硬體表達式：

```rust
pub enum HdlExpr {
    Const(u64, u32),            // 常數（值, 位元寬度）
    Ident(String),              // 識別符號
    Index(Box<HdlExpr>, u32),   // 位元選取  expr[i]
    Range(Box<HdlExpr>, u32, u32), // 範圍選取 expr[msb:lsb]
    Concat(Vec<HdlExpr>),       // 串接 {a, b}
    Add(Box<HdlExpr>, Box<HdlExpr>), // 加法
    Sub(Box<HdlExpr>, Box<HdlExpr>), // 減法
    And(Box<HdlExpr>, Box<HdlExpr>), // 位元 AND
    Or(Box<HdlExpr>, Box<HdlExpr>),  // 位元 OR
    Xor(Box<HdlExpr>, Box<HdlExpr>), // 位元 XOR
    Not(Box<HdlExpr>),               // 位元 NOT
}
```

### HdlStmt — 陳述式 IR

`HdlStmt` 列舉（`hdl.rs:16-23`）描述模組內的陳述：

```rust
pub enum HdlStmt {
    Assign { target: String, value: HdlExpr },         // assign
    Blocking { target: String, value: HdlExpr },       // = （阻塞賦值）
    Nonblocking { target: String, value: HdlExpr },    // <=（非阻塞賦值）
    DeclReg { name: String, width: u32 },              // reg 宣告
    DeclWire { name: String, width: u32 },             // wire 宣告
}
```

### HdlPort 與 HdlPortDir

連接埠宣告（`hdl.rs:25-37`）：

```rust
pub struct HdlPort {
    pub name: String,
    pub direction: HdlPortDir,
    pub width: u32,
}

pub enum HdlPortDir {
    Input,
    Output,
    Inout,
}
```

### HdlModule — builder 模式

`HdlModule`（`hdl.rs:39-88`）採用 builder 模式建構：

```rust
pub struct HdlModule {
    pub name: String,
    pub ports: Vec<HdlPort>,
    pub stmts: Vec<HdlStmt>,
}
```

方法一覽：

| 方法 | 對應 Verilog | 說明 |
|------|--------------|------|
| `input(name, width)` | `input [w-1:0] name` | 新增輸入埠 + 自動宣告 wire |
| `output(name, width)` | `output [w-1:0] name` | 新增輸出埠 + 自動宣告 wire |
| `inout(name, width)` | `inout [w-1:0] name` | 新增雙向埠 + 自動宣告 wire |
| `reg(name, width)` | `reg [w-1:0] name` | 宣告暫存器 |
| `assign(target, value)` | `assign target = value` | 連續賦值 |
| `blocking(target, value)` | `target = value` | 阻塞賦值 |
| `dff(target, value)` | `target <= value` | 非阻塞賦值（觸發器） |

所有方法都回傳 `Self`，可鏈式呼叫：

```rust
HdlModule::new("Top")
    .input("clk", 1)
    .output("led", 4)
    .reg("counter", 26)
    .assign("led", HdlExpr::Index(Box::new(HdlExpr::Ident("counter".into())), 25))
```

### compile() — Yosys-JSON 輸出

`compile()` 函數（`compile.rs:39-163`）將 `HdlModule` 編譯為 Yosys 相容的 JSON 格式（`SynthOutput`）。編譯過程：

1. **位元分配**：為每個連接埠與內部訊號分配遞增的位元編號（`next_bit`）
2. **單元生成**：
   - 非阻塞賦值（`Nonblocking`）→ `$_DFF_P_` 單元（正緣觸發 D 型觸發器）
   - `Assign` 與 `Blocking` → `$_OUTPUT_` 單元（連接線）
3. **JSON 輸出**：符合 Yosys JSON 後端格式，包含 `creator`、`modules`、`ports`、`cells`、`netnames`

輸出的 JSON 可直接餵給 `v2f-pnr` 進行佈局佈線。

### to_verilog() — Verilog 輸出

`to_verilog()` 函數（`backend.rs:3-42`）將 `HdlModule` 轉回 Verilog 原始碼：

```verilog
module Blinky (
  input clk,
  output led
);
  reg [25:0] counter;
  assign led = counter[25];
  counter <= counter + 1;
endmodule
```

## 在 pipeline 中的角色

完整的合成流程：

```
Rust 原始碼 → fpga! 巨集 → HdlModule (IR)
                                   ↓
                          compile() 或 to_verilog()
                                   ↓
                         Yosys-JSON (或 Verilog)
                                   ↓
                            v2f-pnr (place & route)
                                   ↓
                                 ASC
                                   ↓
                          v2f-bitstream (packing)
                                   ↓
                                 BIN
```

階層：
1. 使用者撰寫含 `fpga!` 巨集的 Rust 程式
2. 編譯期巨集展開為 `HdlModule` builder 呼叫
3. `HdlModule` 可序列化為 Yosys-JSON
4. JSON 進入純 Rust 的 PNR 流程
5. 最終產出 FPGA 位元流（.bin）

## 與其他 Rust HDL 的比較

### rust-hdl

`rust-hdl` 使用 Rust 的類型系統（泛型、const generic）來建模硬體。訊號寬度透過類型參數表達，電路結構在型別層級組合。優點是類型安全，缺點是學習曲線陡峭、錯誤訊息不直觀。

### veryl

`veryl` 是一種全新的硬體描述語言，語法類似 Rust 但獨立編譯。它不像 `fpga!` 那樣嵌入在 Rust 中，而是有自己的編譯器，輸出 Verilog。

### hdl (另一套 Rust HDL)

另有名為 `hdl` 的 Rust crate，也是內嵌 DSL，但語法更接近 Rust 本身的閉包與 trait，而非 Verilog 風格。

### fpga! 的優勢

- **熟悉的 Verilog 語法**：硬體工程師不需學習新的 DSL 語法，直接寫類似 Verilog 的程式碼
- **編譯期檢查**：巨集在編譯期解析，語法錯誤在編譯期即報錯
- **無縫整合 CLI**：`v2f build --lang rust` 可直接處理 `.rs` 檔案，不需要額外的建置步驟
- **與純 Rust PNR 流程互通**：不需要外部合成工具（yosys），整個流程在 Rust 內完成
- **雙向轉換**：可輸出 Yosys-JSON 或 Verilog，視需求選擇後端

## 使用方式：v2f CLI

透過 `v2f` 命令列工具，可以直接從 Rust 原始檔開始建置：

```sh
v2f build --lang rust Blinky.rs --output _out/blinky --device hx8k
```

CLI 會：
1. 編譯 Rust 原始碼（需有 `main` 函數或 `fpga!` 巨集）
2. 提取 `fpga!` 產生的 `HdlModule`
3. 呼叫 `compile()` 輸出 JSON
4. 送入 PNR → bitstream 流程
5. 產生 `_out/blinky.json`、`_out/blinky.asc`、`_out/blinky.bin`

完整範例（Blinky.rs）：

```rust
use v2f_rust::fpga;

fn main() {
    let _module = fpga! {
        module Blinky {
            input clk: 1,
            output led: 1,
            reg [25:0] counter,
            always(posedge clk) {
                counter <= counter + 1;
            }
            assign led = counter[25];
        }
    };
}
```

## Cursor 解析器內部

`fpga!` 巨集使用手寫的 `Cursor` 解析器（`macros/src/lib.rs:264-369`），以字元陣列為基礎，逐字掃描：

- `peek()` 查看當前字元但不消耗
- `eat_char(c)` 消耗一個特定字元
- `eat_str(s)` 消耗一個特定字串
- `expect(s)` 類似 `eat_str`，但失敗時回傳含位置資訊的錯誤
- `ident()` 解析識別符號（字母或底線開頭，後接字母/數字/底線）
- `number()` 解析十進位數字
- `check(s)` 向前看是否匹配某字串（自動跳過空白，且確保關鍵字不會部分匹配）

`skip_ws_and_comma()` 會在每個宣告之間跳過空白與選擇性逗號，讓使用者可以自由選擇是否在宣告結尾加逗號。

解析器採用遞迴下降（recursive descent）配合運算子優先級（precedence climbing）處理表達式：

- `parse_expr(p, min_prec)` 處理 `+`、`-`（優先級 10）與 `&`、`|`、`^`（優先級 5）
- `parse_primary(p)` 處理常數、識別符號、括號、`~` 運算、以及 `[i]` / `[msb:lsb]` 後綴

## 與 verilog2rust 程式碼生成的關係

eda4 專案中存在兩套互補的硬體生成路徑：

### verilog2rust（translator 路線）

從 `verilog2rust` crate 出發，將 `.v` 檔案解析為 AST，再產生 Rust 程式碼（使用 rhdl 模擬引擎）。生成的程式碼包含 `WireRef`、閘結構體與 `eval()` 方法，可直接編譯執行。這是傳統的「Verilog → Rust」轉換。

### fpga!（DSL 路線）

從 Rust 原始碼出發，使用 `fpga!` 巨集在編譯期定義硬體，產生 `HdlModule` IR，再透過 `compile()` 輸出 Yosys-JSON 進行合成。這是「Rust → JSON → PNR → bitstream」的純 Rust 合成路徑。

### 兩者的互通性

雖然兩條路線目標不同（一條是模擬、一條是合成），但它們共用了一些設計概念：

- `HdlExpr` 與 `HdlStmt` 的設計參考了 Verilog AST 的結構
- 非阻塞賦值（`<=`）在 DSL 中對應 `dff()`，在 rhdl 中對應 `add_seq` 閉包
- 兩者都以「模組」為最上層單位，包含連接埠與內部邏輯

## 程式庫的公開 API

`v2f-rust` crate 的公開介面（`lib.rs:1-9`）：

```rust
pub mod hdl;
pub mod compile;
pub mod backend;

pub use hdl::{HdlModule, HdlExpr, HdlStmt, HdlPortDir};
pub use compile::compile;
pub use backend::to_verilog;
pub use v2f_rust_macros::fpga;
```

使用者只需依賴 `v2f-rust`，`v2f-rust-macros` 會透過 `pub use` 自動匯出。呼叫路徑為：

```
v2f_rust::fpga! { ... }        // 巨集（來自 v2f_rust_macros）
v2f_rust::HdlModule            // IR 型別
v2f_rust::compile(&module)     // 轉 JSON
v2f_rust::to_verilog(&module)  // 轉 Verilog
```

## 現有侷限

- **Verilog 子集**：僅支援 `always(posedge clk)` 形式的同步邏輯，不支援 `always @(*)`、`case`、`for` 迴圈等結構
- **無階層式設計**：`fpga!` 巨集目前只產生單一模組，不支援模組實例化（module instantiation）
- **無參數化**：沒有 Verilog 的 `parameter` 或 `generate` 機制，位元寬度必須是字面常數
- **表達式限制**：`+` `-` `&` `|` `^` `~` 以外的運算元（乘法、移位、比較）尚未實作
- **手寫解析器**：`fpga!` 巨集使用手寫的 `Cursor` 解析器而非 `syn`/`nom` 等函式庫，遇到複雜語法時較難擴充
- **always 區塊僅一個敏感信號**：`always(posedge clk)` 只支援單一邊緣觸發，不支援 `posedge clk or negedge rst`

## 延伸閱讀

- [Domain-Specific Language (Wikipedia)](https://en.wikipedia.org/wiki/Domain-specific_language)
- [Hardware Description Language (Wikipedia)](https://en.wikipedia.org/wiki/Hardware_description_language)
- [Embedded Domain-Specific Language (Wikipedia)](https://en.wikipedia.org/wiki/Embedded_domain-specific_language)
- [Macro (Computer Science) (Wikipedia)](https://en.wikipedia.org/wiki/Macro_(computer_science))
