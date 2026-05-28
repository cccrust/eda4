# CRAM、Frame 與 iCE40 位元流格式

## 什麼是 CRAM

CRAM（Configuration RAM，設定隨機存取記憶體）是 FPGA 晶片內部用以儲存設定資訊的 SRAM 陣列。在 Lattice iCE40 系列 FPGA 中，CRAM 決定了晶片上所有可程式化元件的行為：每個 LUT 的 16-bit 真值表、每個 DFF 的時脈極性與重置模式、每個繞線多工器的開關狀態、每個 I/O 引腳的驅動強度與 slew rate、以及 PLL 分頻倍率等參數，全部由 CRAM 中的位元控制。

CRAM 的組織方式直接對應到 FPGA 的二維拼塊網格（tile grid）。每一個拼塊（logic tile、I/O tile、BRAM tile、DSP tile）佔用特定數量的 Frame，每個 Frame 包含固定數量的 bits。在 eda4 專案中，CRAM 的結構與操作實作於 `v2f-bitstream/src/cram.rs`，以 `Cram` 結構表示：

```rust
pub struct Cram {
    pub device: Ice40Device,
    frames: Vec<Frame>,
}
```

此結構儲存了所有 Frame 的完整狀態，並提供 `to_bytes()` 方法將整個 CRAM 序列化為位元組向量供後續打包使用。建立空白 CRAM 時，`Cram::new(device)` 根據裝置參數計算 Frame 總數，初始化一個全零的 Frame 向量——全零 CRAM 代表一個未經設定的 FPGA，所有 LUT 輸出為 0、所有繞線處於斷開狀態。

CRAM 中的資訊量相當可觀。以 HX8K 為例，CRAM 總容量約 18.7 Mbits（約 2.28 MB），這超過了典型嵌入式 SRAM 的容量，但對於描述一顆擁有 7,680 個邏輯單元與複雜繞線網路的 FPGA 而言，這是必要的代價。值得注意的是，CRAM 中繞線位元佔據了總容量的大約 70-80%，LUT 真值表其次，而 DFF 配置與 I/O 設定所佔比例最小。

## CRAM 的揮發性

iCE40 採用 SRAM 為基礎的設定技術，這意味著 CRAM 是揮發性（volatile）記憶體——斷電後所有設定資訊立即遺失。因此，iCE40 必須在每次上電後從外部來源重新載入設定。典型的設定來源為外部 SPI Flash 記憶體（如 Winbond W25Qxx 系列），FPGA 透過 Master SPI 模式自動讀取 Flash 中的位元流，寫入 CRAM 完成配置。

揮發性設定帶來了幾項重要的設計考量。首先，系統必須在電源穩定後提供足夠時間讓 FPGA 完成設定載入，通常約 1-10 ms（取決於 CRAM 大小與 SPI 時脈頻率）。其次，位元流必須儲存在非揮發性記憶體中，增加了 BOM 成本與 PCB 面積。部分 iCE40 型號（如 LP8K）內建 NVCM（非揮發性設定記憶體）以解決此問題，但 NVCM 僅能重寫有限次數且容量有限。

然而，SRAM 技術也有其優勢：可無限次重寫、設定速度遠快於 Flash-based FPGA、使用標準 CMOS 製程因此邏輯密度高。此外，揮發性也帶來了安全特性——斷電後位元流內容消失，難以透過物理探測擷取設定（相較於 Flash-based FPGA 中設定可長時間留存），這在某些安全敏感應用中反而是優點。

## Frame：CRAM 的基本單位

iCE40 CRAM 的最小組織單元為 Frame。每一個 Frame 具有固定的位元寬度與深度，由以下常數定義（見 `v2f-bitstream/src/frame.rs`）：

```
FRAME_BITS      = 1320 bits
FRAME_BYTES     = 165 bytes
WORDS_PER_FRAME = 33 words
BITS_PER_WORD   = 40 bits
```

亦即每個 Frame 包含 33 個 word，每個 word 為 40 bits（little-endian 儲存），總計 1,320 bits。在 `Frame` 結構中，資料以 `[u8; 165]` 的位元組陣列儲存，並提供逐位元存取（`set_bit()`、`get_bit()`、`clear_bit()`）與逐 word 存取（`set_word()`、`get_word()`）方法。

