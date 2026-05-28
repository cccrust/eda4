# ruspice — Agent Guide

Analog circuit simulator in Rust. Parses SPICE netlists (`.cir`), runs DC/AC/transient analysis.

## Quick start

```sh
cargo run                    # Run demos (RC, divider, AC filter)
cargo run --example basic    # Example circuits
cargo test                   # Run all tests
```

## Commands

| Command | Description |
|---|---|
| `cargo run` | Run all demos |
| `cargo run -- <cmd>` | Run specific demo: `dc`, `transient`, `ac`, `all`, `circuit`, `plot-dc`, `plot-transient`, `svg [file]`, `save-plot [f] [n]` |
| `cargo run --example basic` | Run basic example circuits |
| `cargo test` | Run all tests |
| `./run.sh` | Interactive runner: `./run.sh test`, `./run.sh circuit`, `./run.sh plot-dc`, `./run.sh svg`, etc. |
| `./run.sh all` | Run all demos and generate SVG outputs |
| `./test.sh` | Runs `cargo test` |

## SPICE netlist support

ruspice can parse and simulate standard SPICE netlists:

```sh
cargo run -- cir <file.cir>    # Run .cir file (DC + AC + transient)
cargo run -- list               # List available .cir examples in circuits/
```

### Supported netlist syntax

```
* Comments
V<name> <node_pos> <node_neg> DC <value> [AC <amplitude>]
R<name> <node_pos> <node_neg> <value>
C<name> <node_pos> <node_neg> <value>
L<name> <node_pos> <node_neg> <value>
I<name> <node_pos> <node_neg> <value>
.TITLE <title>
.END
```

### Value suffixes

| Suffix | Multiplier | Example |
|---|---|---|
| k/K | 1e3 | `1k` = 1000 |
| m/M | 1e-3 | `1m` = 0.001 |
| u/U | 1e-6 | `1u` = 1e-6 |
| n/N | 1e-9 | `1n` = 1e-9 |
| p/P | 1e-12 | `1p` = 1e-12 |
| f/F | 1e-15 | `1f` = 1e-15 |

### Example circuits

Located in `circuits/`:
- `divider.cir` — Resistor divider (Vout = Vin/2)
- `rc_charge.cir` — RC charging (τ = 1ms, 5V source)
- `rc_lowpass.cir` — RC low-pass filter (fc ≈ 1.59kHz)
- `wheatstone.cir` — Wheatstone bridge
- `rlc_bandpass.cir` — RLC bandpass filter
- `rectifier.cir` — Diode rectifier

## Architecture

- `src/lib.rs` — Core library: `Circuit`, `Component`, `Solver`, analysis functions (`analyze_dc`, `analyze_ac`, `analyze_transient`), SPICE parser (`parse_spice`)
- `src/main.rs` — CLI binary with demos and commands
- `examples/basic.rs` — Example circuits
- `circuits/` — SPICE netlist examples

### Analysis functions

- `analyze_dc(&circuit)` — DC operating point (returns `DCAnalysisResult`)
- `analyze_ac(&circuit, start_freq, end_freq, num_points)` — AC frequency sweep (returns `ACAnalysisResult`)
- `analyze_transient(&circuit, start_time, end_time, time_step)` — Time-domain (returns `TransientResult`)

### SPICE parser

```rust
use ruspice::{parse_spice, run_analysis, AnalysisOptions};

let netlist = std::fs::read_to_string("circuit.cir").unwrap();
let circuit = parse_spice(&netlist).unwrap();
let opts = AnalysisOptions::default();
run_analysis(&circuit, &opts);
```

`AnalysisOptions` fields: `dc`, `ac`, `transient`, `ac_start`, `ac_end`, `ac_points`, `tran_start`, `tran_end`, `tran_step`.

## Dependencies

- `nalgebra` — Linear algebra (DMatrix, DVector)
- `serde` / `serde_json` — Serialization
- `num-complex` — Complex numbers for AC analysis

## Notes

- Ground node is always `gnd` (node 0)
- Phase in AC analysis currently shows 90° due to simplified complex handling (real admittance model)
- AC magnitude is correct for RC/RL circuits
- Transient analysis uses backward Euler integration