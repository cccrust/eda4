# ruspice

Analog circuit simulator in Rust. Parses SPICE netlists (`.cir`), runs DC/AC/transient analysis.

## Quick start

```bash
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
| `cargo run -- cir <file.cir>` | Run SPICE netlist file |
| `cargo run -- list` | List available `.cir` examples in `circuits/` |
| `./run.sh` | Interactive runner |

## SPICE netlist syntax

```
* Comments
V<name> <pos> <neg> DC <value> [AC <amp>]
R<name> <pos> <neg> <value>
C<name> <pos> <neg> <value>
L<name> <pos> <neg> <value>
I<name> <pos> <neg> <value>
.TITLE <title>
.END
```

## Example circuits

Located in `circuits/`: divider, RC charge, RC low-pass, Wheatstone bridge, RLC bandpass, rectifier.

## Architecture

- `src/lib.rs` — Core: `Circuit`, `Solver`, `parse_spice`, analysis functions
- `src/main.rs` — CLI with demos
- `circuits/` — SPICE netlist examples

## Dependencies

`nalgebra`, `serde` / `serde_json`, `num-complex`
