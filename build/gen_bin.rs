use std::{env, fs::File, io::BufWriter, io::Write, path::Path};

use crate::MAIN_DIR;

const DIFF_U32_VALUE: [&[u32]; 7] = [
  &[0x0134, 0x00F4, 0x00E0],
  &[0x00E8, 0x00A8],
  &[0x00BC, 0x00A8],
  &[0x00BC, 0x00A8],
  &[0x0070],
  &[0x0070],
  &[0x0070],
];

const FILE_STEM: [&str; 7] = ["gsb", "gs", "gb", "sb", "g", "s", "b"];

#[rustfmt::skip]
const FILE_CRC32: [u32; 16] = [
  0x830B795F, // 日
  0x606A1C4D, // 美
  0xD981D889, // 德
  0xEE38BA94, // 西
  0x1090BD33, // 法
  0xA3E49743, // 意
  0x8C9EC3DA, // 日（VC）
  0x726BB764, // 美（VC）
  0xC0B98B76, // 民汉；简
  0x5C78240C, // 民汉；繁
  0xEE55D9D3, // 民汉；贴吧；SOMA
  0x25CA744D, // 民汉；未知来源；32MB
  0x0D4CC274, // 民韩；AB1
  0x32DF3963, // 民韩；AB2
  0x5DD28024, // 民韩；AL1
  0x62417B33, // 民韩；AL2
];

#[rustfmt::skip]
const ROM_NAME: [&str; 16] = [
  "黄金太阳 - 失落的时代 (日)",
  "黄金太阳 - 失落的时代 (美)",
  "黄金太阳 - 失落的时代 (德)",
  "黄金太阳 - 失落的时代 (西)",
  "黄金太阳 - 失落的时代 (法)",
  "黄金太阳 - 失落的时代 (意)",
  "黄金太阳 - 失落的时代 (日) (Virtual Console)",
  "黄金太阳 - 失落的时代 (美) (Virtual Console)",
  "黄金太阳 - 失落的时代 (简) [Mobile&Elffinal]/[SOMA]",
  "黄金太阳 - 失落的时代 (繁) [Mobile&Elffinal]/[SOMA]",
  "黄金太阳 - 失落的时代 (简) [Mobile&Elffinal]/[SOMA]", // 贴吧；SOMA
  "黄金太阳 - 失落的时代 (简) [Mobile&Elffinal]/[SOMA]", // 未知来源；32MB
  "黄金太阳 - 失落的时代 (韩) (AB1) [pjs0493]",
  "黄金太阳 - 失落的时代 (韩) (AB2) [pjs0493]",
  "黄金太阳 - 失落的时代 (韩) (AL1) [pjs0493]",
  "黄金太阳 - 失落的时代 (韩) (AL2) [pjs0493]",
];

const GAME_VERSIONS: [&str; 16] = [
  "GameRegion::Japan",
  "GameRegion::USA",
  "GameRegion::Germany",
  "GameRegion::Spain",
  "GameRegion::France",
  "GameRegion::Italy",
  "GameRegion::Japan",
  "GameRegion::USA",
  "GameRegion::USA",
  "GameRegion::USA",
  "GameRegion::USA",
  "GameRegion::USA",
  "GameRegion::Japan",
  "GameRegion::Japan",
  "GameRegion::Japan",
  "GameRegion::Japan",
];

const INJECT_INDEX: [usize; 16] = [
  0x00F789E0, 0x00F79650, 0x00F79700, 0x00F799E0, 0x00F79610, 0x00F78FE0, 0x00F789E0, 0x00F79650,
  0x00FEC410, 0x00FEC410, 0x00FEC410, 0x00FEC410, 0x01481E00, 0x01481E00, 0x01481E00, 0x01481E00,
];

const BLANK_U8: [u8; 16] = [
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
];

