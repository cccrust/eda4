/// ASC 格式解析器
///
/// ASC 是 nextpnr 輸出的純文字配置檔，描述 FPGA 的完整配置狀態。
/// 包含 Tile 配置（LUT、FF、Carry）與繞線連線資訊。
///
/// 格式範例：
/// ```text
/// .module top
/// .io_tile 0 0
///   .pad 0 clk
/// .logic_tile 1 1
///   .lut 0 2 3 1 0 "0123"
///   .wiring 0 0 0 128
/// .synckey 0x12345678
/// ```

use std::fmt;

use v2f_db::ice40::Ice40Device;
use v2f_db::tile::TilePos;

use crate::cram::Cram;

/// 解析錯誤
#[derive(Debug)]
pub struct AscError {
    pub line: usize,
    pub msg: String,
}

impl fmt::Display for AscError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ASC 解析錯誤 (第 {} 行): {}", self.line, self.msg)
    }
}

/// Logic Tile 配置
#[derive(Debug, Clone)]
pub struct LogicTileConfig {
    pub pos: TilePos,
    pub luts: Vec<LutConfig>,
    pub ffs: Vec<FfConfig>,
    pub carries: Vec<CarryConfig>,
    pub wiring: Vec<WiringEntry>,
}

/// LUT 配置 (16-bit init 值)
#[derive(Debug, Clone)]
pub struct LutConfig {
    pub output: u32,
    pub inputs: [u32; 4],
    pub init: u16,
}

/// Flip-Flop 配置
#[derive(Debug, Clone)]
pub struct FfConfig {
    pub output: u32,
    pub ce: Option<u32>,
    pub sr: Option<u32>,
}

/// Carry 配置
#[derive(Debug, Clone)]
pub struct CarryConfig {
    pub output: u32,
    pub ci: Option<u32>,
}

/// 繞線連線 (icestorm wiring bit 設定)
#[derive(Debug, Clone)]
pub struct WiringEntry {
    pub bit_index: u32,
    pub value: u32,
}

/// IO Tile 配置
#[derive(Debug, Clone)]
pub struct IoTileConfig {
    pub pos: TilePos,
    pub pads: Vec<PadConfig>,
    pub wiring: Vec<WiringEntry>,
}

/// IO Pad 配置
#[derive(Debug, Clone)]
pub struct PadConfig {
    pub index: u32,
    pub name: String,
}

/// 已解析的 ASC 檔案
#[derive(Debug, Clone)]
pub struct AscFile {
    pub module: String,
    pub logic_tiles: Vec<LogicTileConfig>,
    pub io_tiles: Vec<IoTileConfig>,
    pub synckey: u32,
}

