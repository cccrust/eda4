# eda4 — Agent Guide

Rust EDA monorepo with two sub-projects: **verilog2fpga** (FPGA toolchain) and **verilog2rust** (Verilog→Rust HDL translator). Both are Rust 2021 edition.

## Directory layout

```
verilog2fpga/     Rust workspace (10 crates)
  v2f-core/        Shared: Device enum (HX1K/HX4K/HX8K/LP1K/UP5K), Config, V2fError
  v2f-cli/         Binary crate `v2f` — clap CLI (build/synth/pnr/pack/prog/list-devices/check)
  v2f-synth/       Pure-Rust Verilog synthesis (parser → techmap → netlist → JSON)
  v2f-pnr/         Pure-Rust place & route (simmulated annealing)
  v2f-bitstream/   ASC → CRAM → BIN bitstream packing (fixtures: _fixtures/*.asc)
  v2f-programmer/  FPGA programmer (mock JTAG/SPI, optional `ftdi` feature for real hardware)
  v2f-rust/        Rust HDL (`fpga!` macro) → JSON compile → Verilog backend
  v2f-rust-macros/ proc-macro crate for the `fpga!` DSL
  v2f-db/          iCE40 device database (tile/CRAM address maps)
  v2f-viz/         eframe GUI binary `v2f-viz` for visualizing JSON+ASC
  examples/        blinky (v/pcf), adder (v)
verilog2rust/     Single crate: binary + library
  src/verilog/     Verilog tokenizer/parser/AST → code generator
  src/rhdl/        Simulation runtime (Signal, Gate trait, sim harness)
  verilog/         Example designs & testbenches (.v + .rhdl)
  verilog/hackcpu/ Hack CPU (gate-level, memory, mux, PC, ALU)
  verilog/mcu0m/   MCU0m simulation
  tests/           Comprehensive tests (parsing, codegen, syntax round-trip)
```

## Commands

### verilog2fpga

| Command | Description |
|---|---|
| `cargo build` | Build all workspace crates |
| `cargo test` | Test all workspace crates |
| `cargo test -p <crate>` | Test single crate (e.g., `-p v2f-core`, `-p v2f-bitstream`) |
| `cargo run -p v2f-viz -- <input.json> <input.asc>` | Launch GUI visualization |
| `./test.sh` | `cargo build && cargo test` |
| `./run.sh` | Full E2E pipeline (build blinky/adder via pure Rust and yosys) |

The `v2f` binary supports `--backend auto` (try yosys/nextpnr/icepack, fallback Rust), `--backend pure-rust` (Rust only), `--backend yosys`, `--backend pnr-only`. Default device is `hx8k`. Output files get `.json`, `.asc`, `.bin` suffixes appended to `--output`.

### verilog2rust

| Command | Description |
|---|---|
| `cargo build` | Build crate |
| `cargo test` | Run tests (no test binary to run; pure `#[test]` tests) |
| `cargo run -- <file.v>` | Convert Verilog → ruHDL |
| `cargo run -- <file.v> <output.rs>` | Convert with explicit output path |
| `cargo run -- <file.rhdl>` | Compile and run ruHDL |
| `./run.sh` | Convert all `verilog/*.v` files + run tests |
| `./run_tb.sh` | Build + convert testbenches + run them |
| `./test.sh` | `cargo test` |

Set `RUST_BACKTRACE=1` when debugging verilog2rust (used by default in scripts).

### External tools (optional, macOS)

```sh
brew install yosys icestorm openfpgaloader
brew install nextpnr-ice40  # or: brew tap siliconwitchery/oss-fpga && brew install --HEAD siliconwitchery/oss-fpga/nextpnr-ice40
```

Only needed for `--backend yosys` / `icepack` / physical FPGA programming.

## Architecture notes

- **verilog2fpga** uses a pure-Rust EDA pipeline as default (synth → PNR → bitstream). External yosys/nextpnr/icepack are optional. Pipeline: `.v` → `v2f-synth` (JSON netlist) → `v2f-pnr` (ASC) → `v2f-bitstream` (BIN) → `v2f-programmer` (JEDEC/SPI).
- **verilog2rust** parses Verilog, generates Rust code using the rhdl runtime (Signal + Gate trait), and can execute via rustc compilation. The library crate is compiled once to `/tmp/verilog2rust_rlib/` and cached; `.rhdl` files are compiled as separate binaries.
- **Device support**: iCE40 HX1K, HX4K, HX8K, LP1K, UP5K. Device string is case-insensitive.
- **Bitstream packing** reads `.asc` (ASCII place-and-route output) and produces `.bin` (bitstream). Fixtures are in `v2f-bitstream/_fixtures/`.
- **Output directory**: `_out/` (gitignored), created by `run.sh`.
- **HDL DSL**: `v2f-rust` provides the `fpga!{ module ... }` macro for expressing hardware in Rust syntax, callable from the `v2f build --lang rust` command.