40-bit word 的寬度並非偶然，它對應到 CRAM SRAM 陣列的實體位元線寬度。在 iCE40 晶片中，CRAM 陣列以 40 個 bitline 為一組進行讀寫操作，因此 1,320 bits 的 Frame 恰好是 33 組 40-bit 的 word。Frame 的 1,320-bit 寬度亦對應到晶片上分布的實體列數——整個 CRAM 陣列就像一個超寬的記憶體，每一「列」就是一個 Frame。

Frame 的 `Frame::new()` 建立全零 Frame，`Frame::from_bytes()` 從原始位元組陣列還原 Frame，`Frame::as_bytes()` 取得唯讀參考以進行序列化。這些方法在 ASC 解析與 BIN 打包過程中頻繁使用。

## Frame 定址與 Row-Major 組織

iCE40 CRAM 採用 row-major（列主序）方式組織 Frame。整個晶片視為一個二維網格，每一列（row）包含若干個 Frame，Frame 編號從 row 0 開始依序排列到最後一列。

每列的 Frame 數量計算公式如下：

```
frames_per_row = (num_cols x 7) + io_left + io_right
```

其中 `num_cols x 7` 為該列所有 Logic Tile 貢獻的 Frame 數（每個 Logic Tile 佔 7 Frame），`io_left` 與 `io_right` 為左右邊界 I/O Tile 貢獻的 Frame 數（各為 3 Frame）。此計算邏輯實作於 `v2f-db/src/ice40.rs` 的 `Ice40Device::frames_per_row()`：

```rust
pub fn frames_per_row(&self) -> u32 {
    let logic = self.num_cols() * 7;
    match self {
        Ice40Device::HX1K => logic + 3 + 3,
        // ... 所有裝置均為左右各 3
    }
}
```

總 Frame 數為：

```
total_frames = num_rows x frames_per_row
```

各裝置的 Frame 統計如下：

| 裝置 | num_rows | num_cols | frames_per_row | total_frames | CRAM 總位元數 |
|---|---|---|---|---|---|
| HX1K / LP1K | 30 | 16 | 118 | 3,540 | 4,672,800 |
| HX4K | 40 | 20 | 146 | 5,840 | 7,708,800 |
| UP5K | 40 | 22 | 160 | 6,400 | 8,448,000 |
| HX8K | 70 | 28 | 202 | 14,140 | 18,664,800 |

以 HX8K 為例，70 列乘以每列 202 Frame，總計約 14,140 個 Frame。這就是 `v2f-db/src/ice40.rs` 中 `total_frames()` 方法的計算結果。

## CRAM 結構在 v2f-bitstream 中的實作

`Cram` 結構是整個位元流打包流程的核心資料結構，定義於 `v2f-bitstream/src/cram.rs`。它封裝了裝置類型與 Frame 向量：

```rust
pub struct Cram {
    pub device: Ice40Device,
    frames: Vec<Frame>,
}
```

提供以下關鍵方法：

- `Cram::new(device)`：建立指定裝置的全零 CRAM，Frame 數量由 `device.total_frames()` 決定。
- `get_frame_mut(frame_idx)`：取得特定 Frame 的可變參考，用於寫入設定位元。
- `get_frame(frame_idx)`：取得特定 Frame 的唯讀參考。
- `frames()`：返回所有 Frame 的迭代器。
- `num_frames()`：返回 Frame 總數。
- `to_bytes()`：將所有 Frame 序列化為連續的位元組向量（每個 Frame 165 bytes），供後續打包為 BIN 檔案。

`Cram` 的設計相當精簡，因為它僅作為位元流的抽象儲存層。設定資訊的實際寫入邏輯由 ASC 解析與套用階段負責（見下文）。

`to_bytes()` 方法是 CRAM 序列化的關鍵，它簡單地將每個 Frame 的 `[u8; 165]` 依序拼接。由於 Frame 儲存時採用 little-endian word 格式，因此序列化後的位元組順序直接對應到位元流中的 Frame 資料區域。

## ASC 格式解析

ASC（ASCII Place-and-Route Output）是 nextpnr 或其他 PNR 工具輸出的純文字配置檔案，描述 FPGA 的完整配置狀態。在 eda4 中，ASC 解析器實作於 `v2f-bitstream/src/asc.rs`。

ASC 檔案以關鍵字標記為基礎，以空格分隔的 token 進行解析。格式範例如下：

```text
.module top
.io_tile 0 0
  .pad 0 clk
.logic_tile 1 1
  .lut 0 2 3 1 0 "0123"
  .wiring 0 0 0 128
.synckey 0x12345678
```

