# JTAG 與 SPI FPGA 程式化協定

## JTAG 概述

JTAG（Joint Test Action Group，IEEE 1149.1 標準）最初是為了 PCB 邊界掃描測試（boundary scan testing）而設計的，後來被廣泛用於 FPGA 的配置程式化。JTAG 提供了一種標準化的方式來存取晶片內部的測試邏輯，包括邊界掃描暫存器（boundary scan register）、指令暫存器（instruction register）以及各種資料暫存器（data register）。

## JTAG 訊號

JTAG 介面使用四條強制訊號線與一條可選訊號線：

| 訊號 | 名稱 | 方向 | 說明 |
|------|------|------|------|
| TDI | Test Data In | 輸入 | 測試資料輸入，在 TCK 上升沿取樣 |
| TDO | Test Data Out | 輸出 | 測試資料輸出，在 TCK 下降沿更新 |
| TMS | Test Mode Select | 輸入 | 測試模式選擇，控制 TAP 狀態機轉換 |
| TCK | Test Clock | 輸入 | 測試時脈 |
| TRST | Test Reset（可選） | 輸入 | 非同步重置 TAP 狀態機 |

## TAP 狀態機

TAP (Test Access Port) 狀態機是一個 16 狀態的有限狀態機（Finite State Machine, FSM），所有狀態轉換由 TCK 上升沿時的 TMS 訊號值控制。

### 狀態轉換圖

TAP 的 16 個狀態定義於 `v2f-programmer/src/jtag.rs:7-24`：

```
TestLogicReset ──→ RunTestIdle ──→ SelectDRScan ──→ CaptureDR ──→ ShiftDR ──→ Exit1DR ──→ UpdateDR
                      │                │               │              │            │            │
                      │                │               │              │            │            │
                      │                ↓               ↓              ↓            ↓            ↓
                      │           SelectIRScan    CaptureIR      ShiftIR      Exit1IR      UpdateIR
                      │                │               │              │            │            │
                      ↓                ↓               ↓              ↓            ↓            ↓
                   (回到 RunTestIdle 或繼續轉換)    PauseDR       PauseIR        Exit2DR      Exit2IR
```

更具體地說，TAP 狀態機的每個狀態轉換定義如下（`jtag.rs:27-46`）：

```rust
impl TapState {
    pub fn transition(self, tms: bool) -> Self {
        match self {
            TestLogicReset  => if tms { TestLogicReset } else { RunTestIdle },
            RunTestIdle     => if tms { SelectDRScan }   else { RunTestIdle },
            SelectDRScan    => if tms { SelectIRScan }   else { CaptureDR },
            CaptureDR       => if tms { Exit1DR }        else { ShiftDR },
            ShiftDR         => if tms { Exit1DR }        else { ShiftDR },
            Exit1DR         => if tms { UpdateDR }       else { PauseDR },
            PauseDR         => if tms { Exit2DR }        else { PauseDR },
            Exit2DR         => if tms { UpdateDR }       else { ShiftDR },
            UpdateDR        => if tms { SelectDRScan }   else { RunTestIdle },
            SelectIRScan    => if tms { TestLogicReset } else { CaptureIR },
            CaptureIR       => if tms { Exit1IR }        else { ShiftIR },
            ShiftIR         => if tms { Exit1IR }        else { ShiftIR },
            Exit1IR         => if tms { UpdateIR }       else { PauseIR },
            PauseIR         => if tms { Exit2IR }        else { PauseIR },
            Exit2IR         => if tms { UpdateIR }       else { ShiftIR },
            UpdateIR        => if tms { SelectDRScan }   else { RunTestIdle },
        }
    }
}
```

關鍵規則：
- **TMS=1** 通常走向「右側」或「重置」方向
- **TMS=0** 通常保持當前狀態或走向「左側」方向
- 在 `ShiftDR` 與 `ShiftIR` 狀態中，TMS=0 維持 shifting，TMS=1 跳出

