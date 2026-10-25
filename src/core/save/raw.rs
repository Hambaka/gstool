use std::{collections::HashMap, fmt};

use crate::core::{
  enums::{GameRegion, GameTitle},
  save::format::{
    BUILD_DATE_OFFSET, CLEAR_DATA_IDENTIFIER_VALUE, CLEAR_DATA_OFFSET, FIRST_BUILD_DATE_OFFSET,
    GS_BUILD_DATE, HEADER_CAMELOT_ASCII_STRING, MAX_LOOP_COUNT, MAX_VALID_SLOT_INDEX,
    PRIORITY_OFFSET, SAVE_SLOT_SIZE, SLOT_HEADER_SIZE, SLOT_NUMBER_OFFSET, VALID_SAVE_SIZES,
  },
};

#[derive(Debug)]
pub enum SaveFileError {
  BadFileSize(u64),
  NoSaveData,
  NoClearData,
}

impl fmt::Display for SaveFileError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::BadFileSize(len) => write!(
        f,
        "存档文件的大小应为64/128/136KB！（实际{}KB）",
        len / 1024
      ),
      Self::NoSaveData => write!(
        f,
        "输入的不是GBA《黄金太阳》系列游戏的存档文件！或者存档文件中没有数据！"
      ),
      Self::NoClearData => write!(f, "存档文件中没有通关数据！"),
    }
  }
}

