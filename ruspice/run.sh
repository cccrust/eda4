#!/bin/bash
# ruspice 執行腳本

echo "=========================================="
echo "ruspice v0.1.0 - Analog Circuit Simulator"
echo "=========================================="
echo

case "$1" in
  test)
    cargo test
    ;;
  run)
    cargo run
    ;;
  example|examples)
    cargo run --example basic
    ;;
  circuit)
    cargo run -- circuit
    ;;
  plot-dc)
    cargo run -- plot-dc
    ;;
  plot-transient)
    cargo run -- plot-transient vout
    ;;
  svg)
    cargo run -- svg circuit.svg && echo "已產生 circuit.svg" && ls -la circuit.svg
    ;;
  plot)
    cargo run -- save-plot transient.svg vout && echo "已產生 transient.svg" && ls -la transient.svg
    ;;
  all)
    echo "=== 1. 執行測試 ==="
    cargo test
    echo
    echo "=== 2. 電路圖 (ASCII) ==="
    cargo run -- circuit
    echo
    echo "=== 3. DC 電壓分布 ==="
    cargo run -- plot-dc
    echo
    echo "=== 4. 瞬態響應 (ASCII) ==="
    cargo run -- plot-transient vout
    echo
    echo "=== 5. 電路圖 (SVG) ==="
    cargo run -- svg circuit.svg
    echo "已產生 circuit.svg"
    echo
    echo "=== 6. 瞬態圖 (SVG) ==="
    cargo run -- save-plot transient.svg vout
    echo "已產生 transient.svg"
    echo
    echo "可用瀏覽器開啟 SVG 檔案查看圖形"
    ;;
  help|--help|-h)
    echo "用法: ./run.sh [指令]"
    echo
    echo "指令:"
    echo "  test      - 執行所有測試"
    echo "  run       - 執行 CLI 演示"
    echo "  example   - 執行基本範例"
    echo "  circuit   - 顯示電路圖 (ASCII)"
    echo "  plot-dc   - 繪製 DC 電壓分布"
    echo "  plot-transient - 繪製瞬態響應"
    echo "  svg       - 匯出電路圖為 SVG"
    echo "  plot      - 匯出瞬態圖為 SVG"
    echo "  all       - 執行全部功能"
    echo "  help      - 顯示此幫助"
    ;;
  *)
    echo "執行全部功能..."
    echo
    echo "=== 1. 執行測試 ==="
    cargo test
    echo
    echo "=== 2. 電路圖 (ASCII) ==="
    cargo run -- circuit
    echo
    echo "=== 3. DC 電壓分布 ==="
    cargo run -- plot-dc
    echo
    echo "=== 4. 瞬態響應 (ASCII) ==="
    cargo run -- plot-transient vout
    echo
    echo "=== 5. 電路圖 (SVG) ==="
    cargo run -- svg circuit.svg
    echo "已產生 circuit.svg"
    echo
    echo "=== 6. 瞬態圖 (SVG) ==="
    cargo run -- save-plot transient.svg vout
    echo "已產生 transient.svg"
    echo
    echo "完成！可用瀏覽器開啟 SVG 檔案查看圖形"
    ;;
esac