解析後的資料結構為 `AscFile`，包含模組名稱、邏輯拼塊配置（`LogicTileConfig` 列表）、I/O 拼塊配置（`IoTileConfig` 列表）、以及同步金鑰（`synckey`）。每個 `LogicTileConfig` 包含：

- **LUT 配置** (`LutConfig`)：輸出 wire 編號、四個輸入 wire 編號、以及 16-bit 的 init 值（以十六進位字串表示，如 `"0123"` 對應 16 位元的 LUT 真值表內容）。
- **FF 配置** (`FfConfig`)：輸出 wire 編號、可選的時脈致能（ce）與同步重置（sr）輸入。
- **Carry 配置** (`CarryConfig`)：輸出 wire 編號與可選的進位輸入（ci）。
- **繞線連線** (`WiringEntry`)：icestorm 格式的繞線位元索引與設定值（0 或 1）。

每個 `IoTileConfig` 則包含 Pad 配置（引腳編號與名稱）與繞線連線。

`parse_asc()` 函數採用逐行解析的方式，維護當前處理中的 tile（`cur_logic` 或 `cur_io`），遇到 `.logic_tile` 或 `.io_tile` 標記時將前一個 tile 推入結果列表。註解行（以 `#` 開頭）與空白行被忽略。錯誤資訊以 `AscError` 結構記錄行號與錯誤訊息。

## 將 ASC 套用至 CRAM

ASC 解析完成後，`apply_asc_to_cram()` 函數負責將 ASC 配置寫入 CRAM 的正確位置。此函數接收 `AscFile` 的可變參考、`Cram` 的可變參考、以及裝置類型，其流程如下：

首先建立 `CramAddrMap` 用於位址轉換：

```rust
let addr_map = v2f_db::cram_addr::CramAddrMap::new(device);
```

然後分別處理 Logic Tile 與 IO Tile 的繞線設定。對於每個 WiringEntry，需要將 icestorm 格式的位元索引轉換為 (frame_sub, word, bit) 三元組：

- Logic Tile 的 7 個 Frame 共用 1,320 bits 的線性位址空間，因此 `frame_sub = bit_index / (1320 / 7) % 7`。
- IO Tile 的 3 個 Frame 同理：`frame_sub = bit_index / (1320 / 3) % 3`。
- 剩餘的位元偏移再分別計算 word 與 bit。

得到 (frame_sub, word, bit) 後，透過 `CramAddrMap::resolve()` 轉換為絕對 Frame 編號：

```rust
let addr = addr_map.resolve(&tile.pos, TileType::Logic, frame_sub, word, bit);
```

最後設定 CRAM 中該位置的位元（僅當 wiring value 不為 0 時）：

```rust
let f = cram.get_frame_mut(addr.frame);
let bit_pos = (addr.word * 40 + addr.bit) as usize;
f.set_bit(bit_pos);
```

處理完所有 tile 的繞線後，再將 synckey 寫入最後一個 Frame 的固定位元位置。Synckey 是 iCE40 用於驗證位元流完整性的 32-bit 同步標記，其值在配置載入完成後由 FPGA 硬體檢查，若與預期不符則觸發設定錯誤。

## CramAddr 與 CramAddrMap

`CramAddr` 與 `CramAddrMap` 定義於 `v2f-db/src/cram_addr.rs`，是 CRAM 定位的核心抽象層。

`CramAddr` 為一個三元組結構：

```rust
pub struct CramAddr {
    pub frame: u32,  // 絕對 Frame 編號 (0..total_frames)
    pub word: u32,   // Frame 內的 Word 編號 (0..32)
    pub bit: u32,    // Word 內的 Bit 編號 (0..39)
}
```

此結構可唯一標識 CRAM 中的任何一個設定位元。

`CramAddrMap` 負責將 (tile 座標、tile 類型、tile 內 Frame 偏移、word、bit) 轉換為絕對的 `CramAddr`。其關鍵方法為 `resolve()`：

```rust
pub fn resolve(&self, pos: &TilePos, tile_type: TileType,
               frame_within_tile: u32, word: u32, bit: u32) -> CramAddr
```

轉換邏輯分兩步：首先 `tile_start_frame()` 計算 tile 的起始 Frame 編號：