### 狀態說明

| 狀態 | 說明 |
|------|------|
| TestLogicReset | 重置狀態，TMS 保持 1 至少 5 個 TCK 週期可確保進入此狀態 |
| RunTestIdle | 空轉狀態，TAP 空轉時停留於此 |
| SelectDRScan | 選擇 DR 掃描路徑的過渡狀態 |
| CaptureDR | 將資料載入 DR 暫存器 |
| ShiftDR | 逐位元移位 DR，TDI 輸入、TDO 輸出 |
| Exit1DR | 結束 DR 移位的過渡狀態 |
| PauseDR | 暫停 DR 移位 |
| Exit2DR | 從暫停恢復的過渡狀態 |
| UpdateDR | 將移位後的資料更新到並行輸出暫存器 |
| SelectIRScan | 選擇 IR 掃描路徑的過渡狀態 |
| CaptureIR | 將資料載入 IR 暫存器 |
| ShiftIR | 逐位元移位 IR |
| Exit1IR | 結束 IR 移位的過渡狀態 |
| PauseIR | 暫停 IR 移位 |
| Exit2IR | 從暫停恢復的過渡狀態 |
| UpdateIR | 將移位後的指令更新到並行輸出暫存器 |

## v2f JTAG 實作

### JtagStateMachine

JTAG 狀態機的 Rust 實作位於 `v2f-programmer/src/jtag.rs:53-59`：

```rust
pub struct JtagStateMachine {
    pub state: TapState,
    pub tms_buffer: Vec<bool>,
    pub tdi_buffer: Vec<u8>,
    pub tdo_buffer: Vec<u8>,
}
```

初始狀態為 `TestLogicReset`（第 61-70 行）。

### advance() 方法

每次 TCK 時脈的狀態推進（第 77-83 行）：

```rust
pub fn advance(&mut self, tms: bool, tdi: bool) -> bool {
    self.state = self.state.transition(tms);
    self.tms_buffer.push(tms);
    self.tdi_buffer.push(if tdi { 1 } else { 0 });
    self.tdo_buffer.push(0);
    false
}
```

### shortest_path() BFS

`go_to()` 方法使用廣度優先搜尋（BFS）計算從當前狀態到目標狀態的最短路徑（第 85-90 行）：

```rust
pub fn go_to(&mut self, target: TapState) {
    let path = shortest_path(self.state, target);
    for tms in path {
        self.advance(tms, false);
    }
}
```

`shortest_path()` 函數（第 119-137 行）實作了 BFS 搜尋：

```rust
fn shortest_path(from: TapState, to: TapState) -> Vec<bool> {
    // BFS from 'from' to 'to'
    // 每次分支為 TMS=true 與 TMS=false
    // 限制深度最多 10
}
```

### is_shift() 方法

判斷當前是否處於移位狀態（第 48-50 行）：

```rust
pub fn is_shift(&self) -> bool {
    matches!(self, TapState::ShiftDR | TapState::ShiftIR)
}
```

## JTAG 指令

標準 JTAG 指令定義（`v2f-programmer/src/jtag.rs` 中並未直接定義指令常數，而是在 `ice40_prog.rs` 中定義 iCE40 專用指令）：

### iCE40 JTAG 指令

`Ice40JtagInstruction` 枚舉定義於 `v2f-programmer/src/ice40_prog.rs:7-17`：

```rust
pub enum Ice40JtagInstruction {
    Extest  = 0x00,   // 外部測試
    Sample  = 0x01,   // 取樣
    UsrCode = 0x02,   // 使用者代碼
    Usr0    = 0x03,   // 使用者指令 0
    Usr1    = 0x0B,   // 使用者指令 1（用於 CRAM 程式化）
    Highz   = 0x07,   // 高阻抗
    Clamp   = 0x0A,   // 鉗位
    Intest  = 0x0C,   // 內部測試
    Bypass  = 0x3F,   // 繞過（1 bit 暫存器）
}
```

