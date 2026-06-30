use crate::core::data::{ALL_DJINN_NAMES, DYRATI_ITEM_NAMES, Language};

use super::enums::{GameType, GameVersion, NameType, PasswordGrade};
use std::{collections::HashMap, fmt::Write as _};

/* Links to other Golden Sun reference guide (save editing):
https://gamefaqs.gamespot.com/gba/468548-golden-sun/faqs/43776
https://gamefaqs.gamespot.com/gba/561356-golden-sun-the-lost-age/faqs/30811
----------------------------------------------------------------------------------
More reference info/comment about GBA Golden Sun series save file from Dyrati (in "Obababot")

https://github.com/Dyrati/obababot/blob/main/obababot/gsfuncs.py
At line 579, the "get_save_data" function takes raw binary .sav data and returns individual save slots with all of the info from each valid save.
The function checks the file at 0x1000 byte intervals.

The first 16 bytes of each interval (the header) are organized as follows:
- 7 bytes for the ASCII string "CAMELOT"
- 1 byte for the slot number
- 2 bytes for a checksum
- 2 bytes for a priority number
- 4 bytes of garbage data

A header is valid if the first 7 bytes spell "CAMELOT", and the slot number is less than 16.
In the case where multiple headers have the same slot number, use the header with the highest priority number.
That should leave you with up to 3 valid headers.
The next 0x2FF0 bytes after the header constitute the save data for that file. (Note: GS2 only)
----------------------------------------------------------------------------------
Additional reference info/comment about the first Golden Sun save file from Dyrati

For GS1, each save splits into two parts.
In the .sav file, each section is 0x1000 bytes long.
However two separate sections are joined together to create one save file.
Some sections have slot numbers of 3, 4, or 5,
those sections are the second half of slots 0, 1, and 2 respectively.
But seems the second half of the save doesn't store the data for generating password.
----------------------------------------------------------------------------------
Golden Sun password generating explanation document by Dyrati
Source: https://www.reddit.com/r/GoldenSun/comments/jon3h7/golden_sun_password_tools/
        https://docs.google.com/spreadsheets/d/1jQ2Zj2F57Fb4hs0pDYLCaZL-qry7gcVxNbhpq0BJDUs/

Step 1: Convert relevant data into fixed point binary

Levels: 7 bits for Isaac, Garet, Ivan, Mia
Djinn: 7 bits for Venus, Mercury, Mars, Jupiter
Events: 1 bit for Hammet, Colosso, Hsu, Deadbeard, Vale, Vault
Psy Items: 1 bit for each item in the "PsyItems" list (1 if in party inventory, 0 if not)
Coins: 24 bits
Stats: 60 bits per character, arranged as follows:
  11 bits HP, 11 bits PP, 10 bits ATK, 10 bits DEF, 10 bits AGI, 8 bits LCK
Item IDs: 9 bits per item, 15 items per character
Item Amounts: 5 bits for every item in the "Stackables" list (whether or not item is in inventory), for each character. Subtract 1 from in-game amount

Step 2: Arrange bits

Syntax
  name = value (assign value to name)
  & (concatenate)
  name[a:b] (bits of name, from left to right, inclusive, starts at 1)
  rep(name, x) (name repeated x times)

Arrangement
  levelbits = mia_level & ivan_level & garet_level & isaac_level
  levelbits = levelbits[21:28] & levelbits[13:20] & levelbits[5:12] & levelbits[1:4]
  djinnbits = jupiter_djinn & mars_djinn & mercury_djinn & venus_djinn
  djinnbits = djinnbits[25:28] & djinnbits[17:24] & djinnbits[9:16] & djinnbits[1:8]
  eventbits = 0 & 0 & vault_bit & vale_bit & deadbeard_bit & hsu_bit & colosso_bit & hammet_bit
  psyitems = catch_beads & carry_stone & cloak_ball & halt_gem & lifting_gem & frost_jewel & douse_drop & orb_of_force
  [character]_stats = hp & pp & atk & def & agi & lck
  stats = isaac_stats & garet_stats & ivan_stats & mia_stats
  [character]_item_ids = item1_id & item2_id & … & item15_id
  item_ids = isaac_item_ids & garet_item_ids & ivan_item_ids & mia_item_ids
  Every 63 bits (7 items), insert a 0 into item_ids
  [character]_item_amounts = herb_count & nut_count & vial_count & … & crystalpowder_count
  item_amounts = isaac_item_amounts & garet_item_amounts & ivan_item_amounts & mia_item_amounts

  gold_bits = levelbits & djinnbits & eventbits & stats & rep(0,8) & item_ids & item_amounts & coins & rep(0,40)
  silver_bits = levelbits & djinnbits & eventbits & psyitems & stats
  bronze_bits = levelbits & djinnbits & eventbits & psyitems

Step 3: Generate key from bits

  key = 0xFFFF (16 bit register, meaning that bits exceeding bit 15 are dropped)
  for byte in bits:
    key = byte*256 xor key
    for bit in key:
        key = key << 1 (bitshift left)
        if previous operation carried out a 1: key = key + 0xEFDF

  key = not(key)

Step 4: XOR bits with key
  byte1 = key[1:8]
  byte2 = key[9:16]
  append byte1 to bits
  xor every byte in bits with byte2
  append byte2 to bits

Step 5: Divide bits into 6 bit groups.  After every 9 groups, insert a group which is their sum modulo 2^6

Step 6: Add each group's position to itself, and take result modulo 2^6
  group_0 = (group_0 + 0) mod 2^6
  group_1 = (group_1 + 1) mod 2^6
  …
  group_n = (group_n + n) mod 2^6

Step 7: Substitute groups for characters in the "Chars" list

See the code at https://github.com/Dyrati/Golden-Sun-Password-Transfer/blob/main/password.lua */

/// 7 bytes for the ASCII string "CAMELOT" in each save's header.
const HEADER_CAMELOT_ASCII_STRING: &str = "CAMELOT";

/// Golden Sun/Golden Sun: The Lost Age build date
/// Source: Golden Sun Hacking Community Discord Server
///
/// Value: 0x1000 | 1024 * year + 64 * month + day
/// Note: Year is "1", "2", not "2001", "2002"
///
/// GS1 (J) = 0x159C -> 2001/06/28
/// GS1 (U) = 0x1652 -> 2001/09/18
/// GS1 (G) = 0x1849 -> 2002/01/09
/// GS1 (S) = 0x1885 -> 2002/02/05
/// GS1 (F) = 0x1713 -> 2001/12/19
/// GS1 (I) = 0x1886 -> 2002/02/06
///
/// GS2 (J) = 0x198A -> 2002/06/10
/// GS2 (U) = 0x1C85 -> 2003/02/05
/// GS2 (G) = 0x1D97 -> 2003/06/23
/// GS2 (S) = 0x1DC7 -> 2003/07/07
/// GS2 (F) = 0x1D98 -> 2003/06/24
/// GS2 (I) = 0x1DC8 -> 2003/07/08
#[rustfmt::skip]
const GS_BUILD_DATE: [[u16; 6]; 2] = [
  [0x159C, 0x1652, 0x1849, 0x1885, 0x1713, 0x1886],
  [0x198A, 0x1C85, 0x1D97, 0x1DC7, 0x1D98, 0x1DC8],
];

