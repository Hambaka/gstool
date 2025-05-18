use std::collections::HashMap;
use std::env;
use std::fs::{File, read_dir};
use std::io::{BufWriter, Read, Write};
use std::path::Path;

#[rustfmt::skip]
const VALID_PASSWORD_CHARS_JP_EN:[(char, char); 64] = [
  ('あ', 'A'), ('い', 'B'), ('う', 'C'), ('え', 'D'), ('お', 'E'),
  ('か', 'F'), ('き', 'G'), ('く', 'H'), ('け', 'J'), ('こ', 'K'),
  ('さ', 'L'), ('し', 'M'), ('す', 'N'), ('せ', 'P'), ('そ', 'Q'),
  ('た', 'R'), ('ち', 'S'), ('つ', 'T'), ('て', 'U'), ('と', 'V'),
  ('な', 'W'), ('に', 'X'), ('ぬ', 'Y'), ('ね', 'Z'), ('の', '2'),
  ('は', '3'), ('ひ', '4'), ('ふ', '5'), ('へ', '6'), ('ほ', '7'),
  ('ま', '8'), ('み', '9'), ('む', 'a'), ('め', 'b'), ('も', 'c'),
  ('や', 'd'), ('ゆ', 'e'), ('よ', 'f'),
  ('ら', 'g'), ('り', 'h'), ('る', 'i'), ('れ', 'j'), ('ろ', 'k'),
  ('わ', 'm'), ('ん', 'n'), ('を', 'p'),
  ('が', 'q'), ('ぎ', 'r'), ('ぐ', 's'), ('げ', 't'), ('ご', 'u'),
  ('ざ', 'v'), ('じ', 'w'), ('ず', 'x'), ('ぜ', 'y'), ('ぞ', 'z'),
  ('だ', '!'), ('で', '?'), ('ど', '#'),
  ('ば', '&'), ('び', '$'), ('ぶ', '%'), ('べ', '+'), ('ぼ', '=')
];

#[rustfmt::skip]
const VALID_PASSWORD_CHARS_EN_JP: [(char, char); 64] = [
  ('A', 'あ'), ('B', 'い'), ('C', 'う'), ('D', 'え'), ('E', 'お'),
  ('F', 'か'), ('G', 'き'), ('H', 'く'), ('J', 'け'), ('K', 'こ'),
  ('L', 'さ'), ('M', 'し'), ('N', 'す'), ('P', 'せ'), ('Q', 'そ'),
  ('R', 'た'),
  ('S', 'ち'), ('T', 'つ'), ('U', 'て'), ('V', 'と'), ('W', 'な'),
  ('X', 'に'), ('Y', 'ぬ'), ('Z', 'ね'), ('2', 'の'), ('3', 'は'),
  ('4', 'ひ'), ('5', 'ふ'), ('6', 'へ'), ('7', 'ほ'), ('8', 'ま'),
  ('9', 'み'),
  ('a', 'む'), ('b', 'め'), ('c', 'も'), ('d', 'や'), ('e', 'ゆ'),
  ('f', 'よ'), ('g', 'ら'), ('h', 'り'), ('i', 'る'), ('j', 'れ'),
  ('k', 'ろ'), ('m', 'わ'), ('n', 'ん'), ('p', 'を'), ('q', 'が'),
  ('r', 'ぎ'),
  ('s', 'ぐ'), ('t', 'げ'), ('u', 'ご'), ('v', 'ざ'), ('w', 'じ'),
  ('x', 'ず'), ('y', 'ぜ'), ('z', 'ぞ'), ('!', 'だ'), ('?', 'で'),
  ('#', 'ど'), ('&', 'ば'), ('$', 'び'), ('%', 'ぶ'), ('+', 'べ'),
  ('=', 'ぼ')
];

#[rustfmt::skip]
const VALID_PASSWORD_CHARS_JP_BYTE:[(char, u8); 64] = [
  ('あ', 0x00), ('い', 0x01), ('う', 0x02), ('え', 0x03), ('お', 0x04),
  ('か', 0x05), ('き', 0x06), ('く', 0x07), ('け', 0x08), ('こ', 0x09),
  ('さ', 0x0A), ('し', 0x0B), ('す', 0x0C), ('せ', 0x0D), ('そ', 0x0E),
  ('た', 0x0F), ('ち', 0x10), ('つ', 0x11), ('て', 0x12), ('と', 0x13),
  ('な', 0x14), ('に', 0x15), ('ぬ', 0x16), ('ね', 0x17), ('の', 0x18),
  ('は', 0x19), ('ひ', 0x1A), ('ふ', 0x1B), ('へ', 0x1C), ('ほ', 0x1D),
  ('ま', 0x1E), ('み', 0x1F), ('む', 0x20), ('め', 0x21), ('も', 0x22),
  ('や', 0x23), ('ゆ', 0x24), ('よ', 0x25),
  ('ら', 0x26), ('り', 0x27), ('る', 0x28), ('れ', 0x29), ('ろ', 0x2A),
  ('わ', 0x2B), ('ん', 0x2C), ('を', 0x2D),
  ('が', 0x2E), ('ぎ', 0x2F), ('ぐ', 0x30), ('げ', 0x31), ('ご', 0x32),
  ('ざ', 0x33), ('じ', 0x34), ('ず', 0x35), ('ぜ', 0x36), ('ぞ', 0x37),
  ('だ', 0x38), ('で', 0x39), ('ど', 0x3A),
  ('ば', 0x3B), ('び', 0x3C), ('ぶ', 0x3D), ('べ', 0x3E), ('ぼ', 0x3F)
];