/// 將 ASC 解析為內部結構
pub fn parse_asc(input: &str) -> Result<AscFile, AscError> {
    let mut module = String::from("top");
    let mut logic_tiles = Vec::new();
    let mut io_tiles = Vec::new();
    let mut synckey = 0u32;

    let mut cur_logic: Option<LogicTileConfig> = None;
    let mut cur_io: Option<IoTileConfig> = None;

    for (lineno, line) in input.lines().enumerate() {
        let line = match line.find('#') {
            Some(pos) => &line[..pos],
            None => line,
        };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            ".module" => {
                if parts.len() >= 2 {
                    module = parts[1].to_string();
                }
            }
            ".logic_tile" => {
                if let Some(cfg) = cur_logic.take() {
                    logic_tiles.push(cfg);
                }
                if parts.len() >= 3 {
                    let row: u32 = parts[1]
                        .parse()
                        .map_err(|e| err(lineno, format!("無效 row: {e}")))?;
                    let col: u32 = parts[2]
                        .parse()
                        .map_err(|e| err(lineno, format!("無效 col: {e}")))?;
                    cur_logic = Some(LogicTileConfig {
                        pos: TilePos { row, col },
                        luts: Vec::new(),
                        ffs: Vec::new(),
                        carries: Vec::new(),
                        wiring: Vec::new(),
                    });
                }
            }
            ".io_tile" => {
                if let Some(cfg) = cur_io.take() {
                    io_tiles.push(cfg);
                }
                if let Some(cfg) = cur_logic.take() {
                    logic_tiles.push(cfg);
                }
                if parts.len() >= 3 {
                    let row: u32 = parts[1].parse().map_err(|e| err(lineno, format!("無效 row: {e}")))?;
                    let col: u32 = parts[2].parse().map_err(|e| err(lineno, format!("無效 col: {e}")))?;
                    cur_io = Some(IoTileConfig {
                        pos: TilePos { row, col },
                        pads: Vec::new(),
                        wiring: Vec::new(),
                    });
                }
            }
            ".lut" => {
                if let Some(ref mut cfg) = cur_logic {
                    if parts.len() >= 7 {
                        let output = parts[1].parse().map_err(|e| err(lineno, format!("無效 lut output: {e}")))?;
                        let i0 = parts[2].parse().map_err(|e| err(lineno, format!("無效 lut input0: {e}")))?;
                        let i1 = parts[3].parse().map_err(|e| err(lineno, format!("無效 lut input1: {e}")))?;
                        let i2 = parts[4].parse().map_err(|e| err(lineno, format!("無效 lut input2: {e}")))?;
                        let i3 = parts[5].parse().map_err(|e| err(lineno, format!("無效 lut input3: {e}")))?;
                        let hex_str = parts[6].trim_matches('"');
                        let init = u16::from_str_radix(hex_str, 16)
                            .map_err(|e| err(lineno, format!("無效 lut init: {e}")))?;
                        cfg.luts.push(LutConfig {
                            output,
                            inputs: [i0, i1, i2, i3],
                            init,
                        });
                    }
                }
            }
            ".ff" => {
                if let Some(ref mut cfg) = cur_logic {
                    if parts.len() >= 2 {
                        let output = parts[1].parse().map_err(|e| err(lineno, format!("無效 ff output: {e}")))?;
                        let mut ce = None;
                        let mut sr = None;
                        for p in &parts[2..] {
                            if let Some(val) = p.strip_prefix("ce=") {
                                ce = Some(val.parse().map_err(|e| err(lineno, format!("無效 ff ce: {e}")))?);
                            } else if let Some(val) = p.strip_prefix("sr=") {
                                sr = Some(val.parse().map_err(|e| err(lineno, format!("無效 ff sr: {e}")))?);
                            }
                        }
                        cfg.ffs.push(FfConfig { output, ce, sr });
                    }
                }
            }
            ".carry" => {
                if let Some(ref mut cfg) = cur_logic {
                    if parts.len() >= 2 {
                        let output = parts[1].parse().map_err(|e| err(lineno, format!("無效 carry output: {e}")))?;
                        let mut ci = None;
                        for p in &parts[2..] {
                            if let Some(val) = p.strip_prefix("ci=") {
                                ci = Some(val.parse().map_err(|e| err(lineno, format!("無效 carry ci: {e}")))?);
                            }
                        }
                        cfg.carries.push(CarryConfig { output, ci });
                    }
                }
            }
            ".wiring" => {
                if let Some(ref mut cfg) = cur_logic {
                    if parts.len() >= 4 {
                        let bit = parts[1].parse().map_err(|e| err(lineno, format!("無效 wiring bit: {e}")))?;
                        let value = parts[3].parse().map_err(|e| err(lineno, format!("無效 wiring value: {e}")))?;
                        cfg.wiring.push(WiringEntry { bit_index: bit, value });
                    }
                } else if let Some(ref mut cfg) = cur_io {
                    if parts.len() >= 4 {
                        let bit = parts[1].parse().map_err(|e| err(lineno, format!("無效 wiring bit: {e}")))?;
                        let value = parts[3].parse().map_err(|e| err(lineno, format!("無效 wiring value: {e}")))?;
                        cfg.wiring.push(WiringEntry { bit_index: bit, value });
                    }
                }
            }
            ".pad" => {
                if let Some(ref mut cfg) = cur_io {
                    if parts.len() >= 3 {
                        let index = parts[1].parse().map_err(|e| err(lineno, format!("無效 pad index: {e}")))?;
                        let name = parts[2].to_string();
                        cfg.pads.push(PadConfig { index, name });
                    }
                }
            }
            ".synckey" => {
                if parts.len() >= 2 {
                    let s = parts[1];
                    synckey = if s.starts_with("0x") || s.starts_with("0X") {
                        u32::from_str_radix(&s[2..], 16)
                            .map_err(|e| err(lineno, format!("無效 synckey: {e}")))?
                    } else {
                        s.parse().map_err(|e| err(lineno, format!("無效 synckey: {e}")))?
                    };
                }
            }
            _ => {}
        }
    }

    if let Some(cfg) = cur_logic {
        logic_tiles.push(cfg);
    }
    if let Some(cfg) = cur_io {
        io_tiles.push(cfg);
    }

    Ok(AscFile {
        module,
        logic_tiles,
        io_tiles,
        synckey,
    })
}