/// Main characters' default names in different languages.
///
/// Japanese:                                   "ロビン", "ジェラルド", "イワン", "メアリィ", "ガルシア", "ジャスミン", "シバ",  "ピカード"
/// English:                                    "Isaac",  "Garet",      "Ivan",   "Mia",      "Felix",    "Jenna",      "Sheba", "Piers"
/// German:                                     "Isaac",  "Garet",      "Ivan",   "Mia",      "Felix",    "Jenna",      "Cosma", "Aaron"
/// Spanish:                                    "Hans",   "Garet",      "Iván",   "Mia",      "Félix",    "Nadia",      "Sole",  "Piers"
/// French:                                     "Vlad",   "Garet",      "Ivan",   "Sofia",    "Pavel",    "Lina",       "Cylia", "Piers"
/// Italian:                                    "Isaac",  "Garet",      "Ivan",   "Mia",      "Felix",    "Jenna",      "Sara",  "Piers"
/// Chinese fan translation by Mobile/Soma Team for GS2 (SC and TC share same encoding)
/// ├── Simplified Chinese:                     "罗宾",   "杰拉德",     "伊万",   "米雅莉",   "加西亚",   "加斯敏",     "西芭",  "皮卡德"
/// └── Traditional Chinese:                    "羅賓",   "傑拉德",     "伊萬",   "米雅莉",   "加西亞",   "加斯敏",     "西芭",  "皮卡德"
/// Chinese fan translation by 2023 Team for GS1
/// └── Simplified Chinese:                     "罗宾",   "杰拉德",     "伊万",   "梅雅莉",   "加西亚",   "加斯敏",     "西芭",  "皮卡德"
#[rustfmt::skip]
const PC_NAME: [[[u8; 7]; 8]; 8] = [
  [[0xDB, 0xCB, 0xDE, 0xDD, 0x00, 0x00, 0x00], [0xBC, 0xDE, 0xAA, 0xD7, 0xD9, 0xC4, 0xDE], [0xB2, 0xDC, 0xDD, 0x00, 0x00, 0x00, 0x00], [0xD2, 0xB1, 0xD8, 0xA8, 0x00, 0x00, 0x00], [0xB6, 0xDE, 0xD9, 0xBC, 0xB1, 0x00, 0x00], [0xBC, 0xDE, 0xAC, 0xBD, 0xD0, 0xDD, 0x00], [0xBC, 0xCA, 0xDE, 0x00, 0x00, 0x00, 0x00], [0xCB, 0xDF, 0xB6, 0xB0, 0xC4, 0xDE, 0x00]],
  [[0x49, 0x73, 0x61, 0x61, 0x63, 0x00, 0x00], [0x47, 0x61, 0x72, 0x65, 0x74, 0x00, 0x00], [0x49, 0x76, 0x61, 0x6E, 0x00, 0x00, 0x00], [0x4D, 0x69, 0x61, 0x00, 0x00, 0x00, 0x00], [0x46, 0x65, 0x6C, 0x69, 0x78, 0x00, 0x00], [0x4A, 0x65, 0x6E, 0x6E, 0x61, 0x00, 0x00], [0x53, 0x68, 0x65, 0x62, 0x61, 0x00, 0x00], [0x50, 0x69, 0x65, 0x72, 0x73, 0x00, 0x00]],
  [[0x49, 0x73, 0x61, 0x61, 0x63, 0x00, 0x00], [0x47, 0x61, 0x72, 0x65, 0x74, 0x00, 0x00], [0x49, 0x76, 0x61, 0x6E, 0x00, 0x00, 0x00], [0x4D, 0x69, 0x61, 0x00, 0x00, 0x00, 0x00], [0x46, 0x65, 0x6C, 0x69, 0x78, 0x00, 0x00], [0x4A, 0x65, 0x6E, 0x6E, 0x61, 0x00, 0x00], [0x43, 0x6F, 0x73, 0x6D, 0x61, 0x00, 0x00], [0x41, 0x61, 0x72, 0x6F, 0x6E, 0x00, 0x00]],
  [[0x48, 0x61, 0x6E, 0x73, 0x00, 0x00, 0x00], [0x47, 0x61, 0x72, 0x65, 0x74, 0x00, 0x00], [0x49, 0x76, 0xE1, 0x6E, 0x00, 0x00, 0x00], [0x4D, 0x69, 0x61, 0x00, 0x00, 0x00, 0x00], [0x46, 0xE9, 0x6C, 0x69, 0x78, 0x00, 0x00], [0x4E, 0x61, 0x64, 0x69, 0x61, 0x00, 0x00], [0x53, 0x6F, 0x6C, 0x65, 0x00, 0x00, 0x00], [0x50, 0x69, 0x65, 0x72, 0x73, 0x00, 0x00]],
  [[0x56, 0x6C, 0x61, 0x64, 0x00, 0x00, 0x00], [0x47, 0x61, 0x72, 0x65, 0x74, 0x00, 0x00], [0x49, 0x76, 0x61, 0x6E, 0x00, 0x00, 0x00], [0x53, 0x6F, 0x66, 0x69, 0x61, 0x00, 0x00], [0x50, 0x61, 0x76, 0x65, 0x6C, 0x00, 0x00], [0x4C, 0x69, 0x6E, 0x61, 0x00, 0x00, 0x00], [0x43, 0x79, 0x6C, 0x69, 0x61, 0x00, 0x00], [0x50, 0x69, 0x65, 0x72, 0x73, 0x00, 0x00]],
  [[0x49, 0x73, 0x61, 0x61, 0x63, 0x00, 0x00], [0x47, 0x61, 0x72, 0x65, 0x74, 0x00, 0x00], [0x49, 0x76, 0x61, 0x6E, 0x00, 0x00, 0x00], [0x4D, 0x69, 0x61, 0x00, 0x00, 0x00, 0x00], [0x46, 0x65, 0x6C, 0x69, 0x78, 0x00, 0x00], [0x4A, 0x65, 0x6E, 0x6E, 0x61, 0x00, 0x00], [0x53, 0x61, 0x72, 0x61, 0x00, 0x00, 0x00], [0x50, 0x69, 0x65, 0x72, 0x73, 0x00, 0x00]],
  [[0x3F, 0x05, 0x81, 0x01, 0x00, 0x00, 0x00], [0x39, 0x04, 0xC1, 0x04, 0x4E, 0x02, 0x00], [0xDC, 0x08, 0xCF, 0x07, 0x00, 0x00, 0x00], [0x7A, 0x05, 0xA4, 0x08, 0xE7, 0x04, 0x00], [0xFC, 0x03, 0x14, 0x08, 0xA6, 0x08, 0x00], [0xFC, 0x03, 0x37, 0x07, 0x8B, 0x05, 0x00], [0x14, 0x08, 0x24, 0x01, 0x00, 0x00, 0x00], [0x07, 0x06, 0x8A, 0x04, 0x4E, 0x02, 0x00]],
  [[0x25, 0x23, 0x26, 0x23, 0x00, 0x00, 0x00], [0x27, 0x23, 0x28, 0x23, 0x29, 0x23, 0x00], [0x2A, 0x23, 0x2B, 0x23, 0x00, 0x00, 0x00], [0x2C, 0x23, 0x2D, 0x23, 0x2E, 0x23, 0x00], [0x2F, 0x23, 0x30, 0x23, 0x31, 0x23, 0x00], [0x2F, 0x23, 0x32, 0x23, 0x33, 0x23, 0x00], [0x30, 0x23, 0x34, 0x23, 0x00, 0x00, 0x00], [0x76, 0x40, 0x62, 0x24, 0x29, 0x23, 0x00]]
];

/// For TBS, the size of each save slot is 4KB.
/// For TLA, the size of each save slot is 12KB.
const SAVE_SLOT_SIZE: [usize; 2] = [0x1000, 0x3000];

/// TBS: 64KB / 4KB = 16
/// TLA: 64KB / 12KB = 5
const MAX_LOOP_COUNT: [usize; 2] = [16, 5];

/// All build date locations: 0x36 to 0x37, 0x250 to 0x251, 0x508 to 0x509
/// All stored values are same.
const FIRST_BUILD_DATE_LOCATION_INDEX: [usize; 2] = [0x36, 0x37];

/* Unused, since now we have a much better way to check if the save data is clear data.

All save map(location) locations  :0x410 to 0x413, 0x418 to 0x41B
Save map data -> Main:Minor
All stored values are same.
const FIRST_SAVE_MAP_MAIN_LOCATION_INDEX: [usize; 2] = [0x410, 0x411];
const FIRST_SAVE_MAP_MINOR_LOCATION_INDEX: [usize; 2] = [0x412, 0x413];  */

/* Unused, since now we have a much better way to check if the save data is clear data.

Clear data's save location value is -> 1:2
const CLEAR_DATA_SAVE_MAP_VALUE: [u16; 2] = [0x01, 0x02]; */

/// For clear data, its value is 1.
/// If the save data is not a clear data, its value is 0.
const CLEAR_DATA_IDENTIFIER_VALUE: u8 = 0x01;

/// Location of `CLEAR_DATA_IDENTIFIER_VALUE` in save file.
const CLEAR_DATA_IDENTIFIER_INDEX: usize = 0x31;

/// The 8th byte in save header is the slot number, it only show 3 active save data in game.
const HEADER_SAVE_SLOT_NUMBER_LOCATION_INDEX: usize = 0x07;

/// The values for 3 'active' save data are: 0x00, 0x01 and 0x02.
/// For more information, please see the comment at the beginning of the code.
const MAX_VALID_SLOT_NUMBER: u8 = 0x02;

/// Location of save data's priority
/// For more information, please see the comment at the beginning of the code.
const HEADER_PRIORITY_LOCATION_INDEX: [usize; 2] = [0x0A, 0x0B];

/// Valid psynergy items in TBS
///
/// C8: Orb of Force
/// C9: Douse Drop
/// CA: Frost Jewel
/// CB: Lifting Gem
/// CC: Halt Gem
/// CD: Cloak Ball
/// CE: Carry Stone
/// CF: Catch Beads
pub const PSYNERGY_ITEMS: [u16; 8] = [0xC8, 0xC9, 0xCA, 0xCB, 0xCC, 0xCD, 0xCE, 0xCF];

