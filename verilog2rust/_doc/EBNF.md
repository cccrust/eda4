# EBNF — verilog2rust 支援的 Verilog 子集

```ebnf
(* ─── 頂層 ─── *)
source_file      = { module_declaration } ;
module_declaration
                 = "module" module_name "(" port_name_list ")" ";"
                   { port_decl } { module_item }
                   "endmodule"
                 | "module" module_name ";"
                   { port_decl } { module_item }
                   "endmodule" ;

(* ─── 連接埠 ─── *)
port_name_list   = ident { "," ident } ;
port_decl        = "input"  [ range ] id_list ";"
                 | "output" [ "reg" ] [ range ] id_list ";"
                 | "inout"  [ range ] id_list ";" ;
id_list          = ident { "," ident } ";" ;

(* ─── 模組本體項目 ─── *)
module_item      = wire_decl
                 | reg_decl
                 | integer_decl
                 | param_decl
                 | assign_stmt
                 | always_block
                 | initial_block
                 | gate_inst
                 | module_inst ;

wire_decl        = "wire"   [ range ] decl_entry { "," decl_entry } ";" ;
reg_decl         = "reg"    [ range ] decl_entry { "," decl_entry } ";" ;
integer_decl     = "integer" id_list ;
decl_entry       = ident [ array_dim ] ;
param_decl       = "parameter" param_assign { "," param_assign } ";" ;
param_assign     = ident "=" const_expr ;

(* ─── 範圍與維度 ─── *)
range            = "[" const_expr ":" const_expr "]" ;
array_dim        = "[" const_expr [ ":" const_expr ] "]" ;

(* ─── 持續賦值 ─── *)
assign_stmt      = "assign" expr "=" expr ";" ;

(* ─── 程序區塊 ─── *)
always_block     = "always" [ "@" sensitivity_list ] statement_or_block ;
initial_block    = "initial" statement_or_block ;

sensitivity_list = "(" sensitivity { ( "or" | "," ) sensitivity } ")"
                 | "(" "*" ")" ;
sensitivity      = "posedge" ident
                 | "negedge" ident
                 | ident ;  (* 隱含 posedge *)

(* ─── 敘述 ─── *)
statement_or_block
                 = "begin" { statement } "end"
                 | statement ;

statement        = blocking_assign
                 | nonblocking_assign
                 | if_stmt
                 | case_stmt
                 | for_stmt
                 | forever_stmt
                 | syscall
                 | delay_stmt ;

blocking_assign  = lhs "="  expr ";" ;
nonblocking_assign
                 = lhs "<=" expr ";" ;

lhs              = ident
                 | ident "[" expr "]"
                 | ident "[" expr ":" expr "]"
                 | "{" lhs { "," lhs } "}" ;

if_stmt          = "if" "(" expr ")" statement_or_block
                   [ "else" statement_or_block ] ;
case_stmt        = "case" "(" expr ")"
                   { case_item { "," case_item } ":" case_body }
                   [ "default" ":" case_body ]
                   "endcase" ;
case_item        = expr ;
case_body        = { statement } ;
for_stmt         = "for" "(" [ assign_stmt ] expr ";" [ assign_stmt ] ")"
                   statement_or_block ;
forever_stmt     = "forever" statement_or_block ;
syscall          = "$display" "(" [ expr { "," expr } ] ")" ";"
                 | "$monitor" "(" [ expr { "," expr } ] ")" ";"
                 | "$finish" ";" ;
delay_stmt       = "#" number [ statement ] ;

(* ─── 閘級例化 ─── *)
gate_inst        = gate_type [ "#" "(" delay_val ")" ]
                   [ instance_name ] "(" expr { "," expr } ")" ";" ;
gate_type        = "and" | "nand" | "or" | "nor" | "xor" | "xnor"
                 | "not" | "buf"
                 | "bufif0" | "bufif1" | "notif0" | "notif1" ;

(* ─── 模組例化 ─── *)
module_inst      = module_name [ "#" "(" { param_assign "," } ")" ]
                   instance_name "(" connection_list ")" ";" ;
connection_list  = connection { "," connection } ;
connection       = "." port_name "(" expr ")"  (* by‑name *)
                 | expr ;                     (* by‑order *)

(* ─── 表示式（Pratt 解析器，優先順序遞減）─── *)
expr             = ternary_expr ;
ternary_expr     = logical_or_expr [ "?" expr ":" ternary_expr ] ;

logical_or_expr  = logical_and_expr { "||" logical_and_expr } ;
logical_and_expr = bitwise_or_expr { "&&" bitwise_or_expr } ;
bitwise_or_expr  = bitwise_xor_expr { "|" bitwise_xor_expr } ;
bitwise_xor_expr = bitwise_and_expr { ("^" | "^~" | "~^") bitwise_and_expr } ;
bitwise_and_expr = equality_expr { "&" equality_expr } ;
equality_expr    = relational_expr { ("==" | "!=") relational_expr } ;
relational_expr  = shift_expr { ("<" | "<=" | ">" | ">=") shift_expr } ;
shift_expr       = additive_expr { ("<<" | ">>") additive_expr } ;
additive_expr    = multiplicative_expr { ("+" | "-") multiplicative_expr } ;
multiplicative_expr
                 = unary_expr { ("*" | "/" | "%") unary_expr } ;

unary_expr       = primary
                 | "+" primary
                 | "-" primary
                 | "~" primary
                 | "!" primary
                 | "&" primary   (* reduce‑and *)
                 | "|" primary   (* reduce‑or  *)
                 | "^" primary ; (* reduce‑xor *)

primary          = number_lit
                 | string_lit
                 | ident
                 | ident "[" expr "]"
                 | ident "[" expr ":" expr "]"
                 | "(" expr ")"
                 | "{" expr { "," expr } "}"       (* concat *)
                 | "{" expr "{" expr "}" "}"       (* replicate *) ;

const_expr       = (* 僅含常數折疊支援的運算：加法、減法、乘法、一元負號 *) number_lit | ident;

(* ─── 常值 ─── *)
number_lit       = sized_number | unsized_number | decimal_number ;
sized_number     = decimal_digits "'" base_digit digit { digit | "_" } ;
unsized_number   = "'" base_digit digit { digit | "_" } ;
decimal_number   = digit { digit } ;
base_digit       = "b" | "B" | "o" | "O" | "d" | "D" | "h" | "H" ;
string_lit       = '"' { character } '"' ;

(* ─── 識別字 ─── *)
ident            = ( letter | "_" | "$" ) { letter | digit | "_" | "$" } ;
module_name      = ident ;
instance_name    = ident ;
port_name        = ident ;
```

## 說明

- 僅支援傳統（non‑ANSI）連接埠宣告：`module foo(a, b); input a, b; ...`
- `output reg` 為唯一允許的輸出暫存器速記；不支援獨立的 `output` + `reg` 分開宣告
- `integer` 視為 `reg [31:0]`（32 位元有號，但產出 Rust 時一律用 `u16`）
- 陣列維度 `array_dim`：單一數字 `[N]` 表示大小 `N+1`；範圍 `[H:L]` 表示 `|H-L|+1`
- 不支援：ANSI ports、`function`/`task`、`generate`、`while`/`repeat`、`$readmemh`、
  `===`/`!==`、`signed`、`defparam`、`specify`、tri‑state 以外的線網型態
- 巨集 `` `include `` 由前置處理器（preprocessor）展開而非語法解析器處理
```