/// 將 AscFile 應用到 CRAM（填入所有 tile 配置）
///
/// 注意：此為精簡實作，僅處理基本的 wiring 位元設定。
/// 完整的 LUT/FF/Carry 對應需參考 icestorm 的 icebox 資料庫。
pub fn apply_asc_to_cram(asc: &AscFile, cram: &mut Cram, device: Ice40Device) {
    let addr_map = v2f_db::cram_addr::CramAddrMap::new(device);

    // Logic tiles: 將 wiring 設定寫入 CRAM
    for tile in &asc.logic_tiles {
        for w in &tile.wiring {
            // 每個 Logic tile 佔 7 個 frame
            // wiring 的 bit_index 格式：
            //   高 bit = frame_within_tile (0..6)
            //   中 bit = word (0..32)
            //   低 bit = bit_within_word (0..39)
            let frame_sub = (w.bit_index / (crate::frame::FRAME_BITS as u32 / 7)) % 7;
            let remaining = w.bit_index % (crate::frame::FRAME_BITS as u32 / 7);
            let word = remaining / 40;
            let bit = remaining % 40;

            let addr = addr_map.resolve(
                &tile.pos,
                v2f_db::tile::TileType::Logic,
                frame_sub,
                word,
                bit,
            );
            let f = cram.get_frame_mut(addr.frame);
            if w.value != 0 {
                let bit_pos = (addr.word * 40 + addr.bit) as usize;
                f.set_bit(bit_pos);
            }
        }
    }

    // IO tiles: 同樣處理 wiring
    for tile in &asc.io_tiles {
        for w in &tile.wiring {
            let frame_sub = (w.bit_index / (crate::frame::FRAME_BITS as u32 / 3)) % 3;
            let remaining = w.bit_index % (crate::frame::FRAME_BITS as u32 / 3);
            let word = remaining / 40;
            let bit = remaining % 40;

            let addr = addr_map.resolve(
                &tile.pos,
                v2f_db::tile::TileType::Io,
                frame_sub,
                word,
                bit,
            );
            let f = cram.get_frame_mut(addr.frame);
            if w.value != 0 {
                let bit_pos = (addr.word * 40 + addr.bit) as usize;
                f.set_bit(bit_pos);
            }
        }
    }

    // 寫入 synckey (放置在最後一個 frame 的固定位置)
    let last_frame = cram.get_frame_mut(cram.num_frames() - 1);
    let key_bytes = asc.synckey.to_le_bytes();
    for (i, &b) in key_bytes.iter().enumerate() {
        for bit in 0..8 {
            if (b >> bit) & 1 == 1 {
                let pos = (crate::frame::FRAME_BITS - 1) - (i * 8 + bit);
                last_frame.set_bit(pos);
            }
        }
    }
}

fn err(lineno: usize, msg: String) -> AscError {
    AscError {
        line: lineno + 1,
        msg,
    }
}
