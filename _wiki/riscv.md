# RISC-V 開放指令集架構

## 什麼是 RISC-V

RISC-V 是一個開放、自由的指令集架構（ISA, Instruction Set Architecture），於 2010 年由 UC Berkeley 的 Krste Asanović 教授帶領的團隊所開發。與 ARM 或 x86 不同，RISC-V 的指令集規範完全開放，不受任何專利或授權費用的限制。任何人都可以免費使用 RISC-V 規範來設計處理器、編譯器、作業系統或其他軟硬體工具。

RISC-V 的名稱由來：RISC 代表精簡指令集計算（Reduced Instruction Set Computer），V 是羅馬數字 5，代表這是 UC Berkeley 的第五代 RISC 設計（前四代分別是 RISC-I、RISC-II、SOAR、SPUR）。第五代 RISC 的設計目標是成為一個工業級、開放、可擴展的 ISA。

RISC-V 國際協會（RISC-V International）成立於 2015 年，是負責維護與發展 RISC-V 規範的非營利組織。總部位於瑞士，會員涵蓋全球數百家公司與學術機構，包括 Google、Qualcomm、NVIDIA、Intel、Western Digital 等。

RISC-V 的規範由多個卷冊組成：
- 卷冊 I：使用者級 ISA（RV32I、RV64I、RV128I 以及各種標準擴展）
- 卷冊 II：特權級 ISA（機器模式、監督者模式、使用者模式）
- 卷冊 III：除錯與追蹤
- 卷冊 IV：向量擴展

## RISC-V 對開源硬體的重要性

在 RISC-V 出現之前，開源硬體生態面臨一個根本性的困境：缺乏真正開放且實用的 ISA。ARM 架構需要昂貴的授權費，x86 由 Intel 與 AMD 壟斷，其他開放架構（如 OpenRISC、SPARC）則缺乏生態系統支援。

RISC-V 解決了這個問題：

- **無授權費用**：RISC-V 規範採用 BSD 授權，任何人都可以免費用於商業或非商業用途
- **無專利障礙**：RISC-V 的設計確保不侵犯現有的 ISA 專利
- **可擴展性**：提供標準擴展機制，允許使用者定義自訂指令
- **生態系統完整**：GCC、LLVM、Linux、OpenBSD 等主流軟體都已支援 RISC-V
- **學術友善**：適合做為計算機架構教學與研究的平台

RISC-V 常被比喻為 Linux 在作業系統領域的角色。Linux 提供了一個自由開放的作業系統核心，催生了龐大的生態系統；RISC-V 試圖在硬體領域複製相同的成功模式。

## RISC-V 設計原則

RISC-V 的設計繼承了 Berkeley RISC-I 與 RISC-II（1980 年代）的精髓，並結合了過去三十年計算機架構研究的經驗教訓。

### 簡潔性（Simplicity）

RISC-V 的指令格式具有高度規律性。例如，RV32I 的所有指令都是固定的 32 位元寬度，這簡化了解碼器的硬體實作。相比之下，x86 的指令長度從 1 到 15 位元組不等，解碼器極其複雜。

### 模組化（Modularity）

RISC-V 將 ISA 設計為一個精簡的基礎整數指令集（RV32I 只有 47 條指令），加上多個可選的標準擴展。這種設計讓硬體實作者可以根據應用需求選擇所需的擴展，避免為不需要的功能支付硬體代價。

### 可擴展性（Extensibility）

RISC-V 預留了大量的編碼空間給自訂擴展。使用者可以定義自己的客製化指令（custom-1/custom-2/custom-3），實現領域專用的硬體加速。

### 乾淨的設計（Clean-slate Design）

RISC-V 不背負歷史包袱。它不要求向後相容於早期的處理器架構，因此可以採用最現代的設計方案。

## 基礎整數指令集

### RV32I

RV32I 是 RISC-V 的基礎整數指令集，使用 32 位元的指令寬度與 32 位元的資料路徑。它包含 47 條指令，分為以下類別：

**算術運算指令**：ADD、SUB、ADDI、SLT、SLTI、SLTU、SLTIU、LUI、AUIPC

**邏輯運算指令**：XOR、OR、AND、XORI、ORI、ANDI

**移位指令**：SLL、SRL、SRA、SLLI、SRLI、SRAI

**記憶體存取指令**：LB、LH、LW、LBU、LHU、SB、SH、SW

**分支指令**：BEQ、BNE、BLT、BGE、BLTU、BGEU

**跳躍指令**：JAL、JALR

**同步與環境指令**：FENCE、FENCE.I、ECALL、EBREAK

**CSR 指令**：CSRRW、CSRRS、CSRRC、CSRRWI、CSRRSI、CSRRCI

### RV64I

