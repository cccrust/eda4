set -x
export RUST_BACKTRACE=1

echo "=== Verilog → Rust ==="
for f in verilog/*.v; do
    cargo run -- "$f"
done

echo ""
echo "=== Test ==="
cargo test
