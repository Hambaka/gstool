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
pub const MAX_BASE_STATS: [u16; 6] = [
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
pub const MAX_IN_GAME_STATS: [u16; 6] = [
  1999,
  1999,
  999,
  999,
  999,
  99,
];

/// 511
/// Because ATK, DEF, AGI only keep 9 bits value via password transfer, so it's a special case.
const MAX_BASE_STATS_SPECIAL_CASE: u16 = 0b1_1111_1111;

/*
fn base_stats(input_stats: u16, class_multiplier: f64) -> u16 {
  ((((input_stats as f64) + 0.9) / class_multiplier).floor()) as u16
}
*/

pub fn in_game_stats(base_stats: [[u16; 6]; 4]) -> [[u16; 6]; 4] {
  let mut in_game_stats = [[0; 6]; 4];
  for i in 0..6 {
    for j in 0..4 {
      let value = base_stats[j][i];
      let class_multiplier = DEFAULT_CLASS_MULTIPLIERS[j][i];
      let max_in_game_stats = MAX_IN_GAME_STATS[i];
      let max_base_stats_pwd = MAX_BASE_STATS[i];

      let input_stats_f64 = if value > max_base_stats_pwd {
        if max_base_stats_pwd == MAX_BASE_STATS_SPECIAL_CASE {
          value & max_base_stats_pwd
        } else {
          max_base_stats_pwd
        }
      } else {
        value
      } as f64;

      let result = ((input_stats_f64 * class_multiplier).floor()) as u16;

      in_game_stats[j][i] = if result > max_in_game_stats {
        max_in_game_stats
      } else {
        result
      }
    }
  }

  in_game_stats
}
