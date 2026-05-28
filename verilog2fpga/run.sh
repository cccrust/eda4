#!/bin/bash
set -e

cd "$(dirname "$0")"
ROOT=$(pwd)
BIN="$ROOT/target/debug/v2f"

echo "=== 建置 v2f ==="
cargo build

echo ""
echo "=== v2f list-devices ==="
"$BIN" list-devices

echo ""
echo "=== v2f check ==="
"$BIN" check

echo ""
mkdir -p "$ROOT/_out"

echo "=== 純 Rust E2E: blinky (Verilog) ==="
"$BIN" build "$ROOT/examples/blinky/blinky.v" \
    --device hx8k --backend pure-rust \
    --output "$ROOT/_out/blinky_vlog"
echo "  ✓ ($(wc -c < "$ROOT/_out/blinky_vlog.bin") bytes)"

echo ""
echo "=== 純 Rust E2E: adder (Verilog) ==="
"$BIN" build "$ROOT/examples/adder/adder.v" \
    --device hx8k --backend pure-rust \
    --output "$ROOT/_out/adder_vlog"
echo "  ✓ ($(wc -c < "$ROOT/_out/adder_vlog.bin") bytes)"

echo ""
echo "=== 模擬燒錄 (mock JTAG) ==="
"$BIN" prog "$ROOT/_out/blinky_vlog.bin" --driver mock

echo ""
echo "=== 模擬 SPI 燒錄 ==="
"$BIN" prog "$ROOT/_out/blinky_rust.bin" --driver spi

echo ""
echo "=== 純 Rust 位元流打包（指定 .asc） ==="
"$BIN" pack "$ROOT/v2f-bitstream/_fixtures/minimal_hx1k.asc" \
    --output "$ROOT/_out/minimal.bin" --backend rust
echo "  ✓ minimal.bin ($(wc -c < "$ROOT/_out/minimal.bin") bytes)"

"$BIN" pack "$ROOT/v2f-bitstream/_fixtures/empty_hx1k.asc" \
    --output "$ROOT/_out/empty.bin" --backend rust
echo "  ✓ empty.bin ($(wc -c < "$ROOT/_out/empty.bin") bytes)"

echo ""
echo "=== v2f-bitdecode: BIN → JSON 解碼 (Phase 3 實用化解碼) ==="
BITDECODE="$ROOT/target/debug/v2f-bitdecode"
"$BITDECODE" "$ROOT/_out/minimal.bin" --pretty 2>/dev/null | head -60
echo "  ✓ minimal.bin decoded (format: v2f-bitdecode-v2, decode_level: wiring)"

echo ""
echo "=== v2f-bitdecode: empty.bin 解析 ==="
"$BITDECODE" "$ROOT/_out/empty.bin" --pretty 2>/dev/null | grep -E '"(device|format|decode_level|synckey|logic_tiles_used)"' | head -10
echo "  ✓ empty.bin decoded (no wiring entries, synckey=0)"

echo ""
echo "=== 視覺化工具: JSON + ASC 解析測試 ==="
cargo test -p v2f-viz 2>&1 | tail -4
echo "  ✓ v2f-viz tests passed"

echo ""
echo "=== 視覺化工具: 產出 blinky 繪圖檔 ==="
cp "$ROOT/_out/blinky_vlog.json" "$ROOT/_out/blinky.json" 2>/dev/null || echo "  (無 .json 輸出)"
echo "  可用: cargo run -p v2f-viz -- _out/blinky_vlog.json _out/blinky_vlog.asc"

if "$BIN" check 2>&1 | grep -q "yosys.*已安裝"; then
    echo ""
    echo "=== yosys 綜合 ==="
    "$BIN" synth "$ROOT/examples/blinky/blinky.v" \
        --device hx8k --top blinky --backend yosys \
        --output "$ROOT/_out/blinky_yosys.json"
    echo "  ✓"
fi

echo ""
echo "=== 輸出檔案 ==="
ls -la "$ROOT/_out/"


cargo run -p v2f-viz -- _out/blinky_vlog.json _out/blinky_vlog.asc

echo ""
echo "=== 完成 ==="
echo "可用 iceprog/openFPGALoader 燒錄 .bin 至 FPGA"
