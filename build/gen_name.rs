use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::MAIN_DIR;

pub fn run() {
  // val -> u8
  let char_1_byte_typable = serde_json::from_slice::<HashMap<String, String>>(include_bytes!(
    "../resources/json/char/1_byte_typable.json"
  ))
  .unwrap();

  // val -> u8
  let char_1_byte_untypable = serde_json::from_slice::<HashMap<String, String>>(include_bytes!(
    "../resources/json/char/1_byte_untypable.json"
  ))
  .unwrap();

  let mut char_2_bytes_typable = serde_json::from_slice::<HashMap<String, String>>(include_bytes!(
    "../resources/json/char/2_bytes_typable.json"
  ))
  .unwrap();
  let dakuten_kana = serde_json::from_slice::<HashMap<String, String>>(include_bytes!(
    "../resources/json/char/dakuten_kana.json"
  ))
  .unwrap();
  char_2_bytes_typable.extend(dakuten_kana);

  // val -> u16
  let char_2_bytes_untypable = serde_json::from_slice::<HashMap<String, String>>(include_bytes!(
    "../resources/json/char/2_bytes_untypable.json"
  ))
  .unwrap();

  // ============================================================
  // codegen_name_chars.rs：4 张字符编码表
  // ============================================================
  let chars_path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen_name_chars.rs");
  let mut file = BufWriter::new(File::create(&chars_path).unwrap());

  writeln!(&mut file, "use phf::phf_map;\n").unwrap();

  // 单字节字符（起名界面可输入；包含用于组合双字节字符的控制符）
  // CHAR_1_BYTE_TYPABLE_MAP
  writeln!(
    &mut file,
    "pub static CHAR_1_BYTE_TYPABLE_MAP: phf::Map<char, u8> = phf_map! {{"
  )
  .unwrap();
  for (key, value) in char_1_byte_typable {
    if "'".eq(&key) {
      writeln!(&mut file, "  '\\{key}' => 0x{value},").unwrap();
    } else {
      writeln!(&mut file, "  '{key}' => 0x{value},").unwrap();
    }
  }
  writeln!(&mut file, "}};\n").unwrap();

  // 单字节字符（起名界面无法输入）
  // CHAR_1_BYTE_UNTYPABLE_MAP
  writeln!(
    &mut file,
    "pub static CHAR_1_BYTE_UNTYPABLE_MAP: phf::Map<char, u8> = phf_map! {{"
  )
  .unwrap();
  for (key, value) in char_1_byte_untypable {
    writeln!(&mut file, "  '{key}' => 0x{value},").unwrap();
  }
  writeln!(&mut file, "}};\n").unwrap();

  // 双字节字符（起名界面可输入）
  // CHAR_2_BYTES_TYPABLE_MAP
  writeln!(
    &mut file,
    "pub static CHAR_2_BYTES_TYPABLE_MAP: phf::Map<char, &'static str> = phf_map! {{"
  )
  .unwrap();
  for (key, value) in char_2_bytes_typable {
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

  // 双字节字符（起名界面无法输入）
  // CHAR_2_BYTES_UNTYPABLE_MAP
  writeln!(
    &mut file,
    "pub static CHAR_2_BYTES_UNTYPABLE_MAP: phf::Map<char, u16> = phf_map! {{"
  )
  .unwrap();
  for (key, value) in char_2_bytes_untypable {
    writeln!(&mut file, "  '{key}' => 0x{value},").unwrap();
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
