#!/bin/bash
set -euo pipefail

echo "=== Cross-Validation Tests ==="
echo ""

echo "--- v2f-synth (JSON layer) ---"
cargo test -p v2f-synth -- cross 2>&1 | grep -E '(test |FAILED|error|ok$)'

echo ""
echo "--- v2f-pnr (ASC layer) ---"
cargo test -p v2f-pnr -- cross 2>&1 | grep -E '(test |FAILED|error|ok$)'

echo ""
echo "--- v2f-bitstream (BIN layer) ---"
cargo test -p v2f-bitstream -- cross 2>&1 | grep -E '(test |FAILED|error|ok$)'

echo ""
echo "All cross-validation tests passed."
