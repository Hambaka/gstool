use std::{fmt, sync::OnceLock};

mod codegen_bps {
  #![allow(clippy::all, clippy::pedantic, clippy::nursery)]
  include!(concat!(env!("OUT_DIR"), "/codegen_bps.rs"));
}

/// 输入ROM的分类
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatchSource {
  /// 最新汉化版，无需打补丁
  AlreadyLatestChinese,
  /// 可打补丁，携带输入ROM的CRC32值（用于日志展示）
  Patchable { crc32: u32 },
  /// 与任何已知的ROM来源都不匹配
  Unsupported,
}

#[derive(Debug)]
pub enum PatchRomError {
  UnsupportedRom,
  AlreadyLatestChinese,
  /// 缺失对应补丁，理论上不会发生
  MissingPatch {
    crc32: u32,
  },
  BpsFailed,
}

impl fmt::Display for PatchRomError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::UnsupportedRom => write!(f, "不支持输入的 ROM 文件！"),
      Self::AlreadyLatestChinese => {
        write!(f, "输入的是已经应用了最新版汉化补丁的ROM文件！")
      }
      Self::MissingPatch { crc32 } => {
        write!(f, "没有CRC32为{crc32:#010X}的补丁！")
      }
      Self::BpsFailed => write!(f, "应用BPS补丁失败！"),
    }
  }
}

/// 打了补丁后的ROM文件名（无扩展名）
pub const PATCHED_FILE_STEM: &str = codegen_bps::LATEST_CHS_FILE_STEM;

/// 分类输入的ROM
pub fn classify(rom: &[u8]) -> PatchSource {
  let crc32 = crc32fast::hash(rom);
  if crc32 == codegen_bps::LATEST_CHS_CHECKSUM {
    PatchSource::AlreadyLatestChinese
  } else if codegen_bps::VALID_INPUT_ROM_CHECKSUM.contains(&crc32) {
    PatchSource::Patchable { crc32 }
  } else {
    PatchSource::Unsupported
  }
}

/// 用于日志展示的 ROM 名称
pub fn rom_display_name(crc32: u32) -> Option<&'static str> {
  codegen_bps::VALID_INPUT_ROM_INFO_MAP.get(&crc32).copied()
}

pub struct PatchedRom {
  pub bytes: Vec<u8>,
  /// 最源头的输入ROM的CRC32值
  pub source_crc32: u32,
}

/// 应用新汉化补丁：沿CRC32链反复应用BPS补丁，至多两次抵达新汉化版。
///
/// 输入必须是未经修改的原版、老汉化版或者其他语言的版本的ROM；输出是打了补丁后的完整ROM。
pub fn apply_chinese_patch(rom: Vec<u8>) -> Result<PatchedRom, PatchRomError> {
  let source_crc32 = crc32fast::hash(&rom);
  match classify(&rom) {
    PatchSource::AlreadyLatestChinese => return Err(PatchRomError::AlreadyLatestChinese),
    PatchSource::Unsupported => return Err(PatchRomError::UnsupportedRom),
    PatchSource::Patchable { .. } => {}
  }

  let patches = all_patches();
  let mut current = rom;
  let mut current_crc32 = source_crc32;

  // 如果输入的是日版原版ROM，只需循环一次（日版原版 -> 最新汉化版）
  // 如果输入的是其他的汉化版或者其他语言的民间翻译版本ROM，需要循环两次（其他民间翻译版 -> 日版原版 -> 最新汉化版）
  // 如果输入的是美/德/西/法/意原版ROM，需要循环两次（美/德/西/法/意原版 -> 日版原版 -> 最新汉化版）
  for _ in 0..2 {
    let patch_crc32 =
      *codegen_bps::BPS_CRC32_MAP
        .get(&current_crc32)
        .ok_or(PatchRomError::MissingPatch {
          crc32: current_crc32,
        })?;
    let (start, size) = *codegen_bps::BPS_FILE_INDEX_MAP
      .get(&patch_crc32)
      .ok_or(PatchRomError::MissingPatch { crc32: patch_crc32 })?;

    let (next_rom, next_crc32) =
      crate::util::bps::apply_bps_patch(&current, &patches[start..start + size])
        .map_err(|_| PatchRomError::BpsFailed)?;
    current = next_rom;
    current_crc32 = next_crc32;

    if current_crc32 == codegen_bps::LATEST_CHS_CHECKSUM {
      return Ok(PatchedRom {
        bytes: current,
        source_crc32,
      });
    }
  }

  Err(PatchRomError::BpsFailed)
}

/// 内嵌的全部BPS补丁，惰性解压一次并缓存。
fn all_patches() -> &'static [u8] {
  static PATCHES: OnceLock<Vec<u8>> = OnceLock::new();

  PATCHES.get_or_init(|| {
    let compressed = include_bytes!(concat!(env!("OUT_DIR"), "/all_bps.zst")).as_slice();
    zrip_decode::decompress(compressed).unwrap()
    /*
    zstd库其一：zstd
    zstd::decode_all(include_bytes!(concat!(env!("OUT_DIR"), "/all_bps.zst")).as_slice())
      .unwrap()

    zstd库其二：structured-zstd
    let mut compressed_data =
      include_bytes!(concat!(env!("OUT_DIR"), "/all_bps.zst")).as_slice();
    let mut decoder =
      structured_zstd::decoding::StreamingDecoder::new(&mut compressed_data).unwrap();
    let mut decompreessed_data = Vec::new();
    decoder.read_to_end(&mut decompreessed_data).unwrap();
    decompreessed_data

    zstd库其三：ruzstd
    let compressed = include_bytes!(concat!(env!("OUT_DIR"), "/all_bps.zst")).as_slice();
    let mut decoder = ruzstd::decoding::StreamingDecoder::new(compressed).unwrap();
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed).unwrap();
    decompressed
    */
  })
}