```rust
pub fn tile_start_frame(&self, pos: &TilePos, tile_type: TileType) -> u32 {
    let frames_per_row = self.device.frames_per_row();
    let row_offset = pos.row * frames_per_row;
    let col_offset = pos.col * tile_type.num_frames();
    row_offset + col_offset
}
```

其中 `row_offset` 跳過前面所有列的 Frame，`col_offset` 跳過該列中前方所有 tile 的 Frame。然後將 `start_frame + frame_within_tile` 作為絕對 Frame 編號。

此設計將 CRAM 位址計算與裝置參數解耦，使得 `v2f-bitstream` 的 ASC 解析與 BIN 打包代碼無需關心具體的裝置幾何結構，全部透過 `CramAddrMap` 抽象層處理。

## BIN 位元流格式

iCE40 的 BIN 位元流檔案是 FPGA 配置的最終載體，其格式由 Project IceStorm 逆向工程確立。`pack_bitstream()` 函數（實作於 `v2f-bitstream/src/pack.rs`）將 `Cram` 結構打包為標準 iCE40 bitstream：

```text
[0x00 x 32]        前導 (preamble)：32 bytes 全 0
[BitCount: u32 LE]  位元數：Frame 資料的總位元數 (u32, little-endian)
[Frame0..FrameN]    每個 Frame 165 bytes
[CRC32: u32 LE]     校驗碼：CRC-32 (IEEE 802.3)
```

### 前導 (Preamble)

32 bytes 的全零前導，用於讓 FPGA 的設定介面同步。在 iCE40 的設定協定中，前導空白提供了設定控制器初始化所需的時脈週期。

### 位元數 (Bit Count)

緊接前導之後是一個 32-bit little-endian 無號整數，記錄 CRAM 資料的總位元數。例如 HX1K 為 3,540 x 1,320 = 4,672,800 bits。FPGA 硬體在設定載入過程中以此數值驗證資料長度。

### Frame 資料

所有 Frame 依序排列，每個 Frame 固定為 165 bytes（1,320 bits）。此區域即為 CRAM 設定的完整內容，包含 LUT 真值表、繞線位元、DFF 配置等全部資訊。Frame 總數即為 `device.total_frames()`。

### CRC-32 校驗碼

Frame 資料區域之後為 4-byte 的 CRC-32 校驗碼，使用 IEEE 802.3 / PKZIP 標準多項式 `0xEDB88320`。`crc32()` 函數的實作如下：

```rust
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFFFFFFu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}
```

此實作遵循標準的位元序 CRC 計算方式，每個位元組從 LSB 開始處理。CRC-32 覆蓋整個 Frame 資料區域（不含前導與位元數欄位），提供位元流完整性驗證。iCE40 硬體在設定載入完成後自動比對 CRC，若不匹配則拉高 `CRAM_CRC_ERROR` 引腳並中止啟動。

### pack_bitstream() 完整流程

```rust
pub fn pack_bitstream(cram: &Cram) -> Vec<u8> {
    let mut bitstream = Vec::new();
    // 1. 前導 32 bytes 全零
    bitstream.extend(std::iter::repeat(0u8).take(PREAMBLE_SIZE));
    // 2. 總位元數
    let total_bits = cram.num_frames() * FRAME_BITS as u32;
    bitstream.extend_from_slice(&total_bits.to_le_bytes());
    // 3. CRAM 資料 (所有 Frame)
    let cram_data = cram.to_bytes();
    bitstream.extend_from_slice(&cram_data);
    // 4. CRC32 (僅計算 CRAM 資料)
    let crc = crc32(&cram_data);
    bitstream.extend_from_slice(&crc.to_le_bytes());
    bitstream
}
```

此函數的測試位於同檔案的 `#[cfg(test)]` 模組中，包含 CRC-32 標準測試向量（"123456789" 應產生 `0xCBF43926`）以及完整打包測試（驗證前導、位元數、CRC 的正確性）。

## iCE40 設定協定

iCE40 使用 SPI 相容的序列介面載入位元流。完整的設定協定如下：