pub fn run() {
  let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen_bin.rs");
  let mut file = BufWriter::new(File::create(&path).unwrap());

  writeln!(file, "use phf::phf_map;\n").unwrap();

  let mut pwd_u8_option_vec = Vec::new();
  let mut u8_other_option_vec = Vec::new();

  let mut all_u32_vec = Vec::new();

  let mut pwd_buffer_start_vec = Vec::new();
  let mut pwd_buffer_end_vec = Vec::new();
  let mut pwd_exit_vec = Vec::new();

  let mut pwd_value_pos_option_vec = Vec::new();

  for i in 0..7 {
    let bytes = std::fs::read(format!("resources/bin/{}.bin", FILE_STEM[i])).unwrap();

    let mut gold_u8_option = None;
    let mut silver_u8_option = None;
    let mut bronze_u8_option = None;
    let mut other_u8_vec = Vec::new();

    let mut u32_vec = Vec::new();

    let mut gold_pwd_option = None;
    let mut silver_pwd_option = None;
    let mut bronze_pwd_option = None;

    let mut index = 0;
    let len = bytes.len();

    while index < len {
      let value = bytes[index];
      match value {
        0x71 => gold_u8_option = Some(index),
        0x72 => silver_u8_option = Some(index),
        0x73 => bronze_u8_option = Some(index),
        0x74 => other_u8_vec.push(index),
        0x91 | 0x92 | 0x93 => {
          u32_vec.push(index);
          index += 4;
          continue;
        }
        0xA1 => {
          pwd_buffer_start_vec.push(index);
          index += 4;
          continue;
        }
        0xA2 => {
          pwd_buffer_end_vec.push(index);
          index += 4;
          continue;
        }
        0xA3 => {
          pwd_exit_vec.push(index);
          index += 4;
          continue;
        }
        0xB1 => {
          gold_pwd_option = Some(index);
          index += 260;
          continue;
        }
        0xB2 => {
          silver_pwd_option = Some(index);
          index += 61;
          continue;
        }
        0xB3 => {
          bronze_pwd_option = Some(index);
          index += 16;
          continue;
        }
        _ => {}
      }

      index += 1;
    }

    pwd_u8_option_vec.push([gold_u8_option, silver_u8_option, bronze_u8_option]);
    u8_other_option_vec.push(other_u8_vec);
    all_u32_vec.push(u32_vec);
    pwd_value_pos_option_vec.push([gold_pwd_option, silver_pwd_option, bronze_pwd_option]);
  }

  writeln!(file, "const INJECT_TEMPLATES: [InjectTemplate; 7] = [").unwrap();
  for i in 0..7 {
    writeln!(file, "  InjectTemplate {{ grade_placeholder_positions: {}, misc_placeholder_positions: {}, address_patch_positions: {}, pwd_buffer_start_pos: {}, pwd_buffer_end_pos: {}, exit_address_pos: {}, password_slot_positions: {} }},",
     get_diff_u8_gsb_str(i, &pwd_u8_option_vec),
     get_diff_u8_other_str(i, &u8_other_option_vec),
     get_diff_u32_str(i, &all_u32_vec),
     format!("{:#04X}", pwd_buffer_start_vec[i]),
     format!("{:#04X}", pwd_buffer_end_vec[i]),
     format!("{:#04X}", pwd_exit_vec[i]),
     get_pwd_pos_str(i, &pwd_value_pos_option_vec),
    ).unwrap();
  }
  writeln!(file, "];\n").unwrap();

  writeln!(file, "const INJECT_PAYLOADS: [&'static [u8]; 7] = [").unwrap();

  for i in 0..7 {
    let bin_path = Path::new(MAIN_DIR)
      .join("resources")
      .join("bin")
      .join(format!("{}.bin", FILE_STEM[i]));
    let bin_path_str = bin_path.as_os_str().to_str().unwrap();
    let new_str = if bin_path_str.contains("\\") {
      bin_path_str.replace("\\", "\\\\")
    } else {
      bin_path_str.to_owned()
    };

    writeln!(&mut file, "  include_bytes!(\"{new_str}\"),").unwrap();
  }

  writeln!(file, "];").unwrap();

  writeln!(file, "const BASE_ROM_NAME: [&str; 6] = [").unwrap();
  for i in 0..6 {
    writeln!(&mut file, "  \"{}\",", ROM_NAME[i]).unwrap();
  }
  writeln!(file, "];").unwrap();

  writeln!(
    file,
    "static KNOWN_INPUT_ROM_INFO_MAP: phf::Map<u32, (&'static str, GameRegion, usize, u8)> = phf_map! {{"
  )
  .unwrap();
  for i in 0..16 {
    writeln!(
      &mut file,
      "  {:#010X}u32 => (\"{}\", {}, {:#010X}, {:#04X}),",
      FILE_CRC32[i], ROM_NAME[i], GAME_VERSIONS[i], INJECT_INDEX[i], BLANK_U8[i]
    )
    .unwrap();
  }
  writeln!(file, "}};").unwrap();
}

fn get_diff_u8_gsb_str(index: usize, vec: &Vec<[Option<usize>; 3]>) -> String {
  let mut result = String::new();
  result.push_str("[");

  let temp = vec[index];
  if let Some(gold) = temp[0] {
    result.push_str(&format!("Some({gold:#04X}), "));
  } else {
    result.push_str("None,");
  }
  if let Some(silver) = temp[1] {
    result.push_str(&format!("Some({silver:#04X}), "));
  } else {
    result.push_str("None,");
  }
  if let Some(bronze) = temp[2] {
    result.push_str(&format!("Some({bronze:#04X})"));
  } else {
    result.push_str("None");
  }

  result.push_str("]");
  result
}

fn get_diff_u8_other_str(index: usize, vec: &Vec<Vec<usize>>) -> String {
  let mut result = String::new();
  result.push_str("&[");

  let temp = &vec[index];
  let len = temp.len();
  for i in 0..len {
    if i + 1 == len {
      result.push_str(&format!("{:#04X}", temp[i]));
    } else {
      result.push_str(&format!("{:#04X}, ", temp[i]));
    }
  }

  result.push_str("]");
  result
}

fn get_diff_u32_str(index: usize, vec: &Vec<Vec<usize>>) -> String {
  let mut result = String::new();
  result.push_str("&[");

  let index_vec = &vec[index];
  let values = DIFF_U32_VALUE[index];
  let len = index_vec.len();
  for i in 0..len {
    if i + 1 == len {
      result.push_str(&format!("({:#04X}, {:#06X})", index_vec[i], values[i]));
    } else {
      result.push_str(&format!("({:#04X}, {:#06X}), ", index_vec[i], values[i]));
    }
  }

  result.push_str("]");
  result
}

fn get_pwd_pos_str(index: usize, vec: &Vec<[Option<usize>; 3]>) -> String {
  let mut result = String::new();
  result.push_str("[");

  let temp = vec[index];
  let len = temp.len();
  for i in 0..len {
    if i + 1 == len {
      if let Some(value) = temp[i] {
        result.push_str(&format!("Some({value:#04X})"));
      } else {
        result.push_str("None");
      }
    } else {
      if let Some(value) = temp[i] {
        result.push_str(&format!("Some({value:#04X}), "));
      } else {
        result.push_str("None, ");
      }
    }
  }

  result.push_str("]");
  result
}