其中 `Usr1`（0x0B）是 iCE40 中用於 CRAM 程式化的關鍵指令。指令長度除了 `Bypass` 為 1 bit 外，其餘皆為 5 bits（第 21-27 行）：

```rust
pub fn len(&self) -> u8 {
    match self {
        Ice40JtagInstruction::Bypass => 1,
        _ => 5,
    }
}
```

## ShiftDR 與 ShiftIR

### ShiftIR

`shift_ir()` 方法（`jtag.rs:108-116`）將 JTAG 指令移入指令暫存器：

```rust
pub fn shift_ir(&mut self, instruction: u8, len: u8) {
    self.go_to(TapState::ShiftIR);
    for bit in 0..len {
        let is_last = bit == len - 1;
        // 在最後一個 bit 將 TMS 設為 1 以跳出 ShiftIR
        self.state = self.state.transition(is_last);
    }
    self.go_to(TapState::RunTestIdle);
}
```

### ShiftDR

`shift_dr()` 方法（`jtag.rs:92-106`）將資料移入資料暫存器：

```rust
pub fn shift_dr(&mut self, data: &[u8], _tdi_last: bool) -> Vec<u8> {
    self.go_to(TapState::ShiftDR);
    // 逐位元移位，最後一個位元時設定 TMS=1 以跳出 ShiftDR
    self.go_to(TapState::RunTestIdle);
    // 回傳 TDO 資料
}
```

## iCE40 JTAG 程式化流程

iCE40 晶片透過 JTAG 介面進行 CRAM 程式化的標準流程如下：

1. **進入 RunTestIdle**：從 TestLogicReset 轉移到 RunTestIdle
2. **進入 ShiftIR**：透過 SelectIRScan → CaptureIR → ShiftIR
3. **移入 USR1 指令**：將 0x0B（5 bits）移入指令暫存器
4. **進入 ShiftDR**：透過 SelectDRScan → CaptureDR → ShiftDR
5. **移入位元流**：移入 8 個 0x00 dummy bytes 後緊接 bitstream payload
6. **回到 RunTestIdle**：完成資料傳送

這個流程實作於 `Ice40Programmer::program_cram_jtag()`（`ice40_prog.rs:32-41`）：

```rust
pub fn program_cram_jtag(jtag: &mut JtagStateMachine, bitstream: &[u8]) -> Result<(), String> {
    jtag.go_to(TapState::RunTestIdle);
    jtag.shift_ir(Ice40JtagInstruction::Usr1.code(), Ice40JtagInstruction::Usr1.len());
    let mut framed = Vec::new();
    framed.extend_from_slice(&[0x00u8; 8]);  // 8 dummy bytes
    framed.extend_from_slice(bitstream);
    jtag.shift_dr(&framed, true);
    jtag.go_to(TapState::RunTestIdle);
    Ok(())
}
```

## SPI 協定

SPI（Serial Peripheral Interface）是一種同步序列通訊協定，使用四條訊號線：

| 訊號 | 名稱 | 說明 |
|------|------|------|
| SCK | Serial Clock | 由主裝置產生的時脈訊號 |
| MOSI | Master Out Slave In | 主裝置輸出、從裝置輸入 |
| MISO | Master In Slave Out | 從裝置輸出、主裝置輸入 |
| SS | Slave Select | 從裝置選擇（低電位有效） |

SPI 是全雙工協定：在每個 SCK 時脈週期，主裝置同時在 MOSI 上送出資料並在 MISO 上接收資料。

## SPI Flash 指令

標準 SPI flash 記憶體指令定義於 `v2f-programmer/src/spi.rs:6-22`：

