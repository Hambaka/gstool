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
pub const HEADER_CAMELOT_ASCII_STRING: &str = "CAMELOT";

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
pub const GS_BUILD_DATE: [[u16; 6]; 2] = [
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
pub const PC_NAME: [[[u8; 7]; 8]; 8] = [
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
pub const SAVE_SLOT_SIZE: [usize; 2] = [0x1000, 0x3000];

/// TBS: 64KB / 4KB = 16
/// TLA: 64KB / 12KB = 5
pub const MAX_LOOP_COUNT: [usize; 2] = [16, 5];

/// All build date locations: 0x36 to 0x37, 0x250 to 0x251, 0x508 to 0x509
/// All stored values are same.
pub const FIRST_BUILD_DATE_LOCATION_INDEX: [usize; 2] = [0x36, 0x37];

/* Unused, since now we have a much better way to check if the save data is clear data.

All save map(location) locations  :0x410 to 0x413, 0x418 to 0x41B
Save map data -> Main:Minor
All stored values are same.
pub const FIRST_SAVE_MAP_MAIN_LOCATION_INDEX: [usize; 2] = [0x410, 0x411];
pub const FIRST_SAVE_MAP_MINOR_LOCATION_INDEX: [usize; 2] = [0x412, 0x413];  */

/* Unused, since now we have a much better way to check if the save data is clear data.

Clear data's save location value is -> 1:2
pub const CLEAR_DATA_SAVE_MAP_VALUE: [u16; 2] = [0x01, 0x02]; */

/// For clear data, its value is 1.
/// If the save data is not a clear data, its value is 0.
pub const CLEAR_DATA_IDENTIFIER_VALUE: u8 = 0x01;

/// Location of `CLEAR_DATA_IDENTIFIER_VALUE` in save file.
pub const CLEAR_DATA_IDENTIFIER_INDEX: usize = 0x31;

/// The 8th byte in save header is the slot number, it only show 3 active save data in game.
pub const HEADER_SAVE_SLOT_NUMBER_LOCATION_INDEX: usize = 0x07;

/// The values for 3 'active' save data are: 0x00, 0x01 and 0x02.
/// For more information, please see the comment at the beginning of the code.
pub const MAX_VALID_SLOT_NUMBER: u8 = 0x02;

/// Location of save data's priority
/// For more information, please see the comment at the beginning of the code.
pub const HEADER_PRIORITY_LOCATION_INDEX: [usize; 2] = [0x0A, 0x0B];

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
pub const CHECKSUM_INDEX: [usize; 26] = [
  9, 18, 27, 36, 45, 54, 63, 72, 81, 90, 99, 108, 117, 126, 135, 144, 153, 162, 171, 180, 189, 198,
  207, 216, 225, 234,
];

/// These default classes are:
/// Squire (Isaac), Guard (Garet), Wind Seer (Ivan), Water Seer (Mia)
#[rustfmt::skip]
pub const DEFAULT_CLASS_MULTIPLIERS: [[f64; 6]; 4] = [
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
pub const MAX_VALID_IN_GAME_STATS_VALUES: [u16; 6] = [
  1999,
  1999,
  999,
  999,
  999,
  99,
];

/// In TBS, it should be 0 -> Robin (Isaac)
/// In TLA, it should be 4 -> Garcia (Felix)
pub const PARTY_MAIN_LEADER_INDEX: [usize; 2] = [0, 4];

/// In TBS, we only have four available members: Robin (Isaac), Gerald (Garet), Ivan and Mary/Mearī (Mia).
/// We should also include Garcia (Felix), Jasmine (Jenna) and Shiba (Sheba).
/// In TLA, well, We have Picard (Piers) in party now.
pub const PARTY_MEMBERS_COUNT: [usize; 2] = [7, 8];
pub const PC_NAME_LOCATION_INDEX: [usize; 2] = [0x510, 0x530];
pub const BUILD_DATE_LOCATION_INDEX: [[[usize; 2]; 3]; 2] = [
  [[0x36, 0x37], [0x250, 0x251], [0x508, 0x509]],
  [[0x36, 0x37], [0x250, 0x251], [0x528, 0x529]],
];

/// Save slot size - header size
/// Header size is 0x10.
/// TBS: 0x1000 - 0x10
/// TLA: 0x3000 - 0x10
pub const CHECKSUM_RANGE: [usize; 2] = [0xFF0, 0x2FF0];
pub const HEADER_CHECKSUM_LOCATION_INDEX: [usize; 2] = [0x08, 0x09];
