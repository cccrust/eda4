cargo run -- verilog/mcu0/mcu0m.v
# cp verilog/mcu0/mcu0m.hex verilog/mcu0/
rustc --extern verilog2rust=target/debug/libverilog2rust.rlib \
      -L target/debug -L target/debug/deps \
      verilog/mcu0/mcu0m.rs -o /tmp/test_mcu0m --edition 2021
cd verilog/mcu0/
/tmp/test_mcu0m