RV64I 是 RV32I 的 64 位元擴充版本。它將通用暫存器從 32 位元擴展到 64 位元，並增加了 64 位元的算術與記憶體操作指令（如 ADDW、SUBW、SLLW、SRLW、SRAW、LWU、LD、SD 等）。

### RV128I

RV128I 是實驗性的 128 位元基礎整數指令集，主要用於未來的高效能運算與大記憶體位址空間場景。目前規範尚未完全穩定。

## 標準擴展

### M 擴展（整數乘除法）

M 擴展為 RV32I 與 RV64I 增加了整數乘法與除法指令：
- MUL、MULH、MULHSU、MULHU：各種形式的乘法（低 64 位元、高位元等）
- DIV、DIVU、REM、REMU：除法與餘數運算

### F 擴展（單精度浮點數）

F 擴展增加了 IEEE 754-2008 相容的單精度浮點運算：
- 32 個浮點暫存器 f0-f31
- 浮點加減乘除（FADD、FSUB、FMUL、FDIV）
- 浮點比較（FEQ、FLT、FLE）
- 整數與浮點間的轉換（FCVT）
- 浮點平方根（FSQRT）

### D 擴展（雙精度浮點數）

D 擴展在 F 擴展的基礎上增加了雙精度浮點運算。F 與 D 擴展通常合稱為 FD 擴展。

### C 擴展（壓縮指令）

C 擴展將常見的指令編碼為 16 位元格式，以減少程式碼大小。實作 C 擴展的處理器可以混合執行 16 位元與 32 位元的指令。

壓縮指令節省的程式碼空間通常在 25-30%。C 擴展在嵌入式系統與 IoT 裝置中特別重要。

### A 擴展（原子操作）

A 擴展提供了不可分割的讀改寫（read-modify-write）操作，對於多核心處理器的同步機制至關重要：
- LR（Load Reserved）與 SC（Store Conditional）
- AMO（Atomic Memory Operation）指令：AMOSWAP、AMOADD、AMOAND 等

### V 擴展（向量運算）

V 擴展是一個可變長度的向量處理架構，支援從 128 位元到 65536 位元的向量暫存器寬度。向量擴展的設計目標是為資料平行運算提供高效能的程式設計模型。

V 擴展包含了：
- 向量算術與邏輯運算
- 記憶體 gather/scatter 操作
- 向量遮罩（masking）支援
- 向量長度可配置

## 指令格式

RISC-V 的指令格式是 RISC-V 架構的重要特色。所有 RV32I 指令都是 32 位元固定長度，並以規律的方式編碼。共有六種基本格式：

### R-type（暫存器型）

用於雙暫存器運算（如 ADD、SUB、XOR）。
```
funct7[31:25] | rs2[24:20] | rs1[19:15] | funct3[14:12] | rd[11:7] | opcode[6:0]
```
三個暫存器運算元（rs1、rs2、rd），加上 funct7 與 funct3 區分運算類型。

### I-type（立即值型）

用於帶立即值的運算（如 ADDI、LW）與 jalr。
```
imm[31:20] | rs1[19:15] | funct3[14:12] | rd[11:7] | opcode[6:0]
```
一個來源暫存器（rs1）、一個目標暫存器（rd）、和一個 12 位元帶正負號的立即值。

### S-type（儲存型）

用於儲存指令（如 SW、SH、SB）。
```
imm[11:5] | rs2[24:20] | rs1[19:15] | funct3[14:12] | imm[4:0] | opcode[6:0]
```
兩個來源暫存器（rs1=基址、rs2=資料），立即值被分割為兩個欄位編碼。

### B-type（分支型）

用於條件分支指令（如 BEQ、BNE）。
```
imm[12|10:5] | rs2[24:20] | rs1[19:15] | funct3[14:12] | imm[4:1|11] | opcode[6:0]
```
與 S-type 類似，但立即值編碼為 13 位元（除以 2 的位移量），且不包含 rd。

### U-type（上立即值型）

用於 LUI 與 AUIPC。
```
imm[31:12] | rd[11:7] | opcode[6:0]
```
提供 20 位元的上半部立即值，與 rd 合併產生 32 位元結果。

### J-type（跳躍型）

用於 JAL 指令。
```
imm[20|10:1|11|19:12] | rd[11:7] | opcode[6:0]
```
21 位元帶正負號的跳躍位移（除以 2），可達 ±1MB 的跳躍範圍。

指令格式的規律性大大簡化了 RISC-V 處理器的解碼器硬體設計。所有格式的 opcode 與 rs1/rd 都在相同的位置，解碼器可以平行地解碼所有欄位。

## 暫存器組

RISC-V 定義了 32 個通用暫存器（x0-x31），每個暫存器在 RV32I 中是 32 位元寬：