1. **CRAM 重置**：主機發送 4 個 `0xFF` 位元組，使 FPGA 內部的 CRAM 位址計數器重置至初始狀態。
2. **前導**：發送 8 個 dummy 位元組（通常為 0x00），提供設定控制器初始化所需的時脈週期。
3. **位元流資料**：依序發送 BIN 檔案內容（從 preamble 開始），FPGA 接收後寫入 CRAM。此階段 FPGA 持續拉低 `DONE` 引腳。
4. **CRC 驗證**：FPGA 接收到 CRC-32 後自動比對，若不匹配則放棄設定並拉高 `CRAM_CRC_ERROR`。
5. **空閒時脈**：發送 48 個 dummy 時脈週期，供 FPGA 完成內部初始化。
6. **喚醒**：FPGA 釋放 `DONE` 引腳（拉高），進入使用者模式，開始執行所載入的設定。

整個設定流程中，FPGA 的 I/O 引腳處於高阻抗狀態，待 `DONE` 訊號確認後才按照設定內容啟用。時脈頻率通常為 10-30 MHz，由主機或外部晶振提供。

## 位元流載入方式

### JTAG 介面

透過標準 IEEE 1149.1 JTAG 介面載入位元流是最常見的除錯與開發方式。iCE40 支援以下 JTAG 指令：

- **Usr1 指令**（0x02）：啟動位元流載入模式，將 JTAG TDI 輸入直接連接至 CRAM 寫入通道。
- **Usr2 指令**（0x03）：用於讀回 CRAM 內容（配置讀回），可用於驗證位元流是否正確寫入。
- **ISC_ENABLE**（0x10）：啟動 ISP（In-System Programming）模式。
- **ISC_DISABLE**（0x11）：結束 ISP 模式。

在 eda4 專案中，`v2f-programmer` crate 提供 mock 模式（用於測試）與選用的 `ftdi` 功能（透過 FTDI FT2232H 等晶片實現實際 JTAG 通訊）。openFPGALoader 是另一個常見的開放原始碼燒錄工具，支援 iCE40 的 FTDI 與 SPI 燒錄。

### SPI Flash

量產環境中，位元流通常預先燒錄至 SPI Flash 記憶體（如 Winbond W25Q32）。iCE40 上電後自動作為 SPI Master 從 Flash 讀取資料。SPI Flash 的連線僅需 4 條訊號線（SCK、SI、SO、CS），PCB 繞線非常簡潔。

此模式支援以下變體：

- **Master SPI**：FPGA 產生時脈，主動讀取 Flash。為最常用的量產設定方式。
- **Slave SPI**：由外部處理器提供時脈並寫入資料，適合系統級動態重配置。
- **Dual SPI / Quad SPI**：使用多條資料線提高傳輸吞吐量，適用於大容量裝置（如 HX8K）以縮短設定時間。

### 安全性與位元流加密

iCE40 支援 AES-128 位元流加密（需使用特定型號如 iCE40 HX 8K-C 等含解密器的版本）。加密後的位元流以金鑰儲存於晶片內的專用記憶體（透過 JTAG 燒錄金鑰），每次上電後 FPGA 自動解密載入。eda4 目前未實作位元流加密功能，但位元流格式本身預留了加密相關的擴展空間。

## 與其他 FPGA 位元流格式的比較

### Xilinx .bit

Xilinx FPGA 的 .bit 格式包含固定長度的前導（包含同步字 `0xAA995566`、裝置 ID、日期戳等 metadata），後接 Frame 資料與 CRC。Xilinx 的 Frame 結構完全不同：每個 Frame 對應到 Configurable Logic Block (CLB) 的一「行」，Frame 長度因裝置而異（如 7-Series 為 1,012 words）。Xilinx 的 CRC 使用 CCITT 32 而非 IEEE 802.3。與 iCE40 相比，Xilinx 格式更為複雜，包含大量冗餘的 metadata 與除錯資訊。

### Altera/Intel .sof

Altera 的 .sof（SRAM Object File）為專有格式，其架構基於「區塊」(block) 而非 Frame。每個區塊具有獨立的類型標記與 CRC。Altera 也支援 .pof（Programmer Object File）用於 Flash 燒錄。Intel Quartus 工具鏈產生的 .sof 可轉換為 .rbf（Raw Binary File）供處理器載入。與 iCE40 的簡潔格式相比，Altera 格式更關注靈活性與進階功能（如遠端更新、多映像支援）。

### Lattice .jed

Lattice Diamond 工具鏈產生的 .jed（JEDEC）格式不同於 iCE40 的 BIN 格式。.jed 是一種標準化的 ASCII 格式，用於描述可程式邏輯元件的配置資料，包含熔絲陣列資訊。iCE40 的 BIN 格式則更接近底層 CRAM 的二進位映像，不經過 JEDEC 中間格式。這是因為 iCE40 的開放生態系（IceStorm + Yosys + nextpnr）直接產生 BIN，跳過了 Lattice 專有工具的層層轉換。

