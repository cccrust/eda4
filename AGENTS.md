# eda4 — Agent Guide

Rust EDA monorepo. Each sub-project is its own Cargo workspace with an independent lockfile.

## Top-level layout

| Path | Description |
|---|---|
| `verilog2fpga/` | Workspace (9 crates) — FPGA toolchain: synth, PnR, bitstream, programmer |
| `verilog2rust/` | Single crate — Verilog→Rust HDL translator + runtime |
| `verilog-parser/` | Standalone Verilog parser (used by v2f-synth AND verilog2rust) |
| `ruspice/` | Analog circuit simulator (see `ruspice/AGENTS.md`) |
| `web/` | Web workspace — `eda4-web-server` (actix-web) + static frontend + Playwright e2e |
| `_wiki/` | Internal design docs (43 markdown files) |

## Commands

### verilog2fpga workspace

```
cargo build                  # build all 9 workspace crates
cargo test                   # test all
cargo test -p <crate>        # test single crate (v2f-core, v2f-bitstream, etc.)
cargo run -p v2f-viz -- <json> <asc>   # GUI visualization
./test.sh                    # cargo build && cargo test
./run.sh                     # full E2E pipeline → _out/
```

`v2f` binary (`v2f-cli`): `build`, `synth`, `pnr`, `pack`, `prog`, `list-devices`, `check`. Backends: `auto` (default), `pure-rust`, `yosys`, `pnr-only`. Default device `hx8k`. Output files get `.json`, `.asc`, `.bin` suffixes.

### verilog2rust

```
cargo test                   # uses --test-threads=1 (see test.sh)
cargo run -- <file.v>        # Verilog → ruHDL (stdout)
cargo run -- <file.v> <out.rs>  # with output path
cargo run -- <file.rhdl>     # compile & run ruHDL
./run.sh                     # convert all verilog/*.v + cargo test
./run_tb.sh                  # convert testbenches (verilog/*_tb.v) + run them
./test.sh                    # cargo test -- --test-threads=1
```

Always set `RUST_BACKTRACE=1` when debugging verilog2rust (default in scripts).

### web workspace

```
cargo test -p eda4-web-server   # backend tests
cargo run -p eda4-web-server     # start server (port 8080)
cd frontend && npx playwright test  # e2e tests (needs server running)
```

Frontend is plain HTML+JS (no framework). E2E via Playwright (`tests/e2e.spec.js`).

### ruspice

See its own `ruspice/AGENTS.md` — standalone analog simulator with SPICE parser.

## Architecture notes

- **verilog2fpga** pipeline: `.v` → `v2f-synth` (JSON netlist) → `v2f-pnr` (ASC) → `v2f-bitstream` (BIN) → `v2f-programmer` (mock/real JTAG/SPI). Default is pure-Rust; yosys/nextpnr/icepack are optional (`brew install` on macOS).
- **verilog2rust** depends on `verilog-parser` via path. Testbenches: convert `.v` → `.rhdl` then run `.rhdl` (`run_tb.sh`). MCU0m sim uses `verilog/mcu0m/mcu0m_sim.rs`.
- **verilog-parser** is shared; both `v2f-synth` and `verilog2rust` use it (verilog2rust via path dep).
- **web server** depends on crates from across the monorepo: `verilog2rust`, `v2f-*`, `ruspice`, `base64`. Four tabs: Verilog sim, Verilog PnR, SPICE, Bitstream decode.
- **Device support**: iCE40 HX1K, HX4K, HX8K, LP1K, UP5K (case-insensitive). Default: `hx8k`.
- **Output dir**: `_out/` (gitignored).
- **Fixtures**: `v2f-bitstream/_fixtures/*.asc`.
- **`v2f-rust` / `v2f-rust-macros`**: exist only in docs/wiki (not yet implemented on disk).

## Known rHDL limitations (verilog2rust)

### 1. `@(posedge clock)` ignored — CPU simulation wrong
`always @(posedge clock)` has no `#delay` inside, so `has_delay_in_stmts()` returns false → categorized as `combo_always`, placed in `eval()` instead of the clocked loop in `run()`. CPU logic executes on EVERY eval tick instead of only on clock edges. Edge-triggered circuits (MCU0m, FSMs, counters) run at 10× the expected rate.

**Fix location**: `verilog2rust/src/verilog/gen.rs:45-50` — always-block categorization. Need a way to detect edge sensitivity (`posedge`/`negedge`) in the sensitivity list, or treat all `always @(*)` as combo and all `always @(posedge/negedge ...)` as clocked regardless of `#delay`.

### 2. `$finish` + delays — simulation terminates instantly
`DelayStmt` originally discarded the delay value (`Stmt::DelayStmt { stmt, .. }` → just `self.eval()`). Fixed 2026-05-29 to `for _ in 0..delay { self.eval(); }` (line 448-454). BUT: the while-loop structure puts both clocked always and delayed initials in the same loop body. `always #10` and `initial #2000 $finish` run sequentially each iteration, so `$finish` fires after 2000 evals on the first iteration. Correct behavior: concurrent time-advance loop where each block's delay is checked against a shared `sim_time` counter.

### 3. Possible SW bit ordering bug (MCU0m CMP)
Icarus shows `SW=8000` after first CMP, rHDL shows `SW=4000`. `bus_to_u16` reads bit `i` from wire `i` (`val |= 1 << i`), and `u16_to_bus` writes bit `i` to wire `i`. This is LSB0 (bit 0 = wire 0). The Verilog `SW[15]` should map to the MSB (wire 15), which it does in the generated code. Need to investigate why `0 < 10` evaluates to setting bit 14 instead of bit 15 — could be an issue in the generated comparison expression or the `s_w` vector layout.

### 4. MCU0m sim test status
`test_gen_mcu0m` exists (structural — parses + validates codegen). No simulation test yet because the rHDL output doesn't match Icarus. Need a functional simulation test that checks MCU0m behavior once the edge-trigger and SW issues are resolved.
