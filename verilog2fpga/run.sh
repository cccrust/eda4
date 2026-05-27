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

synth_example() {
    local name=$1
    echo "=== $name: 綜合 ==="
    if "$BIN" check 2>&1 | grep -q "yosys.*已安裝"; then
        "$BIN" synth "$ROOT/examples/$name/$name.v" \
            --device hx8k --top "$name" \
            --output "$ROOT/_out/$name.json"
        echo "  ✓ 綜合成功"
    else
        echo "  ✗ yosys 未安裝，跳過"
    fi
}

synth_example blinky
synth_example adder

echo ""
echo "=== 純 Rust 位元流打包（無需外部工具） ==="
"$BIN" pack "$ROOT/v2f-bitstream/_fixtures/minimal_hx1k.asc" \
    --output "$ROOT/_out/minimal.bin" --backend rust
echo "  ✓ minimal.bin ($(wc -c < "$ROOT/_out/minimal.bin") bytes)"

"$BIN" pack "$ROOT/v2f-bitstream/_fixtures/empty_hx1k.asc" \
    --output "$ROOT/_out/empty.bin" --backend rust
echo "  ✓ empty.bin ($(wc -c < "$ROOT/_out/empty.bin") bytes)"

echo ""
echo "=== 輸出檔案 ==="
ls -la "$ROOT/_out/"

echo ""
echo "=== 完成 ==="
echo "下一步: 可用 iceprog/openFPGALoader 燒錄 .bin 至 FPGA"
