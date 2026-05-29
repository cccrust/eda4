# verilog-parser

Standalone Verilog parser — shared dependency of `v2f-synth` (verilog2fpga) and `verilog2rust`.

## Usage

```rust
use verilog_parser::parse::parse_verilog;
use verilog_parser::ast::Module;

let modules: Vec<Module> = parse_verilog(source_code);
```

## Test

```bash
cargo test
```

## Supported Syntax

- Module declaration with ports (input/output/inout)
- `reg`, `wire`, `[msb:lsb]` ranges
- `parameter`, `parameter [msb:lsb]`, `signed`
- `integer` type
- `assign`, `always @(*)` / `@(posedge/negedge ...)`
- `if` / `else`, `case` / `default`, `for` loop
- `begin` / `end` (named support via token only)
- Gate instantiation: `and`, `or`, `xor`, `not`, `buf`
- Module instantiation (positional & named ports)
- `initial` block
- `$display`, `$finish`, `$stime`, `$time`, `$readmemh`, `$readmemb`
- `#delay`
- Blocking (`=`) / non-blocking (`<=`)
- Operators: `+`, `-`, `&`, `|`, `^`, `~`, `<`, `>`, `==`
- Concatenation `{a, b}`, bit-select `a[i]`, part-select `a[msb:lsb]`

## Limitations

- `casex` / `casez` not yet implemented
- `nand` / `nor` / `xnor` gate primitives parsed as verbatim `Ident`
- Preprocessor directives (`` `define ``, `` `include ``) handled upstream by `verilog2rust`