/// Valid stackable items in TBS
///
/// B4: Herb
/// B5: Nut
/// B6: Vial
/// B7: Potion
/// BA: Psy Crystal
/// BB: Antidote
/// BC: Elixir
/// BD: Water of Life
/// BF: Power Bread
/// C0: Cookie
/// C1: Apple
/// C2: Hard Nut
/// C3: Mint
/// C4: Lucky Pepper
/// E2: Smoke Bomb
/// E3: Sleep Bomb
/// E4: Game Ticket
/// E5: Lucky Medal
/// EC: Sacred Feather
/// EE: Oil Drop
/// EF: Weasel's Claw
/// F0: Bramble Seed
/// F1: Crystal Powder
pub const STACKABLE_ITEMS: [u16; 23] = [
  0xB4, 0xB5, 0xB6, 0xB7, 0xBA, 0xBB, 0xBC, 0xBD, 0xBF, 0xC0, 0xC1, 0xC2, 0xC3, 0xC4, 0xE2, 0xE3,
  0xE4, 0xE5, 0xEC, 0xEE, 0xEF, 0xF0, 0xF1,
];

/// Bronze: 9
/// Silver: 9, 19, 29, 39, 49, 59
/// Gold  : 9, 19, 29, 39, 49, 59, 69, 79, 89, 99, 109, 119, 129, 139, 149, 159, 169, 179, 189, 199, 209, 219, 229, 239, 249, 259
///
/// The values we want :
/// 9 - 0, 19 - 1, 29 - 2, 39 - 3, 49 - 4,
/// 59 - 5, 69 - 6, 79 - 7, 89 - 8, 99 - 9,
/// 109 - 10, 119 - 11, 129 - 12, 139 - 13, 149 - 14,
/// 159 - 15, 169 - 16, 179 - 17, 189 - 18, 199 - 19,
/// 209 - 20, 219 - 21, 229 - 22, 239 - 23, 249 - 24,
/// 259 - 25
const CHECKSUM_INDEX: [usize; 26] = [
  9, 18, 27, 36, 45, 54, 63, 72, 81, 90, 99, 108, 117, 126, 135, 144, 153, 162, 171, 180, 189, 198,
  207, 216, 225, 234,
];

/// These default classes are:
/// Squire (Isaac), Guard (Garet), Wind Seer (Ivan), Water Seer (Mia)
#[rustfmt::skip]
const DEFAULT_CLASS_MULTIPLIERS: [[f64; 6]; 4] = [
  [1.1, 0.8, 1.1, 1.0, 1.1, 1.0],
  [1.1, 0.8, 1.0, 1.1, 0.7, 1.0],
  [0.8, 1.4, 0.8, 0.9, 1.3, 1.1],
  [0.9, 1.3, 0.9, 1.0, 0.8, 1.3],
];

/// Details on the assword-making procedure - by Paulygon (Golden Sun Password Generator, Version 0.85, March 22, 2004)
/// 
/// "Information validation: The first thing done is to make sure that certain data, like stats, are capped to certain limits.
/// 1999 for HP and PP; 999 for Attack, Defense, and Agility; 99 for Luck.
/// Character class or other factors may limit what is shown during the game to a lower number.
/// 
/// You may note that in the program, you can't use 999 for attack, defense, or agility.
/// The reason for this is that although the password can store this information,
/// The Lost Age doesn't seem to acknowledge anything higher than 511.
/// For those more technically inclined, this means that the highest bit is ignored and treated as 0.
/// Rather surprising, but oh well... :-}"
#[rustfmt::skip]
pub const MAX_VALID_BASE_STATS_VALUES_PWD: [u16; 6] = [
  // 11 bits, 0111 1111 1111, 2047 -> 1999
  2047,
  2047,
  // 10 bits, 0011 1111 1111, 1023 -> 999 -> 511
  // 0001 1111 1111
  511,
  511,
  511,
  // 8 bits, 1111 1111, 255 -> 127 -> 99
  127,
];

#[rustfmt::skip]
const MAX_VALID_IN_GAME_STATS_VALUES: [u16; 6] = [
  1999,
  1999,
  999,
  999,
  999,
  99,
];

/// In TBS, it should be 0 -> Robin (Isaac)
/// In TLA, it should be 4 -> Garcia (Felix)
const PARTY_MAIN_LEADER_INDEX: [usize; 2] = [0, 4];

/// In TBS, we only have four available members: Robin (Isaac), Gerald (Garet), Ivan and Mary/Mearī (Mia).
/// We should also include Garcia (Felix), Jasmine (Jenna) and Shiba (Sheba).
/// In TLA, well, We have Picard (Piers) in party now.
const PARTY_MEMBERS_COUNT: [usize; 2] = [7, 8];
const PC_NAME_LOCATION_INDEX: [usize; 2] = [0x510, 0x530];
const BUILD_DATE_LOCATION_INDEX: [[[usize; 2]; 3]; 2] = [
  [[0x36, 0x37], [0x250, 0x251], [0x508, 0x509]],
  [[0x36, 0x37], [0x250, 0x251], [0x528, 0x529]],
];

/// Save slot size - header size
/// Header size is 0x10.
/// TBS: 0x1000 - 0x10
/// TLA: 0x3000 - 0x10
const CHECKSUM_RANGE: [usize; 2] = [0xFF0, 0x2FF0];
const HEADER_CHECKSUM_LOCATION_INDEX: [usize; 2] = [0x08, 0x09];

struct HeaderInfo {
  // Range: 0 <= x < 16
  index: usize,
  priority: u16,
  // Start from 0
  slot_number: u8,
  // Please note, the value of this is not in save header, this is only for checking valid header/save
  is_clear_data: bool,
}

pub struct RawSaveData {
  // Please note, the priority is in save header, this is only for checking valid header/save
  priority: u16,
  is_clear_data: bool,
  data: Vec<u8>,
}

