use std::fmt;

use crate::core::{
  bits::BitVec,
  enums::PasswordGrade,
  password::{
    PasswordData,
    items::{PSYNERGY_ITEMS, STACKABLE_ITEMS},
    text::contains_invalid_byte,
  },
};

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
const PASSWORD_CHECKSUM_POSITIONS: [usize; 26] = [
  9, 18, 27, 36, 45, 54, 63, 72, 81, 90, 99, 108, 117, 126, 135, 144, 153, 162, 171, 180, 189, 198,
  207, 216, 225, 234,
];

#[derive(Debug)]
pub enum PasswordBytesError {
  /// 长度不是16/61/260
  BadSize(usize),
  /// 密码中包含非法字节
  InvalidByte,
  InvalidPassword,
}

impl fmt::Display for PasswordBytesError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::BadSize(len) => write!(f, "密码的长度应为16/61/260字节！（实际{len}字节）"),
      Self::InvalidByte => write!(f, "密码包含无效字节！"),
      Self::InvalidPassword => write!(f, "密码无效！"),
    }
  }
}

pub struct ParsedPasswordBytes {
  pub bytes: Vec<u8>,
  pub bits: BitVec,
  pub grade: PasswordGrade,
}

pub fn parse_password_bytes(bytes: Vec<u8>) -> Result<ParsedPasswordBytes, PasswordBytesError> {
  let len = bytes.len();
  let grade = PasswordGrade::from_size(len).ok_or(PasswordBytesError::BadSize(len))?;
  if contains_invalid_byte(&bytes) {
    return Err(PasswordBytesError::InvalidByte);
  }

  let bits = bits_from_bytes(&bytes, grade, true).ok_or(PasswordBytesError::InvalidPassword)?;
  Ok(ParsedPasswordBytes { bytes, bits, grade })
}

