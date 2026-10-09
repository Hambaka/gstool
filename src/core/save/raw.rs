use std::collections::HashMap;

use crate::core::{
  enums::{GameRegion, GameTitle},
  save::format::{
    BUILD_DATE_OFFSET, CLEAR_DATA_IDENTIFIER_VALUE, CLEAR_DATA_OFFSET, FIRST_BUILD_DATE_OFFSET,
    GS_BUILD_DATE, HEADER_CAMELOT_ASCII_STRING, MAX_LOOP_COUNT, MAX_VALID_SLOT_INDEX,
    PRIORITY_OFFSET, SAVE_SLOT_SIZE, SLOT_HEADER_SIZE, SLOT_NUMBER_OFFSET,
  },
};

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
  pub title: GameTitle,
  pub region: GameRegion,
  pub slot_index: usize,
}

impl GameInfo {
  pub fn new(title: GameTitle, region: GameRegion, slot_index: usize) -> Self {
    Self {
      title,
      region,
      slot_index,
    }
  }
}

pub struct SaveSlot {
  // Please note, the priority is in save header, this is only for checking valid header/save
  pub priority: u16,
  pub is_clear_data: bool,
  pub data: Vec<u8>,
}

// password
pub fn find_first_slot_index(raw_save_file: &[u8]) -> Option<usize> {
  for i in 0..MAX_LOOP_COUNT[0] {
    let Ok(header_string) = std::str::from_utf8(
      &raw_save_file[(i * SAVE_SLOT_SIZE[0])..(i * SAVE_SLOT_SIZE[0] + SLOT_NUMBER_OFFSET)],
    ) else {
      continue;
    };
    if !header_string.eq(HEADER_CAMELOT_ASCII_STRING) {
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

// convert save
pub fn identify_game(raw_save_file: &[u8]) -> Option<GameInfo> {
  for i in 0..MAX_LOOP_COUNT[0] {
    let Ok(header_string) = std::str::from_utf8(
      &raw_save_file[(i * SAVE_SLOT_SIZE[0])..(i * SAVE_SLOT_SIZE[0] + SLOT_NUMBER_OFFSET)],
    ) else {
      continue;
    };

    if !header_string.eq(HEADER_CAMELOT_ASCII_STRING) {
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
      } else {
        return Some(GameInfo::new(GameTitle::TheLostAge, game_version, i / 3));
      }
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
    if j > 0 && save_data_map.contains_key(&header.slot_number) {
      if let Some(existed_save_data) = save_data_map.get(&header.slot_number) {
        if header.priority > existed_save_data.priority {
          save_data_map.remove(&header.slot_number);
        } else {
          continue;
        }
      }
    }
    /* Does not include first 16 bytes header.
    The data from 0xA40 (Felix, Jenna, Sheba, PC07) is useless for password generating.
    0xA40 - 0x10 = 0xA30 */
    let mut save_data = SaveSlot {
      priority: 0,
      is_clear_data: false,
      data: vec![0; 0xA30],
    };
    save_data.data = raw
      .get(
        header.index * SAVE_SLOT_SIZE[0] + SLOT_HEADER_SIZE
          ..header.index * SAVE_SLOT_SIZE[0] + 0xA40,
      )
      .unwrap()
      .to_vec();

    save_data.is_clear_data = header.is_clear_data;
    save_data.priority = header.priority;
    // Key is save slot number: 0, 1, 2
    save_data_map.insert(header.slot_number, save_data);
  }

  save_data_map
}