impl RawSaveData {
  pub fn get_priority(&self) -> &u16 {
    &self.priority
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

#[derive(Debug)]
pub struct SaveData {
  levels: [u8; 4],
  djinn: [u32; 4],
  events: [u8; 6],
  stats: [[u16; 6]; 4],
  items: [[u16; 15]; 4],
  coins: u32,
}

#[derive(Debug, Clone)]
pub struct PasswordInfo {
  levels: [u8; 4],
  djinn: [[bool; 7]; 4],
  events: [bool; 6],
  base_stats: [[u16; 6]; 4],
  stats: [[u16; 6]; 4],
  psy_items: [bool; 8],
  items: [[(u16, u8); 15]; 4],
  coins: u32,
}

impl PasswordInfo {
  pub fn get_levels(&self) -> [u8; 4] {
    self.levels
  }

  pub fn get_djinn(&self) -> [[bool; 7]; 4] {
    self.djinn
  }

  pub fn get_events(&self) -> [bool; 6] {
    self.events
  }

  pub fn get_base_stats(&self) -> [[u16; 6]; 4] {
    self.base_stats
  }

  pub fn get_stats(&self) -> [[u16; 6]; 4] {
    self.stats
  }

  pub fn get_psy_items(&self) -> [bool; 8] {
    self.psy_items
  }

  pub fn get_items(&self) -> [[(u16, u8); 15]; 4] {
    self.items
  }

  pub fn get_coins(&self) -> u32 {
    self.coins
  }
}

// Port from Dyrati's "Golden-Sun-Password-Transfer" lua script for "vba-rr" and "Bizhawk" emulators.
pub struct BitArray {
  bits: Vec<u8>,
}

// Port from Dyrati's "Golden-Sun-Password-Transfer" lua script for "vba-rr" and "Bizhawk" emulators.
impl Default for BitArray {
  fn default() -> Self {
    Self::new()
  }
}

impl BitArray {
  pub fn new() -> Self {
    BitArray { bits: Vec::new() }
  }

  pub fn from(value: &[u8]) -> Self {
    BitArray {
      bits: Vec::from(value),
    }
  }

  pub fn get_bits(&self) -> Vec<u8> {
    self.bits.clone()
  }

  // Link: https://github.com/Dyrati/Golden-Sun-Password-Transfer/blob/5ec2d52553ec8f4e0fe77854bc2b31956ac09a11/password.lua#L16
  fn push_bit(&mut self, value: u32) {
    self.bits.push((value & 1) as u8);
  }

  // Link: https://github.com/Dyrati/Golden-Sun-Password-Transfer/blob/5ec2d52553ec8f4e0fe77854bc2b31956ac09a11/password.lua#L16
  fn push_bits(&mut self, value: u32, size: usize) {
    for i in 0..size {
      self.bits.push(((value >> (size - i - 1)) & 1) as u8);
    }
  }

  // Link: https://github.com/Dyrati/Golden-Sun-Password-Transfer/blob/5ec2d52553ec8f4e0fe77854bc2b31956ac09a11/password.lua#L16
  fn replace_bits(&mut self, value: u32, size: usize, pos: usize) {
    for i in 0..size {
      self.bits[pos + i] = ((value >> (size - i - 1)) & 1) as u8;
    }
  }

  // Link: https://github.com/Dyrati/Golden-Sun-Password-Transfer/blob/5ec2d52553ec8f4e0fe77854bc2b31956ac09a11/password.lua#L23
  fn sub_bits_u8(&mut self, min: usize, max: usize) -> u8 {
    let mut acc = 0;
    for i in 0..=(max - min) {
      if min < self.bits.len() {
        acc = 2 * acc + self.bits[min + i];
      } else {
        acc *= 2;
      }
    }
    acc
  }

  // Link: https://github.com/Dyrati/Golden-Sun-Password-Transfer/blob/5ec2d52553ec8f4e0fe77854bc2b31956ac09a11/password.lua#L23
  fn sub_bits_u16(&mut self, min: usize, max: usize) -> u16 {
    let mut acc = 0;
    for i in 0..=(max - min) {
      if min < self.bits.len() {
        acc = 2 * acc + u16::from(self.bits[min + i]);
      } else {
        acc *= 2;
      }
    }
    acc
  }

  // Link: https://github.com/Dyrati/Golden-Sun-Password-Transfer/blob/5ec2d52553ec8f4e0fe77854bc2b31956ac09a11/password.lua#L23
  fn sub_bits_u32(&mut self, min: usize, max: usize) -> u32 {
    let mut acc = 0;
    for i in 0..=(max - min) {
      if min < self.bits.len() {
        acc = 2 * acc + u32::from(self.bits[min + i]);
      } else {
        acc *= 2;
      }
    }
    acc
  }

  fn get_len(&self) -> usize {
    self.bits.len()
  }
}

// password
pub fn get_loop_start_index_option(raw_save_file: &[u8]) -> Option<usize> {
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
pub fn get_game_type_with_loop_start_index_option(
  raw_save_file: &[u8],
) -> Option<(GameType, GameVersion, usize)> {
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
    if GS_BUILD_DATE[0].contains(&build_date) {
      let game_version = match build_date {
        0x159C => GameVersion::Japan,
        0x1652 => GameVersion::USA,
        0x1849 => GameVersion::Germany,
        0x1885 => GameVersion::Spain,
        0x1713 => GameVersion::France,
        0x1886 => GameVersion::Italy,
        _ => unreachable!(),
      };
      return Some((GameType::TheBrokenSeal, game_version, i));
    }
    if GS_BUILD_DATE[1].contains(&build_date) {
      let game_version = match build_date {
        0x198A => GameVersion::Japan,
        0x1C85 => GameVersion::USA,
        0x1D97 => GameVersion::Germany,
        0x1DC7 => GameVersion::Spain,
        0x1D98 => GameVersion::France,
        0x1DC8 => GameVersion::Italy,
        _ => unreachable!(),
      };
      return Some((GameType::TheLostAge, game_version, i / 3));
    }
  }

  None
}

pub fn get_raw_save_data_map(
  to_export_all_data: bool,
  raw_save_file: &[u8],
  loop_start_index: usize,
) -> HashMap<u8, RawSaveData> {
  let mut all_possible_headers = Vec::new();
  // "u8" is slot number. (Start from 0)
  let mut save_data_map: HashMap<u8, RawSaveData> = HashMap::new();
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
        if header.priority > *existed_save_data.get_priority() {
          save_data_map.remove(&header.slot_number);
        } else {
          continue;
        }
      }
    }
    /* Does not include first 16 bytes header.
    The data from 0xA40 (Felix, Jenna, Sheba, PC07) is useless for password generating.
    0xA40 - 0x10 = 0xA30 */
    let mut save_data = RawSaveData {
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

/* Port from Dyrati's "Golden-Sun-Password-Transfer" lua script for "vba-rr" and "Bizhawk" emulators.
Link: https://github.com/Dyrati/Golden-Sun-Password-Transfer/blob/5ec2d52553ec8f4e0fe77854bc2b31956ac09a11/password.lua#L8 */
fn get_event_flag(raw_save: &[u8], flag: u32) -> u8 {
  let byte_pos = (flag >> 3) as usize;
  let bit_pos = flag & 7;
  (raw_save[0x40 + byte_pos] >> bit_pos) & 1
}

/* Port from Dyrati's "Golden-Sun-Password-Transfer" lua script for "vba-rr" and "Bizhawk" emulators.
Link: https://github.com/Dyrati/Golden-Sun-Password-Transfer/blob/5ec2d52553ec8f4e0fe77854bc2b31956ac09a11/password.lua#L35 */
fn gen_save_data_with_raw_save_bytes(raw_save: &[u8]) -> SaveData {
  // [u8; 4]
  let mut levels = [0; 4];
  // [u32; 4]
  // All: [0x7F, 0x7F, 0x7F, 0x7F]
  let mut djinn = [0; 4];
  // [u8; 6]
  // All: [1, 1, 0, 1, 1, 1]
  let mut events = [0; 6];
  // [[u16; 6]; 4]
  let mut stats = [[0; 6]; 4];
  // [[u16; 15]; 4]
  let mut items = [[0; 15]; 4];

  for i in 0..4 {
    let base = 0x500 + 0x14C * i;
    levels[i] = raw_save[base + 0xF];

    for j in 0..4 {
      djinn[j] |= u32::from_le_bytes([
        raw_save[base + 0xF8 + 4 * j],
        raw_save[base + 0xF9 + 4 * j],
        raw_save[base + 0xFA + 4 * j],
        raw_save[base + 0xFB + 4 * j],
      ]);
    }

    // HP
    stats[i][0] = u16::from_le_bytes([raw_save[base + 0x10], raw_save[base + 0x11]]);
    // EP
    stats[i][1] = u16::from_le_bytes([raw_save[base + 0x12], raw_save[base + 0x13]]);
    // Attack
    stats[i][2] = u16::from_le_bytes([raw_save[base + 0x18], raw_save[base + 0x19]]);
    // Defense
    stats[i][3] = u16::from_le_bytes([raw_save[base + 0x1A], raw_save[base + 0x1B]]);
    // Agility
    stats[i][4] = u16::from_le_bytes([raw_save[base + 0x1C], raw_save[base + 0x1D]]);
    // Luck
    stats[i][5] = u16::from(raw_save[base + 0x1E]);

    for j in 0..15 {
      items[i][j] =
        u16::from_le_bytes([raw_save[base + 0xD8 + 2 * j], raw_save[base + 0xD9 + 2 * j]]);
    }
  }
  // Saved Hametto/Hammet
  events[0] = get_event_flag(raw_save, 0x941);
  // Beat Colosso (Won the final fight against Navampa)
  events[1] = get_event_flag(raw_save, 0x951);
  // Ulmuch/Hsu was rescued by Hamo/Hama and Kouran/Feizhi alone, and Robin/Isaac's party didn't lend a hand
  events[2] = get_event_flag(raw_save, 0x8B3);
  // Beat Talos/Deadbeard
  events[3] = get_event_flag(raw_save, 0x8D1);
  // Visited Haidia/Vale after entering Kalay (Return to Haidia/Vale and find out that Robin/Isaac's mother's sick.)
  events[4] = get_event_flag(raw_save, 0x81E);
  // Visited Coopup/Vault after entering Kalay (Return to Coopup/Vault and talk with the mayor about the thieves, and find out that thieves have fled Coopup/Vault.)
  events[5] = get_event_flag(raw_save, 0x868);

  // u32
  let coins = u32::from_le_bytes([
    raw_save[0x250],
    raw_save[0x251],
    raw_save[0x252],
    raw_save[0x253],
  ]);
  SaveData {
    levels,
    djinn,
    events,
    stats,
    items,
    coins,
  }
}

/* Port from Dyrati's "Golden-Sun-Password-Transfer" lua script for "vba-rr" and "Bizhawk" emulators.
Link: https://github.com/Dyrati/Golden-Sun-Password-Transfer/blob/5ec2d52553ec8f4e0fe77854bc2b31956ac09a11/password.lua#L79 */
fn gen_password_bytes_with_save_data(grade: PasswordGrade, save_data: &SaveData) -> Vec<u8> {
  let mut bits = BitArray::new();

  // Insert 7 bits per level, 7 bits per jinn element
  let mut level_bits = BitArray::new();
  let mut djinn_bits = BitArray::new();
  for i in (0..=3).rev() {
    level_bits.push_bits(u32::from(save_data.levels[i]), 7);
  }

  for i in (0..=3).rev() {
    djinn_bits.push_bits(save_data.djinn[i], 7);
  }

  for i in (11..=27).rev().step_by(8) {
    bits.push_bits(u32::from(level_bits.sub_bits_u8(i - 7, i)), 8);
  }

  bits.push_bits(u32::from(level_bits.sub_bits_u8(0, 3)), 4);
  bits.push_bits(u32::from(djinn_bits.sub_bits_u8(24, 27)), 4);

  for i in (7..=23).rev().step_by(8) {
    bits.push_bits(u32::from(djinn_bits.sub_bits_u8(i - 7, i)), 8);
  }

  for i in (0..=7).rev() {
    if i < save_data.events.len() {
      bits.push_bit(u32::from(save_data.events[i]));
    } else {
      bits.push_bit(0);
    }
  }

  let size = match grade {
    PasswordGrade::Bronze => 9,
    PasswordGrade::Silver => 39,
    PasswordGrade::Gold => 173,
  };

  // If password grade is silver or bronze,
  // insert 8 bits representing which of these items your party has.
  if grade.is_silver() || grade.is_bronze() {
    let mut flags = 0;
    for items_per_person in &save_data.items {
      for item in items_per_person {
        let id = item & 0x1FF;
        for (j, psynergy_item) in PSYNERGY_ITEMS.iter().enumerate() {
          if id == *psynergy_item {
            flags |= u32::pow(2, u32::try_from(j).unwrap());
          }
        }
      }
    }
    bits.push_bits(flags, 8);
  }

  // If password grade is gold or silver, insert stats.
  if grade.is_gold() || grade.is_silver() {
    for stats_per_person in &save_data.stats {
      // HP
      bits.push_bits(u32::from(stats_per_person[0]), 11);
      // EP
      bits.push_bits(u32::from(stats_per_person[1]), 11);
      // Attack
      bits.push_bits(u32::from(stats_per_person[2]), 10);
      // Defense
      bits.push_bits(u32::from(stats_per_person[3]), 10);
      // Agility
      bits.push_bits(u32::from(stats_per_person[4]), 10);
      // Luck
      bits.push_bits(u32::from(stats_per_person[5]), 8);
    }
  }

  // If password grade is gold, insert items and coins.
  if grade.is_gold() {
    bits.push_bits(0, 8);
    let mut counter = 0;
    for items_per_person in &save_data.items {
      for item in items_per_person {
        let id = item & 0x1FF;
        bits.push_bits(u32::from(id), 9);
        counter += 1;
        // Append a 0 bit every 7 items.
        if counter == 7 {
          bits.push_bit(0);
          counter = 0;
        }
      }
    }

    // Insert quantities of stackable items for each energist(adept).
    for items_per_person in &save_data.items {
      for stackable_item in &STACKABLE_ITEMS {
        let mut quantity = 0;
        for item in items_per_person {
          let id = item & 0x1FF;
          if id == *stackable_item {
            quantity = item >> 11;
          }
        }
        bits.push_bits(u32::from(quantity), 5);
      }
    }
    bits.push_bits(save_data.coins, 24);
  }

  // Append 0 until reaching the correct password size.
  for _i in 0..8 * size - bits.get_len() {
    bits.push_bit(0);
  }

  // Encrypt with key 0x1021.
  let mut x_sum = 0xFFFF;

  for i in 0..size {
    let byte = u32::from(bits.sub_bits_u8(8 * i, 8 * i + 7));
    x_sum ^= byte << 8;
    for _j in 0..8 {
      if (x_sum & 0x8000) == 0 {
        x_sum <<= 1;
      } else {
        x_sum = (x_sum << 1) + 0xEFDF;
      }
    }
  }

  x_sum = (!x_sum) & 0xFFFF;
  let xor_value = x_sum & 0xFF;
  bits.push_bits(x_sum >> 8, 8);
  bits.push_bits(xor_value, 8);

  for i in 0..=size {
    let byte = u32::from(bits.sub_bits_u8(8 * i, 8 * i + 7));
    bits.replace_bits(byte ^ xor_value, 8, 8 * i);
  }

  // Split into 6-bit entries
  let mut password_bytes = Vec::new();
  let mut acc = 0_u32;
  let max_size = (size + 2) * 8;

  for i in (0..max_size).step_by(6) {
    let mut max = i + 5;
    let entry;

    if max <= bits.get_len() {
      entry = bits.sub_bits_u8(i, max);
    } else {
      let last_left_shift = max - max_size + 1;
      max = bits.get_len() - 1;
      entry = bits.sub_bits_u8(i, max) << last_left_shift;
    }

    password_bytes.push(entry);
    acc += u32::from(entry);
    // Insert checksum for each line
    if password_bytes.len() % 10 == 9 {
      password_bytes.push((acc & 0x3F) as u8);
      acc = 0;
    }
  }

  for (i, password_byte) in password_bytes.iter_mut().enumerate() {
    let mut temp = u16::from(*password_byte);
    temp += u16::try_from(i).unwrap();
    *password_byte = (temp & 0x3F) as u8;
  }

  password_bytes
}

pub fn get_password_bytes_with_raw_save_bytes(raw_save: &[u8], grade: PasswordGrade) -> Vec<u8> {
  let save_data = gen_save_data_with_raw_save_bytes(raw_save);
  gen_password_bytes_with_save_data(grade, &save_data)
}

pub fn get_valid_password_bits_option(
  password_bytes: &[u8],
  check_validity: bool,
) -> Option<BitArray> {
  let password_grade = match password_bytes.len() {
    260 => PasswordGrade::Gold,
    61 => PasswordGrade::Silver,
    16 => PasswordGrade::Bronze,
    _ => unreachable!(),
  };

  let mut bytes_data = vec![0; password_bytes.len()];
  /* Remove each byte's position data by minus its index.
  I'm not sure if it's the correct way to do that, but "+ 0x200" works.
  Maybe there is a better way for this...

  Notes about Hex to Bin:
  63  -> 0x3F  ->     11 1111

  64  -> 0x40  ->    100 0000
  128 -> 0x80  ->   1000 0000
  256 -> 0x100 -> 1 0000 0000

  Max len
  16  -> 0x10  ->      1 0000
  61  -> 0x3D  ->     11 1101
  260 -> 0x104 -> 1 0000 0100 */
  for (i, data) in password_bytes.iter().enumerate() {
    bytes_data[i] = ((u16::from(*data) + 0x200 - u16::try_from(i).unwrap()) & 0x3F) as u8;
  }

  /* Remove all checksum bytes.
  Those checksum bytes are useless for getting game data. */
  match password_grade {
    PasswordGrade::Gold => {
      // Or you can use: for i in 9..260.step_by(9)
      for i in CHECKSUM_INDEX {
        bytes_data.remove(i);
      }
    }
    PasswordGrade::Silver => {
      // Or you can use: for i in 9..61.step_by(9)
      for index in CHECKSUM_INDEX.iter().take(6) {
        bytes_data.remove(*index);
      }
    }
    PasswordGrade::Bronze => {
      bytes_data.remove(CHECKSUM_INDEX[0]);
    }
  }

  // Convert bytes to bit array
  let size = match password_grade {
    // Gold: 260 - 26 = 234 => 173
    PasswordGrade::Gold => 173,
    // Silver: 61 - 6 = 55 => 39
    PasswordGrade::Silver => 39,
    // Bronze: 16 - 1 = 15 => 9
    PasswordGrade::Bronze => 9,
  };
  let mut bits = BitArray::new();
  let max_size = (size + 2) * 8;

  for (j, i) in (0..max_size).step_by(6).enumerate() {
    let max = i + 5;
    let entry = bytes_data[j];

    if max <= max_size {
      bits.push_bits(u32::from(entry), 6);
    } else {
      let last_right_shift = max - max_size + 1;
      bits.push_bits(u32::from(entry >> last_right_shift), max_size - i);
    }
  }

  // Get the second half of the key to decrypt all bits with it.
  let key_second_half_from_source_bytes = bits.sub_bits_u8(max_size - 8, max_size - 1);

  // Decrypt bits
  for i in 0..=size {
    let byte = u32::from(bits.sub_bits_u8(8 * i, 8 * i + 7));
    bits.replace_bits(
      byte ^ u32::from(key_second_half_from_source_bytes),
      8,
      8 * i,
    );
  }

  if check_validity {
    // All bits have been decrypted, so you can get the first half of the key directly
    let key_first_half_from_source_bytes = bits.sub_bits_u8(max_size - 16, max_size - 9);

    // Recalculate the key from decrypted bits to check the validity of the password.
    let mut x_sum = 0xFFFF;

    for i in 0..size {
      let byte = u32::from(bits.sub_bits_u8(8 * i, 8 * i + 7));
      x_sum ^= byte << 8;
      for _j in 0..8 {
        if (x_sum & 0x8000) == 0 {
          x_sum <<= 1;
        } else {
          x_sum = (x_sum << 1) + 0xEFDF;
        }
      }
    }
    x_sum = (!x_sum) & 0xFFFF;
    let xor_value = x_sum & 0xFF;

    // Let's see if the values of the two keys are equal.
    let key_from_source_bytes = u16::from_be_bytes([
      key_first_half_from_source_bytes,
      key_second_half_from_source_bytes,
    ]);
    let key_from_result_bits =
      u16::from_be_bytes([u8::try_from(x_sum >> 8).unwrap(), xor_value as u8]);
    if key_from_result_bits == key_from_source_bytes {
      Some(bits)
    } else {
      None
    }
  } else {
    Some(bits)
  }
}

fn gen_save_data_with_password_bits(
  password_bits: &mut BitArray,
  password_grade: PasswordGrade,
) -> SaveData {
  /* [u8; 4]
  TEMP Default Values
  Order: Isaac, Garet, Ivan, Mia */
  let mut levels = [28, 28, 28, 28];

  /* [u32; 4]
  TEMP Default Values
  Venus:   0101101 (Order: Bane, Ground, Sap, Vine, Quartz, Granite, Flint)
  Mercury: 0001111 (Order: Dew, Tonic, Hail, Spritz, Mist, Sleet, Fizz)
  Mars:    1101110 (Order: Torch, Flash, Ember, Scorch, Corona, Fever, Forge)
  Jupiter: 1001111 (Order: Luff, Squall, Kite, Smog, Zephyr, Breeze, Gust) */
  let mut djinn = [0x2D, 0x0F, 0x6E, 0x4F];

  /* [u8; 6]
  TEMP Default values
  Order is: Hammet, Colosso, Hsu, Deadbeard, Vale, Vault*/
  let mut events = [0, 0, 1, 0, 0, 0];

  /* [[u16; 6]; 4]
  TEMP Default values
  Isaac: HP, EP, Attack, Defense, Agility, Luck
  Garet: HP, EP, Attack, Defense, Agility, Luck
  Ivan:  HP, EP, Attack, Defense, Agility, Luck
  Mia:   HP, EP, Attack, Defense, Agility, Luck */
  let mut stats = [
    [241, 101, 116, 53, 117, 3],
    [257, 97, 112, 53, 103, 2],
    [220, 117, 100, 51, 124, 4],
    [230, 115, 102, 49, 109, 5],
  ];

  /* [[u16; 15]; 4]
  TEMP Default Values
  Isaac: Great Sword, Steel Armor, Knight's Shield, Knight's Helm
  Garet: Great Axe, Steel Armor, Knight's Shield, Knight's Helm
  Ivan:  Master Rapier, Silver Vest, Silver Armlet, Platinum Circlet
  Mia:   War Mace, Silver Vest, Silver Armlet, Platinum Circlet */
  let mut items = [
    [
      0x04, 0x50, 0x79, 0x96, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ],
    [
      0x21, 0x50, 0x79, 0x96, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ],
    [
      0x13, 0x5E, 0x8B, 0xA9, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ],
    [
      0x2E, 0x5E, 0x8B, 0xA9, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ],
  ];

  // u32
  // Coins, literally.
  let mut coins = 0;

  /* Remove all useless bits.
  Gold grade:
  Step 1: Remove 16 bits of the key for encryption/decryption, and those 40 useless "blank" bits before the key bits.
  Step 2: Remove 8 useless bits in item bits, since the password already appended a 0 bit every 7 items. (each item take 9 bits, so that means, it append a 0 bit every 63 bits)
  Step 3: Remove 8 useless "blank" bits ("00000000") between stats bits and item bits.
  Step 4: Remove 2 useless "blank" bits ("00") before events' bits.

  Silver and bronze grades:
  Step 1: Remove 16 bits of the key for encryption/decryption.
  Step 2: Remove two useless bits "00" before events' bits. */
  match password_grade {
    PasswordGrade::Gold => {
      /* (2 * 8) + 40
      -> 16 + 40
      -> 56 */
      for _i in 0..56 {
        password_bits.bits.remove(password_bits.get_len() - 1);
      }
      /* (4 * 7) + (4 * 7) + (2 + 6) + (4 * (11 + 11 + 10 + 10 + 10 + 8)) + 8 - 1
      -> 28 + 28 + 8 + 240 + 8 - 1
      -> 311 */
      for i in (1..=8).rev() {
        password_bits.bits.remove(311 + i * 64);
      }
      /* (4 * 7) + (4 * 7) + (2 + 6) + (4 * (11 + 11 + 10 + 10 + 10 + 8))
      -> 28 + 28 + 8 + 240
      -> 304 */
      for _i in 0..8 {
        password_bits.bits.remove(304);
      }
      /* (4 * 7) + (4 * 7)
      -> 28 + 28
      -> 56 */
      for _i in 0..2 {
        password_bits.bits.remove(56);
      }
    }
    PasswordGrade::Silver | PasswordGrade::Bronze => {
      /* (2 * 8)
      -> 16 */
      for _i in 0..16 {
        password_bits.bits.remove(password_bits.get_len() - 1);
      }
      /* (4 * 7) + (4 * 7)
      -> 28 + 28
      -> 56 */
      for _i in 0..2 {
        password_bits.bits.remove(56);
      }
    }
  }

  /* Get level bits, rearrange it to normal order.
  Each level value has 7 bits.
  Order is: Mia, Ivan, Garet, Isaac */
  let mut level_bits = BitArray::new();
  level_bits.push_bits(u32::from(password_bits.sub_bits_u8(24, 27)), 4);
  for i in (7..=23).rev().step_by(8) {
    level_bits.push_bits(u32::from(password_bits.sub_bits_u8(i - 7, i)), 8);
  }
  for (i, level) in levels.iter_mut().enumerate() {
    *level = level_bits.sub_bits_u8(21 - i * 7, 27 - i * 7);
  }

  /* Get djinn bits, rearrange it to normal order.
  Each element has 7 bits.
  Order is: Jupiter, Mars, Mercury, Venus */
  let mut djinn_bits = BitArray::new();
  for i in (39..=55).rev().step_by(8) {
    djinn_bits.push_bits(u32::from(password_bits.sub_bits_u8(i - 7, i)), 8);
  }
  djinn_bits.push_bits(u32::from(password_bits.sub_bits_u8(28, 31)), 4);
  for (i, djinn_data) in djinn.iter_mut().enumerate() {
    *djinn_data = u32::from(djinn_bits.sub_bits_u8(21 - i * 7, 27 - i * 7));
  }

  /* Get all event bits.
  Order is: Vault, Vale, Deadbeard, Hsu, Colosso, Hammet */
  for (i, event) in events.iter_mut().enumerate() {
    *event = password_bits.bits[61 - i];
  }

  /* If password grade is silver or bronze, check 8 psynergy items.
  if it does has one, insert it back to Isaac's inventory */
  if password_grade.is_silver() || password_grade.is_bronze() {
    /* Isaac's inventory: Great Sword, Steel Armor, Knight's Shield, Knight's Helm
    So the start index is 4 */
    let mut isaac_inventory_start_index = 4;
    for (i, psynergy_item) in PSYNERGY_ITEMS.iter().enumerate() {
      // Reversed order
      if password_bits.bits[62 + (7 - i)] == 1 {
        items[0][isaac_inventory_start_index] = *psynergy_item;
        isaac_inventory_start_index += 1;
      }
    }
  }

  // If password grade is gold or silver, get stats.
  if password_grade.is_gold() || password_grade.is_silver() {
    let start_index = if password_grade.is_gold() {
      /* (4 * 7) + (4 * 7) + 6
      -> 28 + 28 + 6
      -> 62 */
      62
    } else {
      /* (4 * 7) + (4 * 7) + 6 + 8
      -> 28 + 28 + 6 + 8
      -> 70 */
      70
    };
    for (i, stats_per_person) in stats.iter_mut().enumerate() {
      // HP
      stats_per_person[0] =
        password_bits.sub_bits_u16(start_index + i * 60, start_index + 10 + i * 60);
      // EP
      stats_per_person[1] =
        password_bits.sub_bits_u16(start_index + 11 + i * 60, start_index + 21 + i * 60);
      // Attack
      stats_per_person[2] =
        password_bits.sub_bits_u16(start_index + 22 + i * 60, start_index + 31 + i * 60);
      // Defense
      stats_per_person[3] =
        password_bits.sub_bits_u16(start_index + 32 + i * 60, start_index + 41 + i * 60);
      // Agility
      stats_per_person[4] =
        password_bits.sub_bits_u16(start_index + 42 + i * 60, start_index + 51 + i * 60);
      // Luck
      stats_per_person[5] =
        password_bits.sub_bits_u16(start_index + 52 + i * 60, start_index + 59 + i * 60);
    }
  }

  if password_grade.is_gold() {
    /* (4 * 7) + (4 * 7) + 6 + (4 * (11 + 11 + 10 + 10 + 10 + 8))
    -> 28 + 28 + 6 + 240
    -> 302 */
    let item_id_start_index = 302;
    /* (4 * 7) + (4 * 7) + 6 + (4 * (11 + 11 + 10 + 10 + 10 + 8)) + (4 * (9 * 15))
    -> 28 + 28 + 6 + 240 + 540
    -> 842 */
    let stackable_item_quantity_start_index = 842;
    for (i, items_per_person) in items.iter_mut().enumerate() {
      for (j, item) in items_per_person.iter_mut().enumerate() {
        let item_id = password_bits.sub_bits_u16(
          item_id_start_index + i * 135 + j * 9,
          item_id_start_index + 8 + i * 135 + j * 9,
        );
        if let Some(stackable_item_index) = STACKABLE_ITEMS.iter().position(|&x| x == item_id) {
          let item_quantity = password_bits.sub_bits_u8(
            stackable_item_quantity_start_index + i * 115 + stackable_item_index * 5,
            stackable_item_quantity_start_index + 4 + i * 115 + stackable_item_index * 5,
          );
          // The minimium value of "item quantity" is 0 (0 -> 1 in-game)
          let quantity_part = u16::from(item_quantity) << 11;
          *item = quantity_part + item_id;
        } else {
          *item = item_id;
        }
      }
    }
    coins = password_bits.sub_bits_u32(password_bits.get_len() - 24, password_bits.get_len() - 1);
  }

  SaveData {
    levels,
    djinn,
    events,
    stats,
    items,
    coins,
  }
}

pub fn get_save_data_with_password_bits(
  password_bits: &mut BitArray,
  source_grade: PasswordGrade,
) -> SaveData {
  gen_save_data_with_password_bits(password_bits, source_grade)
}

pub fn get_password_bytes_with_save_data(
  save_data: &SaveData,
  target_grade: PasswordGrade,
) -> Vec<u8> {
  gen_password_bytes_with_save_data(target_grade, save_data)
}

fn gen_exported_data_for_dyrati_sheet_with_info(pwd_info: &PasswordInfo) -> String {
  let mut exported_text = String::new();
  // Text start
  exported_text.push_str("用于 Dyrati 的“Golden Sun Password Generator”表格的密码数据文本文件\n");
  exported_text.push_str(
    "来源出处：https://www.reddit.com/r/GoldenSun/comments/jon3h7/golden_sun_password_tools/\n",
  );
  exported_text.push_str("在线版表格：https://docs.google.com/spreadsheets/d/1jQ2Zj2F57Fb4hs0pDYLCaZL-qry7gcVxNbhpq0BJDUs/\n");
  exported_text.push_str("对于 Windows 系统用户，推荐直接使用系统自带的记事本打开该文本文件。\n");
  exported_text.push_str(
    "复制“===开始===”和“===结束===”之间的所有内容，并粘贴到表格对应的单元格范围。\n\n\n\n",
  );

  // Levels
  exported_text.push_str("等级\n罗宾，杰拉德，伊万，梅雅莉\n表格中的范围 -> C8:F8\n");
  exported_text.push_str("============开始============\n");
  for (i, level) in pwd_info.levels.iter().enumerate() {
    if i < pwd_info.levels.len() - 1 {
      let _ = write!(exported_text, "{level}\t");
    } else {
      let _ = write!(exported_text, "{level}\n");
    }
  }
  exported_text.push_str("============结束============\n\n\n\n");

  // Djinn
  exported_text.push_str("精灵\n维纳斯（地/金星），玛尔斯（火/火星），朱庇特（风/木星），墨丘利（水/水星）\n表格中的范围 -> C12:F18\n");
  exported_text.push_str("====================开始====================\n");

  for i in 0..7 {
    for j in 0..4 {
      if pwd_info.djinn[0][i] {
        exported_text.push_str(ALL_DJINN_NAMES[j][i].get_name(Language::English));
      }
      if j == 3 {
        exported_text.push('\n');
      } else {
        exported_text.push('\t');
      }
    }
  }
  exported_text.push_str("====================结束====================\n\n\n\n");

  // Events
  exported_text.push_str("剧情事件\n从上至下的顺序为：\n（1）营救哈梅特，（2）竞技大赛获胜（最终决战获胜），（3）帮忙救助乌鲁木齐，（4）打败塔罗斯（宝岛最终Boss），（5）重返海迪亚（获悉罗宾母亲生病），（6）重返库普阿普（获悉小偷越狱逃跑）。\n\n");
  exported_text.push_str("注：在 Paulygon 开发的“黄金太阳密码生成器”中，“帮忙救助乌鲁木齐”（事件 #3）的值与表格中的相反，\n");
  exported_text.push_str(
    "　　即在“黄金太阳密码生成器”中，勾选了“事件 #3”反而表示未完成该事件，而不勾选“事件 #3”才表示完成了该事件，不过这与游戏使用的实际数值对应。\n",
  );
  exported_text.push_str("　　表格版密码生成器为了方便理解，与其他事件进行了同步，即用1表示完成了该事件，用0表示未完成该事件。\n\n");
  exported_text.push_str("表格中的范围 -> E21:E26\n");
  exported_text.push_str("====开始====\n");
  for event in pwd_info.events {
    let _ = writeln!(exported_text, "{}", if event { "1" } else { "0" });
  }
  exported_text.push_str("====结束====\n\n\n\n");

  // Base Stats
  exported_text.push_str("基础能力状态属性\n罗宾，杰拉德，伊万，梅雅莉\n");
  exported_text.push_str("从上至下的顺序为：\nHP, EP, 攻击, 防御, 敏捷, 幸运\n\n");
  exported_text.push_str(
    "注：“基础能力状态属性”即\"Base stats\"，是不装备任何物品，没有附身任何精灵，也没有乘上角色默认职业的各属性倍率进行修正的角色能力属性值。\n",
  );
  exported_text.push_str(
    "　　每个角色都拥有默认的基础职业，并且在游戏中基础属性值会乘上职业对应的属性倍率,\n",
  );
  exported_text.push_str("　　也就是说，这些是角色最本质的基础值，是无法在游戏里查看确认到的。\n");
  exported_text
    .push_str("　　如果要使用“基础能力状态属性”来生成密码，记得把 J3 单元格选为\"Base stats\"。\n");
  exported_text
    .push_str("　　（“基础能力状态属性”使用起来可能不太方便，不过不会出现数值误差）\n\n");
  exported_text.push_str("表格中的范围 -> J5:M10\n");
  exported_text.push_str("============开始============\n");

  // Stats cap in dyrati sheet.
  let mut dyrati_base_stats = pwd_info.base_stats;
  let mut dyrati_stats = pwd_info.stats;
  for i in 0..6 {
    for j in 0..4 {
      if dyrati_base_stats[j][i] > MAX_VALID_IN_GAME_STATS_VALUES[i] {
        dyrati_base_stats[j][i] = MAX_VALID_IN_GAME_STATS_VALUES[i];
        dyrati_stats[j][i] = (MAX_VALID_IN_GAME_STATS_VALUES[i] as f64
          * DEFAULT_CLASS_MULTIPLIERS[j][i])
          .floor() as u16;
      }
    }
  }

  for i in 0..6 {
    let _ = write!(exported_text, "{}\t", dyrati_base_stats[0][i]);
    let _ = write!(exported_text, "{}\t", dyrati_base_stats[1][i]);
    let _ = write!(exported_text, "{}\t", dyrati_base_stats[2][i]);
    let _ = writeln!(exported_text, "{}", dyrati_base_stats[3][i]);
  }
  exported_text.push_str("============结束============\n\n\n\n");

  // Stats, there may be some problems in this.
  exported_text.push_str("能力状态属性\n罗宾，杰拉德，伊万，梅雅莉\n");
  exported_text.push_str("从上至下的顺序为：\nHP, EP, 攻击, 防御, 敏捷, 幸运\n\n");
  exported_text
    .push_str("注：“能力状态属性”即\"Stats\"，是不装备任何物品，没有附身任何精灵，但是有乘上角色默认职业的各属性倍率进行修正的角色能力属性值。\n");
  exported_text.push_str("　　换句话说，这些属性值是可以在游戏里亲自确认到的，只要卸下所有装备，待命所有精灵就能看到。\n");
  exported_text
    .push_str("　　如果要使用“能力状态属性”来生成密码，记得把 J3 单元格选为\"Stats\"。\n");
  exported_text
    .push_str("　　（使用起来方便，但是在一些情况下会产生极其微小的数值误差，导致密码发生变化。");
  exported_text.push_str("这是因为在游戏中，几个不同的值有可能在经过计算后得到相同的结果，所以表格在逆向除以职业属性倍率获得真实的基础属性值时会假定该数值为多个可能的数值中的最大值。");
  exported_text.push_str("如果你非常在意这个误差问题的话，请使用“基础能力状态属性”吧）\n\n");
  exported_text.push_str("表格中的范围 -> J5:M10\n");
  exported_text.push_str("============开始============\n");
  for i in 0..6 {
    let _ = write!(exported_text, "{}\t", dyrati_stats[0][i]);
    let _ = write!(exported_text, "{}\t", dyrati_stats[1][i]);
    let _ = write!(exported_text, "{}\t", dyrati_stats[2][i]);
    let _ = writeln!(exported_text, "{}", dyrati_stats[3][i]);
  }
  exported_text.push_str("============结束============\n\n\n\n");

  // Items
  exported_text.push_str("物品\n罗宾，杰拉德，伊万，梅雅莉\n表格中的范围 -> J14:M28\n");
  exported_text.push_str(
    "========================================开始========================================\n",
  );
  for i in 0..15 {
    let (isaac_item_id, item_count) = pwd_info.items[0][i];
    if isaac_item_id != 0 {
      let isaac_item_index = (isaac_item_id - 1) as usize;
      exported_text.push_str(DYRATI_ITEM_NAMES[isaac_item_index]);
      if STACKABLE_ITEMS.contains(&isaac_item_id) {
        let _ = write!(exported_text, "*{}", item_count);
      }
    }
    exported_text.push('\t');

    let (garet_item_id, item_count) = pwd_info.items[1][i];
    if garet_item_id != 0 {
      let garet_item_index = (garet_item_id - 1) as usize;
      exported_text.push_str(DYRATI_ITEM_NAMES[garet_item_index]);
      if STACKABLE_ITEMS.contains(&garet_item_id) {
        let _ = write!(exported_text, "*{}", item_count);
      }
    }
    exported_text.push('\t');

    let (ivan_item_id, item_count) = pwd_info.items[2][i];
    if ivan_item_id != 0 {
      let ivan_item_index = (ivan_item_id - 1) as usize;
      exported_text.push_str(DYRATI_ITEM_NAMES[ivan_item_index]);
      if STACKABLE_ITEMS.contains(&ivan_item_id) {
        let _ = write!(exported_text, "*{}", item_count);
      }
    }
    exported_text.push('\t');

    let (mia_item_id, item_count) = pwd_info.items[3][i];
    if mia_item_id != 0 {
      let mia_item_index = (mia_item_id - 1) as usize;
      exported_text.push_str(DYRATI_ITEM_NAMES[mia_item_index]);
      if STACKABLE_ITEMS.contains(&mia_item_id) {
        let _ = write!(exported_text, "*{}", item_count);
      }
    }
    exported_text.push('\n');
  }
  exported_text.push_str(
    "========================================结束========================================\n\n\n\n",
  );
  // Coins
  exported_text.push_str("金币\n表格中的范围 -> D28\n");
  exported_text.push_str("====开始====\n");
  let _ = writeln!(exported_text, "{}", pwd_info.coins);
  exported_text.push_str("====结束====\n");

  exported_text
}

pub fn get_exported_data_for_dyrati_sheet_with_raw_save_bytes(raw_save: &[u8]) -> String {
  let pwd_info = gen_password_info_with_raw_save_bytes(raw_save);
  gen_exported_data_for_dyrati_sheet_with_info(&pwd_info)
}

pub fn get_exported_data_for_dyrati_sheet_by_save_data(save_data: &SaveData) -> String {
  let pwd_info = gen_password_info_with_save_data(save_data);
  gen_exported_data_for_dyrati_sheet_with_info(&pwd_info)
}

pub fn get_exported_data_for_dyrati_sheet_by_bytes(
  password_bytes: &[u8],
  target_grade: PasswordGrade,
) -> String {
  let mut password_bits = get_valid_password_bits_option(password_bytes, false).unwrap();
  let save_data = gen_save_data_with_password_bits(&mut password_bits, target_grade);
  let pwd_info = gen_password_info_with_save_data(&save_data);
  gen_exported_data_for_dyrati_sheet_with_info(&pwd_info)
}

pub fn gen_password_info_with_raw_save_bytes(raw_save: &[u8]) -> PasswordInfo {
  let save_data = gen_save_data_with_raw_save_bytes(raw_save);
  gen_password_info_with_save_data(&save_data)
}

pub fn gen_password_info_with_save_data(save_data: &SaveData) -> PasswordInfo {
  // Levels
  let levels = save_data.levels;
  // Djinn
  let mut venus_djinn_bits = BitArray::new();
  let mut mercury_djinn_bits = BitArray::new();
  let mut mars_djinn_bits = BitArray::new();
  let mut jupiter_djinn_bits = BitArray::new();
  venus_djinn_bits.push_bits(save_data.djinn[0], 7);
  mercury_djinn_bits.push_bits(save_data.djinn[1], 7);
  mars_djinn_bits.push_bits(save_data.djinn[2], 7);
  jupiter_djinn_bits.push_bits(save_data.djinn[3], 7);
  let mut venus_djinn = [false; 7];
  let mut mercury_djinn = [false; 7];
  let mut mars_djinn = [false; 7];
  let mut jupiter_djinn = [false; 7];
  // Reversed order
  for i in 0..7 {
    if venus_djinn_bits.bits[6 - i] == 1 {
      venus_djinn[i] = true;
    }
    if mars_djinn_bits.bits[6 - i] == 1 {
      mars_djinn[i] = true;
    }
    if jupiter_djinn_bits.bits[6 - i] == 1 {
      jupiter_djinn[i] = true;
    }
    if mercury_djinn_bits.bits[6 - i] == 1 {
      mercury_djinn[i] = true;
    }
  }
  //let djinn = [venus_djinn, mercury_djinn, mars_djinn, jupiter_djinn];
  let djinn = [venus_djinn, mars_djinn, jupiter_djinn, mercury_djinn];

  // Events
  let mut events = [false; 6];
  for (i, event) in save_data.events.iter().enumerate() {
    if i == 2 {
      let inverted_event = (event ^ 1) & 1;
      events[i] = inverted_event == 1;
    } else {
      events[i] = *event == 1;
    }
  }
  // Base Stats
  let base_stats = save_data.stats;
  // Stats
  let mut stats = [[0; 6]; 4];
  for i in 0..6 {
    for j in 0..4 {
      stats[j][i] = get_in_game_stats_tla(
        save_data.stats[j][i],
        DEFAULT_CLASS_MULTIPLIERS[j][i],
        MAX_VALID_IN_GAME_STATS_VALUES[i],
        MAX_VALID_BASE_STATS_VALUES_PWD[i],
      );
    }
  }

  let mut psy_items = [false; 8];
  // Items
  let mut items = [[(0, 0); 15]; 4];
  for i in 0..15 {
    let isaac_item = save_data.items[0][i];
    let isaac_item_id = isaac_item & 0x1FF;
    if isaac_item_id != 0 {
      let item_count = if STACKABLE_ITEMS.contains(&isaac_item_id) {
        (isaac_item >> 11) + 1
      } else {
        1
      };
      items[0][i] = (isaac_item_id, item_count as u8);

      if PSYNERGY_ITEMS.contains(&isaac_item_id) {
        psy_items[(isaac_item_id - PSYNERGY_ITEMS[0]) as usize] = true;
      }
    }

    let garet_item = save_data.items[1][i];
    let garet_item_id = garet_item & 0x1FF;
    if garet_item_id != 0 {
      let item_count = if STACKABLE_ITEMS.contains(&garet_item_id) {
        (garet_item >> 11) + 1
      } else {
        1
      };
      items[1][i] = (garet_item_id, item_count as u8);

      if PSYNERGY_ITEMS.contains(&garet_item_id) {
        psy_items[(garet_item_id - PSYNERGY_ITEMS[0]) as usize] = true;
      }
    }

    let ivan_item = save_data.items[2][i];
    let ivan_item_id = ivan_item & 0x1FF;
    if ivan_item_id != 0 {
      let item_count = if STACKABLE_ITEMS.contains(&ivan_item_id) {
        (ivan_item >> 11) + 1
      } else {
        1
      };
      items[2][i] = (ivan_item_id, item_count as u8);

      if PSYNERGY_ITEMS.contains(&ivan_item_id) {
        psy_items[(ivan_item_id - PSYNERGY_ITEMS[0]) as usize] = true;
      }
    }

    let mia_item = save_data.items[3][i];
    let mia_item_id = mia_item & 0x1FF;
    if mia_item_id != 0 {
      let item_count = if STACKABLE_ITEMS.contains(&mia_item_id) {
        (mia_item >> 11) + 1
      } else {
        1
      };
      items[3][i] = (mia_item_id, item_count as u8);

      if PSYNERGY_ITEMS.contains(&mia_item_id) {
        psy_items[(mia_item_id - PSYNERGY_ITEMS[0]) as usize] = true;
      }
    }
  }
  // Coins
  let coins = save_data.coins;

  PasswordInfo {
    levels,
    djinn,
    events,
    base_stats,
    stats,
    psy_items,
    items,
    coins,
  }
}

/*
fn get_base_stats(input_stats: u16, class_multiplier: f64) -> u16 {
  ((((input_stats as f64) + 0.9) / class_multiplier).floor()) as u16
}
*/

fn get_in_game_stats_tla(
  input_stats: u16,
  class_multiplier: f64,
  max_in_game_stats: u16,
  max_base_stats_pwd: u16,
) -> u16 {
  let input_stats_f64 = if input_stats > max_base_stats_pwd {
    if max_base_stats_pwd == MAX_VALID_BASE_STATS_VALUES_PWD[2] {
      input_stats & max_base_stats_pwd
    } else {
      max_base_stats_pwd
    }
  } else {
    input_stats
  } as f64;

  let result = ((input_stats_f64 * class_multiplier).floor()) as u16;

  if result > max_in_game_stats {
    max_in_game_stats
  } else {
    result
  }
}

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
