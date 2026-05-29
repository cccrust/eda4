# eda4 — Agent Guide

Rust EDA monorepo. Each sub-project is its own Cargo workspace with independent lockfile. No CI, no `opencode.json`.

## Layout

| Path | Description |
|---|---|
| `verilog2fpga/` | Workspace (9 crates) — iCE40 FPGA toolchain: synth→PnR→bitstream→programmer |
| `verilog2rust/` | Single crate — Verilog→Rust HDL translator + runtime |
| `verilog-parser/` | Standalone parser (shared dep of `v2f-synth` and `verilog2rust`) |
| `ruspice/` | Analog circuit simulator (see `ruspice/AGENTS.md`) |
| `web/` | Workspace (`eda4-web-server` actix-web) + static HTML/JS frontend + Playwright e2e |
| `_wiki/` | Internal design docs (43 .md files, ref not code) |

## Commands

### verilog2fpga

```
cargo build                    # build all 9 crates
cargo test                     # test all
cargo test -p <crate>          # test single crate
cargo run -p v2f-viz -- <json> <asc>
./test.sh                      # cargo build && cargo test
./test_cross.sh                # cross-validation: run tests tagged `cross` per layer
./run.sh                       # full E2E pipeline → _out/
```

`v2f` CLI: `build`, `synth`, `pnr`, `pack`, `prog`, `list-devices`, `check`. Backends: `auto` (default), `pure-rust`, `yosys`, `pnr-only`. Device default `hx8k`. Output: `.json`, `.asc`, `.bin`.

Separate binary `v2f-bitdecode` in same workspace: `cargo run -p v2f-bitdecode -- <bin> [--pretty]`.

Fixtures: `v2f-bitstream/_fixtures/*.asc`.

### verilog2rust

```
cargo test                     # uses --test-threads=1
cargo run -- <file.v>          # Verilog → ruHDL (stdout)
cargo run -- <file.rhdl>       # compile & run ruHDL
./run.sh                       # convert all verilog/*.v + cargo test
./run_tb.sh                    # convert testbenches (verilog/*_tb.v) + run them
./test.sh                      # cargo test -- --test-threads=1
```

Always `RUST_BACKTRACE=1` when debugging (default in scripts). Tests compile generated code via rustc at runtime (`/tmp/v2r_*.rs`).

### web

```
cargo test -p eda4-web-server   # backend tests
cargo run -p eda4-web-server     # server on :8080
./web.sh                         # kills old server, starts fresh, runs Playwright e2e
cd frontend && npx playwright test  # e2e (needs server already running)
```

Playwright config at `frontend/playwright.config.js` — tests in `frontend/tests/e2e.spec.js`. Frontend is plain HTML+JS, no framework.

### ruspice

See `ruspice/AGENTS.md`. Standalone — `cargo test`, `cargo run`, `cargo run --example basic`.

## Architecture

- **verilog2fpga pipeline**: `.v` → `v2f-synth` (JSON netlist) → `v2f-pnr` (ASC) → `v2f-bitstream` (BIN) → `v2f-programmer` (mock/real JTAG/SPI).
- **verilog2rust** depends on `verilog-parser` via path. Converts `.v` → ruHDL; testbenches get `fn main()`. `verilog/mcu0m/mcu0m_sim.rs` is hand-written MCU0m simulation.
- **verilog-parser** has 3 sources: `lib.rs`, `ast.rs`, `parse.rs` — used by both `v2f-synth` and `verilog2rust`.
- **web server** embeds `verilog2rust`, `v2f-*`, `ruspice`, `base64`. Four tabs: Verilog sim, Verilog PnR, SPICE, Bitstream decode.
- **iCE40 devices**: HX1K, HX4K, HX8K, LP1K, UP5K (case-insensitive). Default HX8K.
- **Output dir**: `_out/` (gitignored).
- **`v2f-rust` / `v2f-rust-macros`**: documented in wiki, not yet implemented.

## Known rHDL limitations (verilog2rust)

### 1. `@(posedge clock)` ignored — CPU runs at 10× speed
`always @(posedge clock)` without `#delay` inside → `has_delay_in_stmts()` is false → categorized as `combo_always` → logic placed in `eval()` instead of the clocked loop in `run()`. Edge-triggered circuits execute every eval tick.

**Fix**: `verilog2rust/src/verilog/gen.rs:45-50` — detect edge sensitivity (`posedge`/`negedge`) in sensitivity list, or treat all `@(*)` as combo and all `@(posedge/negedge ...)` as clocked regardless of `#delay`.

### 2. `$finish` + delays — simulation terminates early
`DelayStmt` was fixed from discarding delays to `for _ in 0..delay { self.eval(); }` (line 448-454). But delayed blocks and clocked always blocks share the same while-loop body → `initial #2000 $finish` fires after 2000 evals on the first iteration.

**Fix needed**: concurrent time-advance loop with shared `sim_time` counter.

### 3. Possible SW bit ordering bug
Icarus shows `SW=8000` after first CMP, rHDL shows `SW=4000`. LSB0 mapping (bit `i` = wire `i`) looks correct for `SW[15]` → wire 15. Investigate generated comparison expression or `s_w` vector layout.

### 4. MCU0m sim test status
`test_gen_mcu0m` validates codegen only. No functional sim test — rHDL output doesn't match Icarus yet.