#[rustfmt::skip]
const VALID_PASSWORD_CHARS_EN_BYTE: [(char, u8); 64] = [
  ('A', 0x00), ('B', 0x01), ('C', 0x02), ('D', 0x03), ('E', 0x04),
  ('F', 0x05), ('G', 0x06), ('H', 0x07), ('J', 0x08), ('K', 0x09),
  ('L', 0x0A), ('M', 0x0B), ('N', 0x0C), ('P', 0x0D), ('Q', 0x0E),
  ('R', 0x0F),
  ('S', 0x10), ('T', 0x11), ('U', 0x12), ('V', 0x13), ('W', 0x14),
  ('X', 0x15), ('Y', 0x16), ('Z', 0x17), ('2', 0x18), ('3', 0x19),
  ('4', 0x1A), ('5', 0x1B), ('6', 0x1C), ('7', 0x1D), ('8', 0x1E),
  ('9', 0x1F),
  ('a', 0x20), ('b', 0x21), ('c', 0x22), ('d', 0x23), ('e', 0x24),
  ('f', 0x25), ('g', 0x26), ('h', 0x27), ('i', 0x28), ('j', 0x29),
  ('k', 0x2A), ('m', 0x2B), ('n', 0x2C), ('p', 0x2D), ('q', 0x2E),
  ('r', 0x2F),
  ('s', 0x30), ('t', 0x31), ('u', 0x32), ('v', 0x33), ('w', 0x34),
  ('x', 0x35), ('y', 0x36), ('z', 0x37), ('!', 0x38), ('?', 0x39),
  ('#', 0x3A), ('&', 0x3B), ('$', 0x3C), ('%', 0x3D), ('+', 0x3E),
  ('=', 0x3F)
];

fn main() {
  gen_pwd_tables();
  gen_name_tables();
  compress_font();
}

fn gen_pwd_tables() {
  let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen_pwd.rs");
  let mut file = BufWriter::new(File::create(&path).unwrap());

  writeln!(&mut file, "use phf::phf_map;\n").unwrap();

  // VALID_PASSWORD_CHARS_JP_EN_MAP
  writeln!(
    &mut file,
    "static VALID_PASSWORD_CHARS_JP_EN_MAP: phf::Map<char, char> = phf_map! {{"
  )
  .unwrap();
  for (key, value) in VALID_PASSWORD_CHARS_JP_EN {
    writeln!(&mut file, "  '{key}' => '{value}',").unwrap();
  }
  writeln!(&mut file, "}};\n").unwrap();

  // VALID_PASSWORD_CHARS_EN_JP_MAP
  writeln!(
    &mut file,
    "static VALID_PASSWORD_CHARS_EN_JP_MAP: phf::Map<char, char> = phf_map! {{"
  )
  .unwrap();
  for (key, value) in VALID_PASSWORD_CHARS_EN_JP {
    writeln!(&mut file, "  '{key}' => '{value}',").unwrap();
  }
  writeln!(&mut file, "}};\n").unwrap();

  // VALID_PASSWORD_CHARS_JP_BYTE_MAP
  writeln!(
    &mut file,
    "static VALID_PASSWORD_CHARS_JP_BYTE_MAP: phf::Map<char, u8> = phf_map! {{"
  )
  .unwrap();
  for (key, value) in VALID_PASSWORD_CHARS_JP_BYTE {
    writeln!(&mut file, "  '{key}' => 0x{value:02X},").unwrap();
  }
  writeln!(&mut file, "}};\n").unwrap();

  // VALID_PASSWORD_CHARS_EN_BYTE_MAP
  writeln!(
    &mut file,
    "static VALID_PASSWORD_CHARS_EN_BYTE_MAP: phf::Map<char, u8> = phf_map! {{"
  )
  .unwrap();
  for (key, value) in VALID_PASSWORD_CHARS_EN_BYTE {
    writeln!(&mut file, "  '{key}' => 0x{value:02X},").unwrap();
  }
  writeln!(&mut file, "}};").unwrap();
}