pub fn bits_from_bytes(
  password_bytes: &[u8],
  password_grade: PasswordGrade,
  check_validity: bool,
) -> Option<BitVec> {
  let size = password_bytes.len();

  let mut bytes_data = vec![0; size];
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
      for i in PASSWORD_CHECKSUM_POSITIONS {
        bytes_data.remove(i);
      }
    }
    PasswordGrade::Silver => {
      // Or you can use: for i in 9..61.step_by(9)
      for index in PASSWORD_CHECKSUM_POSITIONS.iter().take(6) {
        bytes_data.remove(*index);
      }
    }
    PasswordGrade::Bronze => {
      bytes_data.remove(PASSWORD_CHECKSUM_POSITIONS[0]);
    }
  }

  // Convert bytes to bit array
  let size_without_checksum_bytes = match password_grade {
    // Gold: 260 - 26 = 234 => 173
    PasswordGrade::Gold => 173,
    // Silver: 61 - 6 = 55 => 39
    PasswordGrade::Silver => 39,
    // Bronze: 16 - 1 = 15 => 9
    PasswordGrade::Bronze => 9,
  };
  let mut bits = BitVec::new();
  let max_size = (size_without_checksum_bytes + 2) * 8;

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
  for i in 0..=size_without_checksum_bytes {
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

    for i in 0..size_without_checksum_bytes {
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

pub fn read_from_bits(bits: &BitVec, grade: PasswordGrade) -> PasswordData {
  let mut bits = bits.clone();
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
  match grade {
    PasswordGrade::Gold => {
      /* (2 * 8) + 40
      -> 16 + 40
      -> 56 */
      for _i in 0..56 {
        bits.remove(bits.len() - 1);
      }
      /* (4 * 7) + (4 * 7) + (2 + 6) + (4 * (11 + 11 + 10 + 10 + 10 + 8)) + 8 - 1
      -> 28 + 28 + 8 + 240 + 8 - 1
      -> 311 */
      for i in (1..=8).rev() {
        bits.remove(311 + i * 64);
      }
      /* (4 * 7) + (4 * 7) + (2 + 6) + (4 * (11 + 11 + 10 + 10 + 10 + 8))
      -> 28 + 28 + 8 + 240
      -> 304 */
      for _i in 0..8 {
        bits.remove(304);
      }
      /* (4 * 7) + (4 * 7)
      -> 28 + 28
      -> 56 */
      for _i in 0..2 {
        bits.remove(56);
      }
    }
    PasswordGrade::Silver | PasswordGrade::Bronze => {
      /* (2 * 8)
      -> 16 */
      for _i in 0..16 {
        bits.remove(bits.len() - 1);
      }
      /* (4 * 7) + (4 * 7)
      -> 28 + 28
      -> 56 */
      for _i in 0..2 {
        bits.remove(56);
      }
    }
  }

  /* Get level bits, rearrange it to normal order.
  Each level value has 7 bits.
  Order is: Mia, Ivan, Garet, Isaac */
  let mut level_bits = BitVec::new();
  level_bits.push_bits(u32::from(bits.sub_bits_u8(24, 27)), 4);
  for i in (7..=23).rev().step_by(8) {
    level_bits.push_bits(u32::from(bits.sub_bits_u8(i - 7, i)), 8);
  }
  for (i, level) in levels.iter_mut().enumerate() {
    *level = level_bits.sub_bits_u8(21 - i * 7, 27 - i * 7);
  }

  /* Get djinn bits, rearrange it to normal order.
  Each element has 7 bits.
  Order is: Jupiter, Mars, Mercury, Venus */
  let mut djinn_bits = BitVec::new();
  for i in (39..=55).rev().step_by(8) {
    djinn_bits.push_bits(u32::from(bits.sub_bits_u8(i - 7, i)), 8);
  }
  djinn_bits.push_bits(u32::from(bits.sub_bits_u8(28, 31)), 4);
  for (i, djinn_data) in djinn.iter_mut().enumerate() {
    *djinn_data = u32::from(djinn_bits.sub_bits_u8(21 - i * 7, 27 - i * 7));
  }

  /* Get all event bits.
  Order is: Vault, Vale, Deadbeard, Hsu, Colosso, Hammet */
  for (i, event) in events.iter_mut().enumerate() {
    *event = bits.bit_at(61 - i);
  }

  /* If password grade is silver or bronze, check 8 psynergy items.
  if it does has one, insert it back to Isaac's inventory */
  if grade.include_psy_items() {
    /* Isaac's inventory: Great Sword, Steel Armor, Knight's Shield, Knight's Helm
    So the start index is 4 */
    let mut isaac_inventory_start_index = 4;
    for (i, psynergy_item) in PSYNERGY_ITEMS.iter().enumerate() {
      // Reversed order
      if bits.bit_at(62 + (7 - i)) == 1 {
        items[0][isaac_inventory_start_index] = *psynergy_item;
        isaac_inventory_start_index += 1;
      }
    }
  }

  // If password grade is gold or silver, get stats.
  if grade.include_stats() {
    let start_index = if grade == PasswordGrade::Gold {
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
      stats_per_person[0] = bits.sub_bits_u16(start_index + i * 60, start_index + 10 + i * 60);
      // EP
      stats_per_person[1] = bits.sub_bits_u16(start_index + 11 + i * 60, start_index + 21 + i * 60);
      // Attack
      stats_per_person[2] = bits.sub_bits_u16(start_index + 22 + i * 60, start_index + 31 + i * 60);
      // Defense
      stats_per_person[3] = bits.sub_bits_u16(start_index + 32 + i * 60, start_index + 41 + i * 60);
      // Agility
      stats_per_person[4] = bits.sub_bits_u16(start_index + 42 + i * 60, start_index + 51 + i * 60);
      // Luck
      stats_per_person[5] = bits.sub_bits_u16(start_index + 52 + i * 60, start_index + 59 + i * 60);
    }
  }

  if grade == PasswordGrade::Gold {
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
        let item_id = bits.sub_bits_u16(
          item_id_start_index + i * 135 + j * 9,
          item_id_start_index + 8 + i * 135 + j * 9,
        );
        if let Some(stackable_item_index) = STACKABLE_ITEMS.iter().position(|&x| x == item_id) {
          let item_quantity = bits.sub_bits_u8(
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
    coins = bits.sub_bits_u32(bits.len() - 24, bits.len() - 1);
  }

  PasswordData {
    levels,
    djinn,
    events,
    stats,
    items,
    coins,
  }
}
