use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::MAIN_DIR;

pub fn run() {
  // val -> u8
  let base_characters = serde_json::from_slice::<HashMap<String, String>>(include_bytes!(
    "../resources/json/char/base_characters.json"
  ))
  .unwrap();
  // val -> u16
  let special_hanzi = serde_json::from_slice::<HashMap<String, String>>(include_bytes!(
    "../resources/json/char/special_hanzi.json"
  ))
  .unwrap();
  let common_hanzi = serde_json::from_slice::<HashMap<String, String>>(include_bytes!(
    "../resources/json/char/common_hanzi.json"
  ))
  .unwrap();
  let dakuten_kana = serde_json::from_slice::<HashMap<String, String>>(include_bytes!(
    "../resources/json/char/dakuten_kana.json"
  ))
  .unwrap();

  // ============================================================
  // codegen_name_chars.rs：4 张字符编码表
  // ============================================================
  let chars_path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen_name_chars.rs");
  let mut file = BufWriter::new(File::create(&chars_path).unwrap());

  writeln!(&mut file, "use phf::phf_map;\n").unwrap();

  // BASE_CHARACTERS_MAP
  writeln!(
    &mut file,
    "pub static BASE_CHARACTERS_MAP: phf::Map<char, u8> = phf_map! {{"
  )
  .unwrap();
  for (key, value) in base_characters {
    if "'".eq(&key) {
      writeln!(&mut file, "  '\\{key}' => 0x{value},").unwrap();
    } else {
      writeln!(&mut file, "  '{key}' => 0x{value},").unwrap();
    }
  }
  writeln!(&mut file, "}};\n").unwrap();

  // SPECIAL_HANZI_MAP
  writeln!(
    &mut file,
    "pub static SPECIAL_HANZI_MAP: phf::Map<char, u16> = phf_map! {{"
  )
  .unwrap();
  for (key, value) in special_hanzi {
    writeln!(&mut file, "  '{key}' => 0x{value},").unwrap();
  }
  writeln!(&mut file, "}};\n").unwrap();

  // COMMON_HANZI_MAP
  writeln!(
    &mut file,
    "pub static COMMON_HANZI_MAP: phf::Map<char, &'static str> = phf_map! {{"
  )
  .unwrap();
  for (key, value) in common_hanzi {
    if value.contains('"') {
      if '"'.eq(&value.chars().next().unwrap()) {
        writeln!(&mut file, "  '{key}' => \"\\{value}\",").unwrap();
      } else {
        writeln!(
          &mut file,
          "  '{key}' => \"{}\\{}\",",
          value.chars().next().unwrap(),
          value.chars().nth(1).unwrap()
        )
        .unwrap();
      }
    } else {
      writeln!(&mut file, "  '{key}' => \"{value}\",").unwrap();
    }
  }
  writeln!(&mut file, "}};\n").unwrap();

  // DAKUTEN_KANA_MAP
  writeln!(
    &mut file,
    "pub static DAKUTEN_KANA_MAP: phf::Map<char, &'static str> = phf_map! {{"
  )
  .unwrap();
  for (key, value) in dakuten_kana {
    writeln!(&mut file, "  '{key}' => \"{value}\",").unwrap();
  }
  writeln!(&mut file, "}};\n").unwrap();

  // ============================================================
  // codegen_name_kbd.rs：键盘按键图
  // ============================================================
  let kbd_path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen_name_kbd.rs");
  let mut file = BufWriter::new(File::create(&kbd_path).unwrap());

  writeln!(&mut file, "use phf::phf_map;\n").unwrap();
  writeln!(
    &mut file,
    "pub static KEYBOARD_IMG_MAP: phf::Map<u8, &'static [u8]> = phf_map! {{"
  )
  .unwrap();

  // 60,
  // 7F
  // 80, 81, 82, 83, 84, 85,
  // 90,
  // A0, A1, A2, A3, A4, A5
  let invalid_num = [
    0x60, 0x7F, 0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x90, 0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5,
  ];

  for i in 0x20..0xFE {
    if invalid_num.contains(&i) {
      continue;
    }
    let img_path = Path::new(MAIN_DIR)
      .join("assets")
      .join("name_kbd_img")
      .join(format!("{i:#04X}.png"));
    let img_path_str = img_path.as_os_str().to_str().unwrap();
    let new_str = if img_path_str.contains("\\") {
      img_path_str.replace("\\", "\\\\")
    } else {
      img_path_str.to_owned()
    };

    writeln!(&mut file, "  {i:#04X}u8 => include_bytes!(\"{new_str}\"),").unwrap();
  }
  writeln!(&mut file, "}};").unwrap();
}