struct HeaderInfo {
  /// Range: 0 <= x < 16
  index: usize,
  priority: u16,
  /// Start from 0
  slot_number: u8,
  /// Please note, the value of this is not in save header, this is only for checking valid header/save
  is_clear_data: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct GameInfo {
  /// 游戏类型：
  /// 《开启的封印》和《失落的时代》
  pub title: GameTitle,
  /// 游戏的语言版本：
  /// 日、英、德、西、法、意
  pub region: GameRegion,
  /// 第一个存档数据出现在存档文件的位置对应的索引。
  /// 存档文件是64KB，而初代每个槽位是4KB，2代每个槽位是12KB。
  /// 最初的检测时，每次遍历都是以4KB为单位/步长，这样初代和2代都通用。
  /// 而之后的实际转换存档时每次遍历就使用各自的大小了，所以第一次记录时，对于2代的下标要进行特殊处理（即除以3）。
  pub first_slot_index: usize,
}

impl GameInfo {
  pub fn new(title: GameTitle, region: GameRegion, first_slot_index: usize) -> Self {
    Self {
      title,
      region,
      first_slot_index,
    }
  }
}

#[derive(Debug, Clone)]
pub struct SaveSlot {
  // Please note, the priority is in save header, this is only for checking valid header/save
  pub priority: u16,
  pub is_clear_data: bool,
  pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct SaveSlots {
  pub slots: [Option<SaveSlot>; 3],
  pub first_available_slot: Option<u8>,
}

pub fn find_first_slot_index(raw_save_file: &[u8]) -> Option<usize> {
  for i in 0..MAX_LOOP_COUNT[0] {
    if !has_camelot_header(raw_save_file, i) {
      continue;
    }

    let build_date_from_raw_save = u16::from_le_bytes([
      raw_save_file[i * SAVE_SLOT_SIZE[0] + FIRST_BUILD_DATE_OFFSET],
      raw_save_file[i * SAVE_SLOT_SIZE[0] + FIRST_BUILD_DATE_OFFSET + 1],
    ]);

    if GS_BUILD_DATE[0].contains(&build_date_from_raw_save) {
      return Some(i);
    }
  }

  None
}

pub fn identify_game(raw_save_file: &[u8]) -> Option<GameInfo> {
  for i in 0..MAX_LOOP_COUNT[0] {
    if !has_camelot_header(raw_save_file, i) {
      continue;
    }

    let build_date = u16::from_le_bytes([
      raw_save_file[i * SAVE_SLOT_SIZE[0] + BUILD_DATE_OFFSET[0][0]],
      raw_save_file[i * SAVE_SLOT_SIZE[0] + BUILD_DATE_OFFSET[0][0] + 1],
    ]);

    let game_version_option = GameRegion::from_save_build_date(build_date);

    if game_version_option.is_none() {
      continue;
    } else {
      let game_version = game_version_option.unwrap();
      if GS_BUILD_DATE[0].contains(&build_date) {
        return Some(GameInfo::new(GameTitle::TheBrokenSeal, game_version, i));
      }
      // 存档文件是64KB，而初代每个槽位是4KB，2代每个槽位是12KB。
      // 最初的检测时，每次遍历都是以4KB为单位/步长，这样初代和2代都通用。
      // 之后的实际转换存档时每次遍历使用各自的步长大小了，所以对于2代的下标要进行特殊处理（即除以3）。
      return Some(GameInfo::new(GameTitle::TheLostAge, game_version, i / 3));
    }
  }

  None
}

pub fn collect_slots(
  raw: &[u8],
  first_slot_index: usize,
  include_non_clear: bool,
) -> HashMap<u8, SaveSlot> {
  let mut all_possible_headers = Vec::new();
  // "u8" is slot number. (Start from 0)
  let mut save_data_map: HashMap<u8, SaveSlot> = HashMap::new();
  for i in first_slot_index..MAX_LOOP_COUNT[0] {
    if raw[i * SAVE_SLOT_SIZE[0] + SLOT_NUMBER_OFFSET] > MAX_VALID_SLOT_INDEX {
      continue;
    }
    let is_clear_save =
      raw[i * SAVE_SLOT_SIZE[0] + CLEAR_DATA_OFFSET] == CLEAR_DATA_IDENTIFIER_VALUE;
    if !is_clear_save && !include_non_clear {
      continue;
    }
    // Get all possible valid save headers first.
    all_possible_headers.push(HeaderInfo {
      index: i,
      priority: u16::from_le_bytes([
        raw[i * SAVE_SLOT_SIZE[0] + PRIORITY_OFFSET],
        raw[i * SAVE_SLOT_SIZE[0] + PRIORITY_OFFSET + 1],
      ]),
      slot_number: raw[i * SAVE_SLOT_SIZE[0] + SLOT_NUMBER_OFFSET],
      is_clear_data: is_clear_save,
    });
  }

  // Get all valid save data.
  for (j, header) in all_possible_headers.iter().enumerate() {
    if j > 0
      && save_data_map.contains_key(&header.slot_number)
      && let Some(existed_save_data) = save_data_map.get(&header.slot_number)
    {
      if header.priority > existed_save_data.priority {
        save_data_map.remove(&header.slot_number);
      } else {
        continue;
      }
    }
    /*
    Does not include first 16 bytes header.
    The data from 0xA40 (Felix, Jenna, Sheba, PC07) is useless for password generating.
    0xA40 - 0x10 = 0xA30

    Default:
    SaveSlot {
      priority: 0,
      is_clear_data: false,
      data: vec![0; 0xA30],
    }
    */
    let save_data = SaveSlot {
      priority: header.priority,
      is_clear_data: header.is_clear_data,
      data: raw
        .get(
          header.index * SAVE_SLOT_SIZE[0] + SLOT_HEADER_SIZE
            ..header.index * SAVE_SLOT_SIZE[0] + 0xA40,
        )
        .unwrap()
        .to_vec(),
    };

    // Key is save slot number: 0, 1, 2
    save_data_map.insert(header.slot_number, save_data);
  }

  save_data_map
}

fn has_camelot_header(raw: &[u8], slot_index: usize) -> bool {
  let Ok(header_string) = std::str::from_utf8(
    &raw[(slot_index * SAVE_SLOT_SIZE[0])..(slot_index * SAVE_SLOT_SIZE[0] + SLOT_NUMBER_OFFSET)],
  ) else {
    return false;
  };

  header_string.eq(HEADER_CAMELOT_ASCII_STRING)
}

/// 主要用于解析《开启的封印》的存档
pub fn parse_save_slots(raw: &[u8], include_non_clear: bool) -> Result<SaveSlots, SaveFileError> {
  check_save_file_size(raw.len() as u64)?;

  let first_slot_index = find_first_slot_index(raw).ok_or(SaveFileError::NoSaveData)?;
  let map = collect_slots(raw, first_slot_index, include_non_clear);
  if map.is_empty() {
    return Err(if include_non_clear {
      SaveFileError::NoSaveData
    } else {
      SaveFileError::NoClearData
    });
  }
  let first_available_slot = map.keys().min().copied();

  let mut slots = [None, None, None];
  for (slot_number, slot) in map {
    slots[slot_number as usize] = Some(slot);
  }

  Ok(SaveSlots {
    slots,
    first_available_slot,
  })
}

/// 解析可供`存档转换`的存档（两代都支持）
pub fn parse_convertible_save(raw: &[u8]) -> Result<GameInfo, SaveFileError> {
  check_save_file_size(raw.len() as u64)?;
  identify_game(raw).ok_or(SaveFileError::NoSaveData)
}

fn check_save_file_size(size: u64) -> Result<(), SaveFileError> {
  if VALID_SAVE_SIZES.contains(&size) {
    Ok(())
  } else {
    Err(SaveFileError::BadFileSize(size))
  }
}
