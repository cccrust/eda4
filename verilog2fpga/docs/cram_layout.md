# CRAM 位元布局

本文說明 iCE40 FPGA 內部配置 RAM（CRAM）的位元布局。這是理解 `v2f-bitstream` 打包和 `v2f-bitdecode` 解碼的基礎。

## Frame 結構

iCE40 的基本配置單元是 **frame**：
- **大小**：1,320 bits = 165 bytes
- **組織**：33 個 word，每個 word 40 bits
- **位元組偏移**：`word_idx * 5`（每個 word 佔 5 bytes，LSB 在前）

```
Bit position  :  [39..32][31..24][23..16][15..8][7..0]  (Byte 4..0 of word)
Bit 0        :  LSB of Byte 0
Bit 39       :  MSB of Byte 4
```

## Logic Tile 布局（7 frames × 33 words）

每個 Logic Tile 佔 7 個連續的 frame。總計 7 × 33 × 40 = 9,240 bits。

### 幀分配

| Frame 範圍 | 用途 |
|-----------|------|
| Frame 0-3 | LUT init 值（LUT0 / LUT1）|
| Frame 4 | LUT2 / LUT3 init + 攜帶鏈 |
| Frame 5-6 | DFF 配置 + 路由 |

**注意**：上述為理論布局。實際上， LUT init 位元的精確映射（哪個 frame/word/bit 對應 LUT0 init 的哪個 bit）需要完整的 icestorm bit database。本文件只描述 `v2f-bitstream` 目前已知且已實作的部分。

### 我們目前支援的部分

`apply_asc_to_cram` 目前只寫入 **wiring bits**，不寫入 LUT init 值。這是因為完整的 LUT init 編碼需要參考 icestorm 的 bit mapping 資料庫。

Wiring bits 的編碼方式如下：

**Logic Tile（7 frames）**：
- 每 frame 可定址範圍：`1320 / 7 = 188` bits
- 實際寫入範圍：`word * 40 + bit < 188`，即 word 0-4（5 個 word，200 bits）
- `bit_index` 編碼：
  ```
  bit_index = frame_sub * 188 + word * 40 + bit
  ```
- 範例：`bit_index=0` → frame_sub=0, word=0, bit=0

**IO Tile（3 frames）**：
- 每 frame 可定址範圍：`1320 / 3 = 440` bits
- 實際寫入範圍：`word * 40 + bit < 440`，即 word 0-10
- `bit_index` 編碼：
  ```
  bit_index = frame_sub * 440 + word * 40 + bit
  ```

### 為什麼 wiring bits 有限制？

`apply_asc_to_cram` 的計算：
```rust
let frame_sub = (w.bit_index / 188) % 7;  // Logic
let remaining = w.bit_index % 188;
let word = remaining / 40;  // 最大 188/40 = 4（word 0-4）
let bit = remaining % 40;    // 0-39
```

這意味著只能寫入每個 Logic Tile frame 的前 5 個 word（bits 0-199）。Word 5-32（bits 200-1319）從未被 wiring 編碼使用。這些區域用於：
- LUT init 值
- DFF 配置
- 路由配置（Routing multiplexers）

## Tile Frame 定址

`CramAddrMap` 提供 Tile 位置到 CRAM frame 的映射：

```rust
pub struct CramAddrMap {
    device: Ice40Device,
    // 內部計算
}

impl CramAddrMap {
    // 計算指定 Tile 的第一個 CRAM frame 編號
    pub fn tile_start_frame(&pos: &TilePos, tile_type: TileType) -> u32

    // 解析 wiring bit_index 為實際 CRAM 地址
    pub fn resolve(&pos: &TilePos, tile_type: TileType,
                   frame_sub: u32, word: u32, bit: u32) -> CramAddr
}
```

**CramAddr 結構**：
```rust
pub struct CramAddr {
    pub frame: u32,  // 全域 CRAM frame 編號
    pub word: u32,    // frame 內 word 索引（0-32）
    pub bit: u32,     // word 內 bit 位置（0-39）
}
```

## CRAM 總覽（以 HX1K 為例）

```
總列數：30 rows
左 IO 列：col = 0（30 tiles × 3 frames = 90 frames）
Logic 列：col = 1..16（16 × 30 tiles × 7 frames = 3,360 frames）
右 IO 列：col = 17（30 tiles × 3 frames = 90 frames）

IO 區域：90 × 2 = 180 frames
Logic 區域：3,360 frames
其他：？frames
─────────────────
總計：6,624 frames
```

## Synckey 位置

Synckey 是使用者資料（32-bit），放在 CRAM 的最後一個 frame 的最高位：

```
最後一個 frame（frame N-1）
Bit 1319（最高位）: synckey bit 31
Bit 1318           : synckey bit 30
...
Bit 1288           : synckey bit 0
Bit 1287-0         : 未使用
```

編碼時（`apply_asc_to_cram`）：
```rust
let pos = (FRAME_BITS - 1) - (i * 8 + bit);  // i = 0..3, bit = 0..7
last_frame.set_bit(pos);
```

解碼時（`decode_synckey`）：
```rust
for i in 0..4 {
    for bit in 0..8 {
        let pos = (FRAME_BITS - 1) - (i * 8 + bit);
        if last_frame.get_bit(pos) {
            byte |= 1 << bit;
        }
    }
    key |= (byte as u32) << (i * 8);
}
```

## Decode 逆向映射

給定 CRAM 中某個 set bit 的位置，如何還原 `bit_index`？

**Logic Tile**（假設 bit 在 frame_sub, word, bit_position）：
```
remaining = word * 40 + bit_position
if remaining < 188:
    bit_index = frame_sub * 188 + remaining
```

**IO Tile**（假設 bit 在 frame_sub, word, bit_position）：
```
remaining = word * 40 + bit_position
if remaining < 440:
    bit_index = frame_sub * 440 + remaining
```

此映射用於 `decode_cram()` 的 wiring 反向解碼。

## 完整的 Bit Mapping

完整的 LUT init / FF / IO 配置需要參考 Project IceStorm 的 `iceboxdb.py` 資料庫。該資料庫定義了：

- `database_logic_txt`：Logic Tile 的配置位元模式（1,598 行）
- `database_io_txt`：IO Tile 的配置位元模式

這些資料庫使用 `Bk[i]` 格式（bit line k, position i）來描述配置條件，與 CRAM frame 的映射由 `icepack/icepack.cc` 實作。

**完整實現需要**：
1. 將 `database_logic_txt` 轉換為 Rust 查詢表
2. 實作 `16×54 tile matrix → 7 frames` 的映射邏輯
3. 在 `apply_asc_to_cram` 中實作 LUT init 編碼
4. 在 `decode_cram` 中實作完整的 LUT init 解碼

這是 v0.9 的已知限制，已記錄在 `v2f-bitdecode` 的 `decode_level` 欄位（目前為 `"wiring"`，目標為 `"lut"`）。