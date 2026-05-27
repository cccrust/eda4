在 iMac (macOS Intel) 上最簡單的方式是用 Homebrew：
brew install yosys icestorm openfpgaloader
nextpnr 不在 homebrew core 中，可用社群 tap 安裝：
brew tap siliconwitchery/oss-fpga
brew install --HEAD siliconwitchery/oss-fpga/nextpnr-ice40
裝完後用 v2f check（v0.1 的指令）確認各工具都在 PATH 上即可。
如果想一次到位（含 Verilator / GTKWave 等模擬工具）：
brew install yosys icestorm openfpgaloader icarus-verilog verilator gtkwave