| 暫存器 | ABI 名稱 | 描述 | 是否保留 |
|--------|----------|------|----------|
| x0 | zero | 恆定為 0 | 硬線接地 |
| x1 | ra | 回傳位址 | 呼叫者保留 |
| x2 | sp | 堆疊指標 | 被呼叫者保留 |
| x3 | gp | 全域指標 | - |
| x4 | tp | 執行緒指標 | - |
| x5-x7 | t0-t2 | 暫時暫存器 | 呼叫者保留 |
| x8 | s0/fp | 儲存暫存器/框架指標 | 被呼叫者保留 |
| x9 | s1 | 儲存暫存器 | 被呼叫者保留 |
| x10-x11 | a0-a1 | 函數參數/回傳值 | 呼叫者保留 |
| x12-x17 | a2-a7 | 函數參數 | 呼叫者保留 |
| x18-x27 | s2-s11 | 儲存暫存器 | 被呼叫者保留 |
| x28-x31 | t3-t6 | 暫時暫存器 | 呼叫者保留 |

x0（zero）是 RISC-V 的一個獨特設計。它被硬體接地，讀取永遠為 0，寫入被忽略。這簡化了許多常見的編碼模式（如移動與比較）。

ABI 名稱是為了組合語言程式設計與編譯器後端設計的便利。它們的約定遵循 RISC-V ABI 規範。

## 特權等級

RISC-V 定義了三種特權等級，從最高到最低：

### 機器模式（Machine Mode, M-mode）

最高特權等級，必須實作。M-mode 可以存取所有 CSR 與硬體資源。處理器啟動時處於 M-mode。在 M-mode 下執行的程式通常是最底層的韌體（如 Berkeley Boot Loader, BBL）或監督者二進制介面（SBI）。

M-mode 使用一組獨立的 CSR 來控制中斷、例外與實體記憶體保護（PMP, Physical Memory Protection）。

### 監督者模式（Supervisor Mode, S-mode）

中等特權等級，用於執行作業系統核心。S-mode 支援虛擬記憶體管理，透過專用 CSR 控制頁表查找。Linux 等現代作業系統在 S-mode 下運作。

S-mode 的中斷與例外處理委託機制（delegation）允許 M-mode 將部分中斷與例外轉交給 S-mode 處理。

### 使用者模式（User Mode, U-mode）

最低特權等級，用於執行應用程式。U-mode 無法存取敏感的 CSR 與硬體資源。試圖在 U-mode 下執行特權操作會觸發例外。

部分嵌入式 RISC-V 處理器可能只實作 M-mode，省略 S-mode 與 U-mode 以節省硬體面積。

## TinyRV2 脈絡

TinyRV2 是 Cornell 大學 ECE 5745 計算機架構課程中使用的一個精簡化 RV32I 子集。它移除了 RV32I 中的部分指令（如 CSR 操作、FENCE、AUIPC），保留了最核心的算術、邏輯、記憶體與分支指令。

TinyRV2 的設計目標是教學：既保留了 RISC-V 的精華，又降低了學生實作處理器的難度。在 Cornell 的課程中，學生從 TinyRV2 開始逐步擴展到完整的 RV32I。

TinyRV2 的經驗也可以應用到 eda4 的 Hack CPU 專案中：如果未來要實作一個 RISC-V 處理器，可以從 TinyRV2 做為起點，先掌握最基本的 datapath 與控制單元，再逐步添加更多指令與功能。

## RISC-V 與 Hack CPU 的比較

eda4 的 Hack CPU 來自 nand2tetris 課程，是一個簡化的 16 位元架構。Hack CPU 與 RISC-V 有許多概念上的相似性，但也有重要的差異：

### 相似點
- **指令暫存器架構（Register-based）**：兩者都使用通用暫存器作為運算元
- **程式計數器（Program Counter）**：都有 PC 來追蹤目前指令位址
- **ALU 為核心**：算術與邏輯運算由 ALU 執行
- **條件分支**：依據前次 ALU 結果進行條件跳躍

### 差異點
- **位元寬度**：Hack CPU 是 16 位元，RISC-V RV32I 是 32 位元
- **指令格式**：Hack CPU 使用 A-instruction 與 C-instruction 兩種格式，遠比 RISC-V 的六種格式簡單
- **暫存器數量**：Hack CPU 有 D 與 A 兩個可程式暫存器（加上 M 記憶體暫存器），而 RISC-V 有 32 個
- **記憶體架構**：Hack CPU 使用統一位址空間（指令與資料共用），RISC-V 使用獨立的哈佛架構或馮紐曼架構
- **ISA 複雜度**：Hack CPU 的 C-instruction 將 ALU 操作、目的暫存器與跳躍條件編碼在同一條指令中，類似於 VLIW；RISC-V 則是標準的 RISC 設計

