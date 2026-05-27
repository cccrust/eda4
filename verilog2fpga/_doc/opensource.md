* https://gemini.google.com/app/8fae5ca38b844b97

在近年「開放原始碼硬體」與 RISC-V 的浪潮推動下，IC 設計與 FPGA 的開源工具鏈（EDA Tools）已經非常成熟，完全可以做到從暫存器傳輸級（RTL）編寫、模擬、邏輯綜合，一路到佈線（Routing）並產生位元流（Bitstream）的完整流程。

要組成一套完整的 FPGA 開源工具鏈，通常會包含以下幾個核心階段的工具：

---

## 1. 前端設計與硬體描述語言（HDL）

除了傳統的 Verilog / VHDL，現在有更多高階建構語言（Chisel, SpinalHDL）可以編譯成 Verilog。

* **Chisel / SpinalHDL / Amaranth:** 基於 Scala 或 Python 的新一代硬體描述語言，大幅提升模組化與參數化的能力。
* **SystemVerilog / Verilog:** 仍是業界標準。

---

## 2. 模擬與驗證（Simulation & Verification）

在燒錄到 FPGA 之前，必須先透過軟體模擬驗證電路邏輯是否正確。

* **Verilator:** 目前最火紅的開源模擬器。它不走傳統的事件驅動模擬，而是把 Verilog/SystemVerilog 程式碼**轉換成 C++ 或 SystemC**，再編譯成執行檔。速度極快，非常適合跑大型 SoC 驗證（例如執行 Linux 的 RISC-V 核心）。
* **Icarus Verilog (iverilog):** 經典的 IEEE-1364 Verilog 模擬器，支援波形輸出，適合中小規模設計與教學。
* **GTKWave:** 搭配上述模擬器使用的**波形觀測軟體**，用來視覺化查看訊號線的變化。

---

## 3. 邏輯綜合（Synthesis）

將你寫的 HDL 程式碼，轉換成由網表（Netlist）組成的邏輯閘或 FPGA 的基本單元（如 LUT、Flip-Flop）。

* **Yosys:** 開源 EDA 界的核心支柱。它是一個功能強大的 RTL 綜合工具，支援 Verilog-2005，並且擁有強大的內建腳本功能，可以把邏輯電路對應（Mapping）到各種不同的 FPGA 架構或 ASIC 製程。

---

## 4. 佈局與佈線（Place and Route, P&R）

拿著 Yosys 產生的網表，決定這些邏輯單元要放在 FPGA 的哪個實體位置（Place），並把它們連線起來（Route）。

* **nextpnr:** 目前最主流、架構最現代化的開源 P&R 工具。它是硬體無關（Hardware-independent）的設計，透過不同的後端（Backends）來支援不同的 FPGA 晶片家族（如 Lattice iCE40, ECP5, Xilinx Series 7 等）。
* **vtr (Verilog-to-Routing):** 一個偏向學術研究與大規模 FPGA 架構探索的強大 P&R 工具鏈。

---

## 5. 位元流打包與燒錄（Bitstream & Programming）

將 P&R 完的結果轉換成 FPGA 看得懂的二進位檔案（Bitstream），並燒錄進去。這部分通常依賴**逆向工程項目**來解析晶片廠的私有格式。

* **Project IceStorm:** 針對 Lattice iCE40 FPGA 的逆向工程工具鏈（包含 `icepack` / `iceprog`）。
* **Project Trellis:** 針對 Lattice ECP5 FPGA 的工具鏈。
* **Project X-Ray:** 針對 Xilinx Series 7 (如 Artix-7, Kintex-7) 的逆向工程專案，產出的資料庫可以讓 nextpnr 支援 Xilinx 晶片。
* **OpenFPGALoader:** 一個非常強大且通用的開源 FPGA 燒錄工具，支援各種常見的 JTAG 下載器與各家 FPGA 晶片。

---

## 經典組合：目前最常用的兩套開源工作流

如果你想直接上手，目前社群最推薦以下兩套整合方案：

### 方案 A：FOSS 100% 全開源工作流（以 Lattice 晶片為主）

這是目前自動化最完整、體驗最流暢的組合（通常被稱為 **SymbiFlow** 或 **CHIPS Alliance F4PGA** 的一部分）：


$$\text{Verilog 原始碼} \xrightarrow{\text{Yosys (綜合)}} \text{Netlist} \xrightarrow{\text{nextpnr (佈線)}} \text{Ascii 配置} \xrightarrow{\text{icepack (打包)}} \text{Bitstream}$$

### 方案 B：高階驗證工作流

如果你在開發大型專案（例如 RISC-V 處理器），通常會用這個組合進行極速模擬：


$$\text{SystemVerilog} \xrightarrow{\text{Verilator}} \text{C++ 程式碼} \xrightarrow{\text{GCC/Clang 編譯}} \text{模擬執行檔} \xrightarrow{\text{GTKWave 觀測}}$$

---

> **補充：** 這些工具不僅能做 FPGA，如果你把後端的 nextpnr 換成 **OpenLane / OpenROAD** 工具鏈，它甚至可以一路做到 **ASIC（晶片下線晶圓生產）** 的 GDSII 格式，直接送去台積電或 SkyWater 投片。

您目前是有特定的 FPGA 晶片（例如 Xilinx 或 Lattice）想要試著用開源工具鏈開發，還是正準備評估一套給團隊或教學使用的 IC 設計流程呢？