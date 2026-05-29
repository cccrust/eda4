# verilog2rust — Verilog to ruHDL Translator + Runtime

Single crate — converts Verilog (`.v`) to Rust HDL (`ruHDL`), then compiles and runs it.

## Quick Start

```bash
# Convert Verilog to Rust
cargo run -- verilog/mcu0/mcu0m.v

# Copy hex file, compile & run
cp verilog/mcu0/mcu0m.hex .
rustc --extern verilog2rust=target/debug/libverilog2rust.rlib \
      -L target/debug -L target/debug/deps \
      verilog/mcu0/mcu0m.rs -o /tmp/test_mcu0m --edition 2021
/tmp/test_mcu0m
```

Or use the script:

```bash
./mcu0m.sh
```

## Test

```bash
cargo test -- --test-threads=1
```

## Supported Features

| Feature | Status |
|---|---|
| Basic gates (and, or, xor, not) | ✓ |
| `module` / `endmodule` | ✓ |
| `input` / `output` / `inout` | ✓ |
| `reg`, `wire`, `[msb:lsb]` | ✓ |
| `parameter` | ✓ |
| `parameter [msb:lsb]` | ✓ |
| `signed` keyword | ✓ |
| `integer` → native Rust `i64` | ✓ |
| `assign` | ✓ |
| `always @(*)` (combo) | ✓ |
| `always @(posedge/negedge clk)` | ✓ |
| `if` / `else` | ✓ |
| `case` / `default` | ✓ |
| `for` loop | ✓ |
| `begin` / `end` | ✓ |
| Blocking (`=`) / non-blocking (`<=`) | ✓ |
| `#delay` | ✓ |
| `initial` block | ✓ |
| `$display` | ✓ |
| `$finish` | ✓ |
| `$stime` / `$time` | ✓ |
| `$readmemh` / `$readmemb` (with comments, space-separated values) | ✓ |
| `` `define `` / `` `undef `` / `` `ifdef `` / `` `ifndef `` / `` `else `` / `` `endif `` | ✓ |
| `` `include `` | ✓ |
| Module instantiation (positional & named ports) | ✓ |
| Concatenation `{a, b}` | ✓ |
| Bit-select `a[i]`, part-select `a[msb:lsb]` | ✓ |
| Operator: `+`, `-`, `&`, `\|`, `^`, `~`, `<`, `>` | ✓ |
| `bus_to_u16` / `u16_to_bus` (LSB0) | ✓ |

## Limitations

- `casex` / `casez` not yet supported (treated as `case`)
- Argument-based macros (`` `define MACRO(a,b) ``) not supported
- `nand` / `nor` / `xnor` gate primitives parsed as verbatim `Ident`
