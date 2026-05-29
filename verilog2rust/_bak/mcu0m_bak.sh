# 自動化測試 (parse → gen → rustc → run)
# cargo test test_sim_mcu0m -- --test-threads=1
# 手動產出 rHDL 並執行
cargo run -- verilog/mcu0/mcu0m.v
rustc --extern verilog2rust=target/debug/libverilog2rust.rlib \
      -L target/debug -L target/debug/deps \
      mcu0m.rs -o /tmp/mcu0m_test --edition 2021
/tmp/mcu0m_test
# 手寫對照組 (驗證 1+...+10 = 55)
# cargo run --example basic