```rust
pub enum SpiCommand {
    WriteEnable      = 0x06,  // 寫入啟用
    WriteDisable     = 0x04,  // 寫入停用
    ReadStatus       = 0x05,  // 讀取狀態暫存器
    WriteStatus      = 0x01,  // 寫入狀態暫存器
    ReadData         = 0x03,  // 讀取資料
    FastRead         = 0x0B,  // 快速讀取（含 dummy cycle）
    PageProgram      = 0x02,  // 頁面程式化
    SectorErase      = 0x20,  // 區段抹除（4KB）
    BlockErase32K    = 0x52,  // 區塊抹除（32KB）
    BlockErase64K    = 0xD8,  // 區塊抹除（64KB）
    ChipErase        = 0xC7,  // 晶片抹除
    DeepPowerDown    = 0xB9,  // 深度省電
    ReleasePowerDown = 0xAB,  // 解除省電
    Enter4ByteAddr   = 0xB7,  // 進入 4 位元組位址模式
    Exit4ByteAddr    = 0xE9,  // 退出 4 位元組位址模式
}
```

## SPI Flash 記憶體架構

SPI flash 的記憶體以層級方式組織：

| 單位 | 大小 | 說明 |
|------|------|------|
| Page | 256 bytes | 最小程式化單位 |
| Sector | 4 KB (4096 bytes) | 最小抹除單位 |
| Block (32K) | 32 KB | 較大抹除單位 |
| Block (64K) | 64 KB | 最大抹除單位 |

### SpiFlash 結構

`SpiFlash` 結構定義於 `spi.rs:28-34`：

```rust
pub struct SpiFlash {
    pub page_size: usize,     // 256
    pub sector_size: usize,   // 4096
    pub num_sectors: usize,
    pub memory: Vec<u8>,      // 初始化為全 0xFF
}
```

主要操作方法：

- **`erase_chip()`**：將全部記憶體設為 0xFF
- **`erase_sector(addr)`**：抹除 addr 所在的 sector（4KB 對齊）
- **`page_program(addr, data)`**：從 addr 開始寫入 data（可跨頁，但實際硬體限制單頁 256 bytes）
- **`read(addr, len)`**：從 addr 開始讀取 len 個位元組

## iCE40 從 SPI Flash 配置的流程

iCE40 FPGA 支援從外部 SPI flash 記憶體自動載入配置：

1. **上電**：晶片偵測到電源穩定
2. **SPI Master 模式**：iCE40 內建 SPI master，自動從 SPI flash 讀取位元流
3. **讀取位元流**：從 SPI flash 的起始位址（通常為 0x000000）開始讀取
4. **載入 CRAM**：將讀取的資料寫入內部 Configuration RAM
5. **啟動**：CRC 驗證通過後，晶片進入使用者模式

此流程透過 `Ice40Config::load_from_flash()`（`ice40_cfg.rs:56-58`）模擬：

```rust
pub fn load_from_flash(flash: &SpiFlash, addr: usize, len: usize) -> Result<Vec<u8>, String> {
    Ok(flash.read(addr, len))
}
```

## Ice40Config：SRAM 載入協定

當透過類 SPI（bitbanging）模式直接載入 CRAM 時，需遵循 iCE40 的 SRAM 載入協定。實作於 `Ice40Config::load_sram()`（`ice40_cfg.rs:29-53`）：

```
步驟 1: Reset CRAM     → 送出 4 個 0xFF（至少 8 個 clock 的 1）
步驟 2: 8 Dummy Bytes  → 送出 8 個 0x00
步驟 3: Bitstream      → 送出完整位元流 payload
步驟 4: CRC            → 送出 4 bytes CRC-32（僅計算 bitstream）
步驟 5: 48 Dummy Clocks → 送出 6 個 0x00（48 個 clock）
步驟 6: Wakeup         → 送出 1 個 0x00
```