fn gen_name_tables() {
  // val -> u8
  let base_characters = serde_json::from_slice::<HashMap<String, String>>(include_bytes!(
    r"resources/json/base_characters.json"
  ))
  .unwrap();
  // val -> u16
  let special_hanzi = serde_json::from_slice::<HashMap<String, String>>(include_bytes!(
    r"resources/json/special_hanzi.json"
  ))
  .unwrap();
  let common_hanzi = serde_json::from_slice::<HashMap<String, String>>(include_bytes!(
    r"resources/json/common_hanzi.json"
  ))
  .unwrap();
  let dakuten_kana = serde_json::from_slice::<HashMap<String, String>>(include_bytes!(
    r"resources/json/dakuten_kana.json"
  ))
  .unwrap();

  let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen_name.rs");
  let mut file = BufWriter::new(File::create(&path).unwrap());

  writeln!(&mut file, "use phf::phf_map;\n").unwrap();

  // BASE_CHARACTERS_MAP
  writeln!(
    &mut file,
    "static BASE_CHARACTERS_MAP: phf::Map<char, u8> = phf_map! {{"
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
    "static SPECIAL_HANZI_MAP: phf::Map<char, u16> = phf_map! {{"
  )
  .unwrap();
  for (key, value) in special_hanzi {
    writeln!(&mut file, "  '{key}' => 0x{value},").unwrap();
  }
  writeln!(&mut file, "}};\n").unwrap();

  // COMMON_HANZI_MAP
  writeln!(
    &mut file,
    "static COMMON_HANZI_MAP: phf::Map<char, &'static str> = phf_map! {{"
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
    "static DAKUTEN_KANA_MAP: phf::Map<char, &'static str> = phf_map! {{"
  )
  .unwrap();
  for (key, value) in dakuten_kana {
    writeln!(&mut file, "  '{key}' => \"{value}\",").unwrap();
  }
  writeln!(&mut file, "}};\n").unwrap();

  // keyboard images
  writeln!(
    &mut file,
    "static KEYBOARD_IMG_MAP: phf::Map<u8, &'static [u8]> = phf_map! {{"
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
    let img_path = Path::new(env!("CARGO_MANIFEST_DIR"))
      .join("assets")
      .join("keyboard_img")
      .join(format!("0x{:02X}.png", i));
    let img_path_str = img_path.as_os_str().to_str().unwrap();
    let new_str = if img_path_str.contains("\\") {
      img_path_str.replace("\\", "\\\\")
    } else {
      img_path_str.to_owned()
    };

    writeln!(&mut file, "  0x{i:02X}u8 => include_bytes!(\"{new_str}\"),").unwrap();
  }
  writeln!(&mut file, "}};").unwrap();
}

fn compress_font() {
  // Find font
  let paths = read_dir("assets/font").unwrap();
  let mut input_path = None;

  // Pick the font file in the folder.
  for ele in paths {
    let path = ele.unwrap().path();
    if path.is_file() {
      if let Some(os_str) = path.extension() {
        match os_str.to_str().unwrap() {
          "otf" | "ttf" => {
            input_path = Some(path);
            break;
          }
          _ => continue,
        }
      }
    }
  }

  if input_path.is_none() {
    eprintln!("Font file not found!")
  }

  // Load font.
  let input_font_path = input_path.unwrap();
  let mut input_file = File::open(input_font_path).expect("Failed to open the font file!");
  let mut input_vec = Vec::new();
  input_file.read_to_end(&mut input_vec).unwrap();

  /*
  1, 3, 7, 11

  Use ZSTD compression levels - Amazon Athena
  https://docs.aws.amazon.com/athena/latest/ug/compression-support-zstd-levels.html
  https://docs.aws.amazon.com/zh_cn/athena/latest/ug/compression-support-zstd-levels.html

  ------------

  ZSTD Decompression Speed vs Compression Level #4091
  https://github.com/facebook/zstd/issues/4091

  VipulGarg869 opened on Jul 5, 2024
  "...The documentation states that the decompression speed doesn't change much with the compression level.
  While that is roughly the case, I was still able to find that the decompression speed decreases up till compression level 6.
  Beyond level 6, it increases. What is the reason for this?"

  Cyan4973 on Jul 5, 2024 · edited by Cyan4973
  "A variation in the number of sequences produced.
  Best decompression speed is typically achieved at level 15,
  though some sources may prefer level 1."
  */
  // Compress with zstd and write.
  let compressed_font = zstd::encode_all(input_vec.as_slice(), 22).unwrap();
  let output_path = Path::new(&env::var("OUT_DIR").unwrap()).join("font.zstd");
  let mut output_file = File::create(output_path).expect("Create \"font.zstd\" failed!");
  output_file
    .write_all(&compressed_font)
    .expect("Write to \"font.zstd\" failed!");
}
