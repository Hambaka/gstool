use crate::core::{
  enums::{GameType, GameVersion, NameType},
  save::format::{
    BUILD_DATE_LOCATION_INDEX, CHECKSUM_RANGE, GS_BUILD_DATE, HEADER_CHECKSUM_LOCATION_INDEX,
    HEADER_SAVE_SLOT_NUMBER_LOCATION_INDEX, MAX_LOOP_COUNT, MAX_VALID_SLOT_NUMBER,
    PARTY_MAIN_LEADER_INDEX, PARTY_MEMBERS_COUNT, PC_NAME, PC_NAME_LOCATION_INDEX, SAVE_SLOT_SIZE,
  },
};

pub fn convert_save_file(
  raw_save_file: &mut [u8],
  game_type: GameType,
  loop_start_index: usize,
  pc_name_type_option: Option<NameType>,
  build_date_type_option: Option<GameVersion>,
) -> Vec<u8> {
  let game_type_index = game_type as usize;

  for i in loop_start_index..MAX_LOOP_COUNT[game_type_index] {
    // Skip "invalid" save data.
    if raw_save_file[i * SAVE_SLOT_SIZE[game_type_index] + HEADER_SAVE_SLOT_NUMBER_LOCATION_INDEX]
      > MAX_VALID_SLOT_NUMBER
    {
      continue;
    }
    // Skip "invalid" save data.
    let build_date_from_raw_save = u16::from_le_bytes([
      raw_save_file
        [i * SAVE_SLOT_SIZE[game_type_index] + BUILD_DATE_LOCATION_INDEX[game_type_index][0][0]],
      raw_save_file
        [i * SAVE_SLOT_SIZE[game_type_index] + BUILD_DATE_LOCATION_INDEX[game_type_index][0][1]],
    ]);
    if !GS_BUILD_DATE[game_type_index].contains(&build_date_from_raw_save) {
      continue;
    }

    if let Some(pc_name_type) = pc_name_type_option {
      let pc_name_type_index = pc_name_type as usize;

      let mut is_main_leader = true;
      /* The party leader name.
      This name only shows in save select menu, it's different from the actual names for party members.
      The max size for this name is 12 bytes, though normally the max size we can use for character name is 10 bytes (Like those Kana in Japanese version)

      Robin (Isaac) is the only possible party leader in Golden Sun.
      But in Golden Sun: The Lost Age, there are two possible party leaders: Garcia (Felix) and Jasmine (Jenna).
      Garcia is the main leader in TLA. */
      if game_type_index == 1 {
        for j in 0..12 {
          // Compare the name to Garcia's name, to see if this name is same as main leader Garcia's name.
          if raw_save_file[i * SAVE_SLOT_SIZE[game_type_index] + 0x10 + j]
            != raw_save_file[i * SAVE_SLOT_SIZE[game_type_index]
              + PC_NAME_LOCATION_INDEX[game_type_index]
              + PARTY_MAIN_LEADER_INDEX[game_type_index] * 0x14C
              + j]
          {
            is_main_leader = false;
            break;
          }
        }
      }

      if is_main_leader {
        /* Golden Sun + is_main_leader(always true) -> Robin
        Golden Sun: The Lost Age + is_main_leader -> Garcia */
        for j in 0..12 {
          if j < 7 {
            raw_save_file[i * SAVE_SLOT_SIZE[game_type_index] + 0x10 + j] =
              PC_NAME[pc_name_type_index][PARTY_MAIN_LEADER_INDEX[game_type_index]][j];
          } else {
            raw_save_file[i * SAVE_SLOT_SIZE[game_type_index] + 0x10 + j] = 0x00;
          }
        }
      } else {
        // Golden Sun: The Lost Age + !is_main_leader -> Jasmine
        for j in 0..12 {
          if j < 7 {
            raw_save_file[i * SAVE_SLOT_SIZE[game_type_index] + 0x10 + j] =
              PC_NAME[pc_name_type_index][PARTY_MAIN_LEADER_INDEX[game_type_index] + 1][j];
          } else {
            raw_save_file[i * SAVE_SLOT_SIZE[game_type_index] + 0x10 + j] = 0x00;
          }
        }
      }

      // Change all party members' names.
      for j in 0..PARTY_MEMBERS_COUNT[game_type_index] {
        for k in 0..15 {
          if k < 7 {
            raw_save_file[i * SAVE_SLOT_SIZE[game_type_index]
              + PC_NAME_LOCATION_INDEX[game_type_index]
              + j * 0x14C
              + k] = PC_NAME[pc_name_type_index][j][k];
          } else {
            raw_save_file[i * SAVE_SLOT_SIZE[game_type_index]
              + PC_NAME_LOCATION_INDEX[game_type_index]
              + j * 0x14C
              + k] = 0x00;
          }
        }
      }
    }

    /* Change build date (to "convert" save data "version")
    If the build date in save data doesn't match the one in game,
    the game will force player to start from sanctum after loading save.
    Every language version has a different build date. */
    if let Some(build_date_type) = build_date_type_option {
      let build_date = GS_BUILD_DATE[game_type_index][build_date_type as usize].to_le_bytes();
      for j in 0..2 {
        raw_save_file[i * SAVE_SLOT_SIZE[game_type_index]
          + BUILD_DATE_LOCATION_INDEX[game_type_index][0][j]] = build_date[j];
        raw_save_file[i * SAVE_SLOT_SIZE[game_type_index]
          + BUILD_DATE_LOCATION_INDEX[game_type_index][1][j]] = build_date[j];
        raw_save_file[i * SAVE_SLOT_SIZE[game_type_index]
          + BUILD_DATE_LOCATION_INDEX[game_type_index][2][j]] = build_date[j];
      }
    }

    /* Calculate save's checksum (16 bit, doesn't need to calculate each save's first 0x10 header),
    if the checksum of save data doesn't match the one in header,
    the game will not consider it as a valid save.
    If the checksum exceeds 4 digits(Hexadecimal, not decimal), just discard extra digits. */
    let mut checksum = 0;
    for j in 0..CHECKSUM_RANGE[game_type_index] {
      checksum += u32::from(raw_save_file[i * SAVE_SLOT_SIZE[game_type_index] + 0x10 + j]);
    }
    let checksum_bytes = checksum.to_le_bytes();
    for j in 0..2 {
      raw_save_file[i * SAVE_SLOT_SIZE[game_type_index] + HEADER_CHECKSUM_LOCATION_INDEX[j]] =
        checksum_bytes[j];
    }
  }

  Vec::from(raw_save_file)
}