```rust
pub fn load_sram(write_byte: &mut dyn FnMut(u8), bitstream: &[u8]) -> Result<(), String> {
    write_byte(0xFF); write_byte(0xFF); // Reset CRAM
    write_byte(0xFF); write_byte(0xFF);
    for _ in 0..8 { write_byte(0x00); } // 8 dummy bytes
    for &b in bitstream { write_byte(b); } // Bitstream payload
    let crc = crc32(bitstream);
    for &b in &crc { write_byte(b); } // CRC
    for _ in 0..6 { write_byte(0x00); } // 48 dummy clocks
    write_byte(0x00); // Wakeup
    Ok(())
}
```

CRC-32 使用 IEEE 802.3 多項式（`ice40_cfg.rs:13-23`）：

```rust
pub fn crc32(data: &[u8]) -> [u8; 4] {
    let mut crc = 0xFFFFFFFFu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 { crc = (crc >> 1) ^ 0xEDB88320; }
            else { crc >>= 1; }
        }
    }
    (crc ^ 0xFFFFFFFF).to_le_bytes()
}
```

## v2f-programmer crate

### Ice40Programmer

主要的程式化介面定義於 `ice40_prog.rs:29-57`：

```rust
pub struct Ice40Programmer;

impl Ice40Programmer {
    // 透過 JTAG 程式化 CRAM
    pub fn program_cram_jtag(jtag: &mut JtagStateMachine, bitstream: &[u8]) -> Result<(), String>;

    // 透過 SPI 寫入 Flash
    pub fn program_spi_flash(bitstream: &[u8], flash: &mut SpiFlash) -> Result<(), String>;

    // 驗證 Flash 內容是否與位元流一致
    pub fn verify_flash(bitstream: &[u8], flash: &SpiFlash) -> bool;
}
```

#### program_spi_flash()

將位元流以 page（256 bytes）為單位寫入 SPI flash（第 43-49 行）：

```rust
pub fn program_spi_flash(bitstream: &[u8], flash: &mut SpiFlash) -> Result<(), String> {
    let page_size = flash.page_size;
    for (offset, chunk) in bitstream.chunks(page_size).enumerate() {
        flash.page_program(offset * page_size, chunk);
    }
    Ok(())
}
```

#### verify_flash()

驗證寫入的正確性（第 51-57 行）：

```rust
pub fn verify_flash(bitstream: &[u8], flash: &SpiFlash) -> bool {
    for (offset, chunk) in bitstream.chunks(256).enumerate() {
        let readback = flash.read(offset * 256, chunk.len());
        if readback != chunk { return false; }
    }
    true
}
```

## Mock 與真實硬體

v2f-programmer 支援兩種 backend：

### MockFtdi（預設）

純 Rust 模擬實作，用於測試與開發，定義於 `ftdi.rs:39-80`：

```rust
pub struct MockFtdi {
    pub written: Vec<u8>,      // 寫入的資料緩衝
    pub read_data: Vec<u8>,    // 預設的讀取資料
    pub read_pos: usize,       // 讀取位置
    pub is_open: bool,         // 開啟狀態
}
```

`MockFtdi` 實作 `FtdiIo` trait（第 58-80 行），提供 `open()`、`write()`、`read()`、`close()` 方法。透過 `enqueue_read()` 可以模擬從裝置讀取的資料。

### RealFtdi（feature = "ftdi"）

真實 FTDI FT2232H 硬體驅動，透過 `libftd2xx-sys` crate 與 USB 裝置通訊（第 82-108 行）：

```rust
#[cfg(feature = "ftdi")]
pub mod real {
    pub struct RealFtdi {
        // handle: ftd2xx::FtHandle,
    }
    impl FtdiIo for RealFtdi { ... }
}
```

`RealFtdi` 實作目前為 stub 狀態（回傳 "尚未整合" 錯誤），需要整合 `libftd2xx-sys` 才能使用。

## FTDI FT2232H

FTDI FT2232H 是一顆 USB 轉 JTAG/SPI/UART/FIFO 的橋接晶片，廣泛用於 FPGA 開發板：

