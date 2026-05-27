set -x
export RUST_BACKTRACE=1

echo "=== Build ==="
cargo build

echo ""
echo "=== Convert & Execute Testbenches ==="
for f in verilog/*_tb.v; do
    [ -f "$f" ] || continue
    stem=$(basename "$f" .v)
    echo ""
    echo "--- $stem ---"
    rhdl="verilog/${stem}.rhdl"
    cargo run -- "$f" "$rhdl"
    cargo run -- "$rhdl"
done

echo ""
echo "=== MCU0m Simulation ==="
cargo run -- verilog/mcu0m/mcu0m_sim.rs