### 格式比較總結

| 特性 | iCE40 .bin | Xilinx .bit | Altera .sof | Lattice .jed |
|---|---|---|---|---|
| Frame 結構 | 33 words x 40 bits | 裝置相關 | 區塊式 | 熔絲陣列 |
| 前導 | 32 bytes 全零 | 含同步字與 metadata | 含區塊類型標記 | ASCII header |
| CRC | IEEE 802.3 (CRC-32) | CCITT-32 | 各區塊獨立 CRC | 無統一 CRC |
| 加密支援 | AES-128 (選用) | AES-256 (選用) | AES-256 (選用) | 無 |
| 中繼格式 | 無 (直接二進位) | 可轉 .bin | .sof -> .rbf | ASCII JEDEC |
| 檔案大小 | CRAM 大小 + 40 bytes | 略大於 CRAM | 大於 CRAM | 通常較大 |

## Project IceStorm 的關鍵貢獻

2015 年，Claire Wolf（Yosys 與 nextpnr 的開發者）發起了 Project IceStorm，對 Lattice iCE40 的 CRAM 位元流格式進行了完整的逆向工程。此專案的成果奠定了開放原始碼 iCE40 工具鏈的基礎，其貢獻包括：

**CRAM 位址映射資料庫**：IceStorm 專案釋出了 `icebox` 資料庫，以 CSV 與 Python 格式記錄了每個裝置的 Frame 佈局、每個 tile 的 Frame 偏移量、以及每個繞線位元的確切位置。此資料庫是 `v2f-db` crate 的主要參考依據。

**icepack/iceunpack 工具**：位元流打包與解包工具。`icepack` 可將 ASC 轉換為 BIN（Python 實作），`iceunpack` 則可將 BIN 解包回 ASC 進行除錯。eda4 的 `pack_bitstream()` 函數等效於 `icepack` 的 BIN 輸出功能。

**icetime 時序分析工具**：透過分析 CRAM 中繞線位元的延遲資料，計算設計的最大運作頻率。

**iceprog 燒錄工具**：透過 FTDI 晶片或直接連接 SPI Flash 編程器，將 BIN 位元流寫入外部 Flash 或 FPGA 的 CRAM。

IceStorm 的成功證明了 FPGA 位元流逆向工程的可行性，此後也激發了針對 Xilinx 7-Series（Project X-Ray）、Lattice ECP5（Project Trellis）、Intel MAX 10（Project Chibi）等 FPGA 架構的類似逆向工程專案，形成了開放原始碼 FPGA 生態系的重要基礎。

## 在 eda4 中的整體流程

eda4 的完整位元流產生流程總結如下：

```
Verilog (.v) -> v2f-synth (合成) -> JSON netlist
                                        |
                                        v
                                   v2f-pnr (佈局繞線)
                                        |
                                        v
                                    ASC 檔案
                                        |
                                        v
                  v2f-bitstream::parse_asc() (解析 ASC)
                                        |
                                        v
                  v2f-bitstream::apply_asc_to_cram() (寫入 CRAM)
                                        |
                                        v
                  v2f-bitstream::pack_bitstream() (打包 BIN)
                                        |
                                        v
                  _out/*.bin (位元流檔案)
```

其中 `v2f-bitstream` crate 負責從 ASC 解析到 BIN 打包的完整流程，依賴 `v2f-db` crate 提供的裝置參數與 CRAM 位址映射資訊。產生的 BIN 檔案可直接透過 `v2f-programmer` crate 或外部工具（如 `iceprog`、`openFPGALoader`）燒錄至 iCE40 FPGA。

## 參考資料

- Project IceStorm: https://github.com/YosysHQ/icestorm
- Yosys Open SYnthesis Suite: https://github.com/YosysHQ/yosys
- nextpnr: https://github.com/YosysHQ/nextpnr
- Lattice iCE40 Programming and Configuration Technical Note (TN1250)
- Lattice iCE40 LP/HX Family Data Sheet (DS1040)
- Lattice iCE40 UltraPlus Family Data Sheet (DS1048)
- eda4 原始碼: verilog2fpga/v2f-bitstream/src/
- eda4 原始碼: verilog2fpga/v2f-db/src/