| 屬性 | 值 |
|------|-----|
| VID | 0x0403（FTDI） |
| PID | 0x6010（FT2232H） |
| 介面 A (Channel A) | 可配置為 JTAG 或 SPI Master |
| 介面 B (Channel B) | 可配置為 SPI Master 或 UART |
| 最大傳輸速率 | 30 MHz |

`FtdiConfig` 結構定義於 `ftdi.rs:13-19`：

```rust
pub struct FtdiConfig {
    pub vid: u16,           // 0x0403
    pub pid: u16,           // 0x6010
    pub interface: FtdiInterface,  // A 或 B
    pub baud_rate: u32,     // 30_000_000
}
```

### FtdiIo Trait

抽象的 FTDI IO trait（`ftdi.rs:32-37`）：

```rust
pub trait FtdiIo {
    fn open(config: &FtdiConfig) -> Result<Self, String> where Self: Sized;
    fn write(&mut self, data: &[u8]) -> Result<(), String>;
    fn read(&mut self, len: usize) -> Result<Vec<u8>, String>;
    fn close(&mut self) -> Result<(), String>;
}
```

## 驅動支援偵測

`check_driver_support()` 函數（`lib.rs:11-16`）回傳當前可用的驅動程式：

```rust
pub fn check_driver_support() -> Vec<&'static str> {
    let drivers = vec!["mock (模擬)"];
    #[cfg(feature = "ftdi")]
    let drivers = { let mut d = drivers; d.push("ftdi (FT2232H)"); d };
    drivers
}
```

編譯時若啟用 `ftdi` feature，則會包含真實硬體驅動支援。

## 程式化流程總覽

完整的 iCE40 程式化流程可分為以下幾種模式：

### JTAG 模式（CRAM 直接載入）

```
上位機 (v2f) → FT2232H USB → JTAG (TDI/TDO/TMS/TCK) → iCE40 (USR1 → ShiftDR)
```

1. v2f 透過 FT2232H 的 JTAG 介面連接 iCE40
2. 將 TAP 狀態機移至 ShiftIR
3. 移入 USR1 指令（0x0B）
4. 移至 ShiftDR
5. 透過 DR chain 移入位元流（8 dummy + payload）
6. 回到 RunTestIdle，iCE40 自動載入 CRAM

### SPI Flash 模式（外部記憶體）

```
上位機 (v2f) → FT2232H USB → SPI (SCK/MOSI/MISO/SS) → SPI Flash → iCE40 (SPI Master)
```

1. v2f 透過 FT2232H 的 SPI 介面連接 SPI flash
2. 發送 Chip Erase（0xC7）或 Sector Erase（0x20）清除舊資料
3. 以 Page Program（0x02）逐頁寫入位元流
4. 驗證寫入內容（Verify Flash）
5. iCE40 上電後自動從 SPI flash 載入配置

### Mock 模式（測試用）

不使用真實硬體，所有操作在記憶體中模擬，適用於 CI/CD 與開發測試。

## 參考實作

- JTAG 狀態機：`v2f-programmer/src/jtag.rs`
- SPI flash 模型與指令：`v2f-programmer/src/spi.rs`
- FTDI 抽象層：`v2f-programmer/src/ftdi.rs`
- iCE40 程式化流程：`v2f-programmer/src/ice40_prog.rs`
- iCE40 SRAM 載入協定：`v2f-programmer/src/ice40_cfg.rs`
- Crate 公開介面：`v2f-programmer/src/lib.rs`

## 延伸閱讀

- [JTAG (Wikipedia)](https://en.wikipedia.org/wiki/JTAG)
- [Serial Peripheral Interface (Wikipedia)](https://en.wikipedia.org/wiki/Serial_Peripheral_Interface)
- [IEEE 1149.1 (Wikipedia)](https://en.wikipedia.org/wiki/IEEE_1149.1)
