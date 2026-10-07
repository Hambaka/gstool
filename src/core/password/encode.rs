use crate::core::{
  bits::BitArray,
  enums::PasswordGrade,
  password::structs::PasswordData,
  save::format::{PSYNERGY_ITEMS, STACKABLE_ITEMS},
};

/* Port from Dyrati's "Golden-Sun-Password-Transfer" lua script for "vba-rr" and "Bizhawk" emulators.
Link: https://github.com/Dyrati/Golden-Sun-Password-Transfer/blob/5ec2d52553ec8f4e0fe77854bc2b31956ac09a11/password.lua#L8 */
fn event_flag(raw_save: &[u8], flag: u32) -> u8 {
  let byte_pos = (flag >> 3) as usize;
  let bit_pos = flag & 7;
  (raw_save[0x40 + byte_pos] >> bit_pos) & 1
}

/* Port from Dyrati's "Golden-Sun-Password-Transfer" lua script for "vba-rr" and "Bizhawk" emulators.
Link: https://github.com/Dyrati/Golden-Sun-Password-Transfer/blob/5ec2d52553ec8f4e0fe77854bc2b31956ac09a11/password.lua#L35 */
pub fn read_from_raw_save(raw_save: &[u8]) -> PasswordData {
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
  events[0] = event_flag(raw_save, 0x941);
  // Beat Colosso (Won the final fight against Navampa)
  events[1] = event_flag(raw_save, 0x951);
  // Ulmuch/Hsu was rescued by Hamo/Hama and Kouran/Feizhi alone, and Robin/Isaac's party didn't lend a hand
  events[2] = event_flag(raw_save, 0x8B3);
  // Beat Talos/Deadbeard
  events[3] = event_flag(raw_save, 0x8D1);
  // Visited Haidia/Vale after entering Kalay (Return to Haidia/Vale and find out that Robin/Isaac's mother's sick.)
  events[4] = event_flag(raw_save, 0x81E);
  // Visited Coopup/Vault after entering Kalay (Return to Coopup/Vault and talk with the mayor about the thieves, and find out that thieves have fled Coopup/Vault.)
  events[5] = event_flag(raw_save, 0x868);

  // u32
  let coins = u32::from_le_bytes([
    raw_save[0x250],
    raw_save[0x251],
    raw_save[0x252],
    raw_save[0x253],
  ]);
  PasswordData {
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
pub fn encode(data: &PasswordData, grade: PasswordGrade) -> Vec<u8> {
  let mut bits = BitArray::new();

  // Insert 7 bits per level, 7 bits per jinn element
  let mut level_bits = BitArray::new();
  let mut djinn_bits = BitArray::new();
  for i in (0..=3).rev() {
    level_bits.push_bits(u32::from(data.levels[i]), 7);
  }

  for i in (0..=3).rev() {
    djinn_bits.push_bits(data.djinn[i], 7);
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
    if i < data.events.len() {
      bits.push_bit(u32::from(data.events[i]));
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
    for items_per_person in &data.items {
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
    for stats_per_person in &data.stats {
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
    for items_per_person in &data.items {
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
    for items_per_person in &data.items {
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
    bits.push_bits(data.coins, 24);
  }

  // Append 0 until reaching the correct password size.
  for _i in 0..8 * size - bits.len() {
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

    if max <= bits.len() {
      entry = bits.sub_bits_u8(i, max);
    } else {
      let last_left_shift = max - max_size + 1;
      max = bits.len() - 1;
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

pub fn from_raw_save(raw_save: &[u8], grade: PasswordGrade) -> Vec<u8> {
  let data = read_from_raw_save(raw_save);
  encode(&data, grade)
}
