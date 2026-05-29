# eda4

EDA toolchain in Rust — monorepo with multiple sub-projects.

## Projects

| Path | Description |
|---|---|
| `verilog2rust/` | Verilog → Rust HDL translator + runtime |
| `verilog2fpga/` | iCE40 FPGA toolchain: synth → PnR → bitstream → programmer |
| `verilog-parser/` | Standalone Verilog parser (shared dependency) |
| `ruspice/` | Analog circuit simulator (SPICE) |
| `web/` | Web server + frontend (Actix + static HTML/JS) |
| `_wiki/` | Internal design docs |

## Quick start

```bash
# verilog2rust
cd verilog2rust && cargo run -- verilog/mcu0/mcu0m.v && cargo test

# verilog2fpga
cd verilog2fpga && cargo build && ./run.sh

# verilog-parser
cd verilog-parser && cargo test

# ruspice
cd ruspice && cargo run && cargo test

# web server
cd web && cargo run -p eda4-web-server
```

## Per-project README

- [verilog2rust](verilog2rust/README.md)
- [verilog2fpga](verilog2fpga/README.md)
- [verilog-parser](verilog-parser/README.md)
- [ruspice](ruspice/README.md)
- [web](web/README.md)
