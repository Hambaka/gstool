use std::collections::HashMap;

use crate::core::{
  enums::{BuildDate, GameType, GameVersion},
  save::format::{
    BUILD_DATE_LOCATION_INDEX, CLEAR_DATA_IDENTIFIER_INDEX, CLEAR_DATA_IDENTIFIER_VALUE,
    FIRST_BUILD_DATE_LOCATION_INDEX, GS_BUILD_DATE, HEADER_CAMELOT_ASCII_STRING,
    HEADER_PRIORITY_LOCATION_INDEX, HEADER_SAVE_SLOT_NUMBER_LOCATION_INDEX, MAX_LOOP_COUNT,
    MAX_VALID_SLOT_NUMBER, SAVE_SLOT_SIZE,
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
  pub game_type: GameType,
  pub version: GameVersion,
  pub slot_index: usize,
}

impl GameInfo {
  pub fn new(game_type: GameType, version: GameVersion, slot_index: usize) -> Self {
    Self {
      game_type,
      version,
      slot_index,
    }
  }
}

pub struct SaveSlot {
  // Please note, the priority is in save header, this is only for checking valid header/save
  priority: u16,
  is_clear_data: bool,
  data: Vec<u8>,
}

impl SaveSlot {
  pub fn get_priority(&self) -> u16 {
    self.priority
  }

  pub fn get_data(&self) -> &[u8] {
    &self.data
  }

  pub fn get_is_clear(&self) -> bool {
    self.is_clear_data
  }

  fn set_priority(&mut self, val: u16) {
    self.priority = val;
  }

  fn set_data(&mut self, val: &[u8]) {
    self.data.clone_from_slice(val);
  }

  fn set_is_clear(&mut self, val: bool) {
    self.is_clear_data = val;
  }
}

// password
pub fn find_first_slot_index(raw_save_file: &[u8]) -> Option<usize> {
  for i in 0..MAX_LOOP_COUNT[0] {
    let Ok(header_string) = std::str::from_utf8(
      &raw_save_file
        [(i * SAVE_SLOT_SIZE[0])..(i * SAVE_SLOT_SIZE[0] + HEADER_SAVE_SLOT_NUMBER_LOCATION_INDEX)],
    ) else {
      continue;
    };
    if !header_string.eq(HEADER_CAMELOT_ASCII_STRING) {
      continue;
    }

    let build_date_from_raw_save = u16::from_le_bytes([
      raw_save_file[i * SAVE_SLOT_SIZE[0] + FIRST_BUILD_DATE_LOCATION_INDEX[0]],
      raw_save_file[i * SAVE_SLOT_SIZE[0] + FIRST_BUILD_DATE_LOCATION_INDEX[1]],
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
      &raw_save_file
        [(i * SAVE_SLOT_SIZE[0])..(i * SAVE_SLOT_SIZE[0] + HEADER_SAVE_SLOT_NUMBER_LOCATION_INDEX)],
    ) else {
      continue;
    };
    if !header_string.eq(HEADER_CAMELOT_ASCII_STRING) {
      continue;
    }

    let build_date = u16::from_le_bytes([
      raw_save_file[i * SAVE_SLOT_SIZE[0] + BUILD_DATE_LOCATION_INDEX[0][0][0]],
      raw_save_file[i * SAVE_SLOT_SIZE[0] + BUILD_DATE_LOCATION_INDEX[0][0][1]],
    ]);

    let game_version_option = GameVersion::from_build_date(BuildDate::Bytes(build_date));

    if game_version_option.is_none() {
      continue;
    } else {
      let game_version = game_version_option.unwrap();
      if GS_BUILD_DATE[0].contains(&build_date) {
        return Some(GameInfo::new(GameType::TheBrokenSeal, game_version, i));
      } else {
        return Some(GameInfo::new(GameType::TheLostAge, game_version, i / 3));
      }
    }
  }

  None
}

pub fn collect_slots(
  to_export_all_data: bool,
  raw_save_file: &[u8],
  loop_start_index: usize,
) -> HashMap<u8, SaveSlot> {
  let mut all_possible_headers = Vec::new();
  // "u8" is slot number. (Start from 0)
  let mut save_data_map: HashMap<u8, SaveSlot> = HashMap::new();
  for i in loop_start_index..MAX_LOOP_COUNT[0] {
    if raw_save_file[i * SAVE_SLOT_SIZE[0] + HEADER_SAVE_SLOT_NUMBER_LOCATION_INDEX]
      > MAX_VALID_SLOT_NUMBER
    {
      continue;
    }
    let is_clear_save = raw_save_file[i * SAVE_SLOT_SIZE[0] + CLEAR_DATA_IDENTIFIER_INDEX]
      == CLEAR_DATA_IDENTIFIER_VALUE;
    if !is_clear_save && !to_export_all_data {
      continue;
    }
    // Get all possible valid save headers first.
    all_possible_headers.push(HeaderInfo {
      index: i,
      priority: u16::from_le_bytes([
        raw_save_file[i * SAVE_SLOT_SIZE[0] + HEADER_PRIORITY_LOCATION_INDEX[0]],
        raw_save_file[i * SAVE_SLOT_SIZE[0] + HEADER_PRIORITY_LOCATION_INDEX[1]],
      ]),
      slot_number: raw_save_file[i * SAVE_SLOT_SIZE[0] + HEADER_SAVE_SLOT_NUMBER_LOCATION_INDEX],
      is_clear_data: is_clear_save,
    });
  }

  // Get all valid save data.
  for (j, header) in all_possible_headers.iter().enumerate() {
    if j > 0 && save_data_map.contains_key(&header.slot_number) {
      if let Some(existed_save_data) = save_data_map.get(&header.slot_number) {
        if header.priority > existed_save_data.get_priority() {
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
    save_data.set_data(
      raw_save_file
        .get(header.index * SAVE_SLOT_SIZE[0] + 0x10..header.index * SAVE_SLOT_SIZE[0] + 0xA40)
        .unwrap(),
    );
    save_data.set_is_clear(header.is_clear_data);
    save_data.set_priority(header.priority);
    // Key is save slot number: 0, 1, 2
    save_data_map.insert(header.slot_number, save_data);
  }

  save_data_map
}