學習 Hack CPU 有助於理解處理器設計的基本概念（datapath、control、fetch-decode-execute cycle），這些知識可以直接應用到 RISC-V 處理器的設計與實作。

## 可用的 RISC-V 核心

以下是在開源社群中廣泛使用的 RISC-V 核心，都可以用 Verilog 取得原始碼。

### PicoRV32

由 Clifford Wolf（Yosys 的作者）開發的尺寸最佳化 RV32IMC 核心。PicoRV32 使用非常少的 LUT，可以在低成本的 FPGA 上運行。它以 Verilog 原始碼提供，適合與 eda4 的合成工具鏈整合。

### VexRiscv

用 SpinalHDL 實作的 RV32IMC 核心，支援多種配置選項。VexRiscv 是 Caravel SoC 使用的處理器核心，廣泛應用在 Open MPW 設計中。

### SERV

SERV 是一個位元序列化（bit-serial）的 RV32I 核心，以最簡化的方式實作 RISC-V 指令集。它的面積極小，適合教育與演示用途。

### Rocket

由 UC Berkeley 開發的 64 位元高效能核心，使用 Chisel 硬體建構語言設計。Rocket 是 Chipyard SoC 框架的核心處理器之一。

### BOOM（Berkeley Out-of-Order Machine）

同樣由 UC Berkeley 開發，BOOM 是一個亂序執行（out-of-order）的 RISC-V 核心，目標是高效能運算。BOOM 也使用 Chisel 設計。

這些核心都可以透過 eda4 的 verilog2fpga 流程合成到 iCE40 FPGA，或透過 OpenLANE 流程實現為 ASIC。

## ISA 與 HDL 的關係

指令集架構（ISA）與硬體描述語言（HDL）有本質上的區別：

**ISA** 是程式設計師與編譯器看到的處理器介面，定義了指令編碼、暫存器組、記憶體模型等。ISA 是規範，不涉及實作細節。

**HDL**（如 Verilog、VHDL、SystemVerilog）是用來描述硬體行為與結構的語言。處理器的 HDL 實作是將 ISA 規範轉換為實際的邏輯閘電路。

舉例來說，RV32I 的 ADD 指令規範定義了它的 opcode、funct3、funct7 與行為（rd = rs1 + rs2）。在 Verilog 實作中，這條指令對應到一個多工器的選擇路徑，最終被合成為加法器電路。

EDA 工具（如 Yosys、OpenLANE、以及 eda4 的合成工具）負責將 HDL 描述轉換為邏輯閘網路，最終實現為 FPGA 位元流或 ASIC 遮罩。

## RISC-V 驗證

### riscv-tests

riscv-tests 是 RISC-V 國際協會維護的官方測試套件，包含針對各個擴展的指令級測試。每個測試都是一個小的組合語言程式，執行後檢查結果是否正確。

測試覆蓋範圍包括：
- RV32I/RV64I 的所有指令
- M、F、D、C、A 擴展的指令
- 特權模式的切換
- CSR 讀寫

### riscv-dv

riscv-dv 是 Google 開發的指令產生器，可以隨機生成 RISC-V 指令序列進行壓力測試。它支援 RV32IMCFD 擴展，並可以注入隨機中斷與例外。

riscv-dv 使用 SystemVerilog UVM 框架，可以與多種模擬器整合。

### 形式驗證

RISC-V 核心的形式驗證使用符號執行與模型檢查技術。SymbiYosys（sby）可以驗證 RISC-V 核心是否正確實現了指令的規範行為。形式驗證的優點在於它可以窮盡檢查所有可能的執行路徑。

## 開源 RISC-V 工具

### GCC

GCC 對 RISC-V 的支援始於 2017 年（GCC 7），目前支援 RV32I/RV64I 以及所有標準擴展。RISC-V GCC 工具鏈包含編譯器、組合語言器、鏈結器與標準庫。

### LLVM/Clang

LLVM 在 2018 年開始對 RISC-V 提供穩定支援。LLVM 的模組化設計讓它更容易整合到不同的開發環境中。

### Spike

Spike 是 RISC-V 的官方 ISA 模擬器，由 UC Berkeley 開發。它是一個 functional 模擬器，不涉及時間與流水線行為。Spike 常做為開發與除錯工具。

### QEMU

QEMU 支援多種 RISC-V 開發板與 SoC 模型，可以啟動 Linux 系統進行完整軟體開發與測試。

### Verilator

Verilator 是高效能的 Verilog/SystemVerilog 模擬器，可以將 HDL 編譯為 C++ 進行高速模擬。Verilator 廣泛用於 RISC-V 核心的開發與驗證流程中。
