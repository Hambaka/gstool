use std::collections::HashMap;
use std::env;
use std::fs::{File, read_dir};
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

const LATEST_CHS_FILE_STEM: &str = "黄金太阳 - 开启的封印 (2024新汉化公测版)";
const JPN_ROM_CRC32: u32 = 0xFB96D9DE;

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

// https://doc.rust-lang.org/cargo/reference/environment-variables.html
fn main() {
  gen_pwd_tables();
  gen_name_tables();

  let is_release = env::var("PROFILE").map(|p| "release" == p).unwrap_or(false);
  let level = if is_release { 22 } else { 3 };

  compress_font(level);
  gen_bps_tables(level);
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
      .join("name_kbd_img")
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

fn compress_font(level: i32) {
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
  let compressed_font = zstd::encode_all(input_vec.as_slice(), level).unwrap();
  let output_path = Path::new(&env::var("OUT_DIR").unwrap()).join("font.zst");
  let mut output_file = File::create(output_path).expect("Create \"font.zst\" failed!");
  output_file
    .write_all(&compressed_font)
    .expect("Write to \"font.zst\" failed!");
}

fn gen_bps_tables(level: i32) {
  let mut all_bps = Vec::new();
  let mut all_file_name = Vec::new();
  let mut start_index = Vec::new();
  let mut all_file_size = Vec::new();
  let mut all_source_crc32 = Vec::new();
  let mut all_target_crc32 = Vec::new();
  let mut all_patch_crc32 = Vec::new();
  let mut file_count = 0;
  let mut file_start_index = 0;
  let mut latest_chinese_patch_index = 0;

  let bps_main_dir_path = Path::new("resources/bps");
  // Find all bps files.
  let main_dir = read_dir(bps_main_dir_path).unwrap();
  for ele in main_dir {
    let sub_dir_path = ele.unwrap().path();
    if sub_dir_path.is_dir() {
      let sub_dir = read_dir(sub_dir_path).unwrap();
      for ele in sub_dir {
        let bps_path = ele.unwrap().path();
        if bps_path.is_file() {
          if let Some(os_str) = bps_path.extension() {
            match os_str.to_str().unwrap() {
              "bps" => {
                // File name...
                let file_name = bps_path.file_stem().unwrap().to_str().unwrap();
                all_file_name.push(file_name.to_owned());
                // Load bps file.
                let mut bps_file = File::open(bps_path).unwrap();
                let metadata = bps_file.metadata().unwrap();

                // Start reading Checksum
                bps_file.seek(SeekFrom::End(-12)).unwrap();

                let mut footer_checksum = [0u8; 4];
                bps_file.read_exact(&mut footer_checksum).unwrap();
                if u32::from_le_bytes(footer_checksum) == JPN_ROM_CRC32 {
                  latest_chinese_patch_index = file_count;
                }
                all_source_crc32.push(u32::from_le_bytes(footer_checksum));

                bps_file.read_exact(&mut footer_checksum).unwrap();
                all_target_crc32.push(u32::from_le_bytes(footer_checksum));

                bps_file.read_exact(&mut footer_checksum).unwrap();
                all_patch_crc32.push(u32::from_le_bytes(footer_checksum));

                // Start index
                start_index.push(file_start_index);
                // File size
                let file_size = metadata.len();
                all_file_size.push(file_size);
                // Update index
                file_start_index += file_size;
                // Append bytes
                bps_file.seek(SeekFrom::Start(0)).unwrap();
                bps_file.read_to_end(&mut all_bps).unwrap();

                file_count += 1;
              }
              _ => continue,
            }
          }
        }
      }
    }
  }

  let compressed_bps = zstd::encode_all(all_bps.as_slice(), level).unwrap();
  let output_path = Path::new(&env::var("OUT_DIR").unwrap()).join("all_bps.zst");
  let mut output_file = File::create(output_path).expect("Create \"all_bps.zst\" failed!");
  output_file
    .write_all(&compressed_bps)
    .expect("Write to \"all_bps.zst\" failed!");

  let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen_bps.rs");
  let mut file = BufWriter::new(File::create(&path).unwrap());

  writeln!(&mut file, "use phf::phf_map;\n").unwrap();
  writeln!(
    &mut file,
    "const LATEST_CHS_FILE_STEM:&str = \"{LATEST_CHS_FILE_STEM}\";"
  )
  .unwrap();
  writeln!(
    &mut file,
    "const LATEST_CHS_CHECKSUM:u32 = 0x{:X};\n",
    all_target_crc32[latest_chinese_patch_index]
  )
  .unwrap();

  writeln!(
    &mut file,
    "const VALID_INPUT_ROM_CHECKSUM:[u32;{file_count}] = [",
  )
  .unwrap();
  for i in 0..file_count {
    writeln!(&mut file, "  0x{:X},", all_source_crc32[i]).unwrap();
  }
  writeln!(&mut file, "];\n").unwrap();

  // Use patch checksum to get patch file start index and file size
  writeln!(
    &mut file,
    "static BPS_FILE_INDEX_MAP: phf::Map<u32, (usize, usize)> = phf_map! {{"
  )
  .unwrap();
  for i in 0..file_count {
    writeln!(
      &mut file,
      "  0x{:08X}u32 => ({}, {}),",
      all_patch_crc32[i], start_index[i], all_file_size[i]
    )
    .unwrap();
  }
  writeln!(&mut file, "}};\n").unwrap();

  // Use input rom's checksum to get patch checksum.
  writeln!(
    &mut file,
    "static BPS_CRC32_MAP: phf::Map<u32, u32> = phf_map! {{"
  )
  .unwrap();
  for i in 0..file_count {
    let crc32 = all_source_crc32[i];
    writeln!(
      &mut file,
      "  0x{:08X}u32 => 0x{:08X}u32,",
      crc32, all_patch_crc32[i],
    )
    .unwrap();
  }
  writeln!(&mut file, "}};\n").unwrap();

  // Checksum + Rom info
  writeln!(
    &mut file,
    "static VALID_INPUT_ROM_INFO_MAP: phf::Map<u32, &'static str> = phf_map! {{"
  )
  .unwrap();
  for i in 0..file_count {
    let crc32 = all_source_crc32[i];
    if crc32 == JPN_ROM_CRC32 {
      writeln!(
        &mut file,
        "  0x{crc32:08X}u32 => \"黄金太阳 - 开启的封印 (日)\","
      )
      .unwrap();
    } else {
      writeln!(
        &mut file,
        "  0x{crc32:08X}u32 => \"{}\",",
        get_non_jpn_rom_info(&all_file_name[i])
      )
      .unwrap();
    }
  }
  writeln!(&mut file, "}};").unwrap();
}

fn get_non_jpn_rom_info(file_stem: &str) -> &'static str {
  match file_stem {
    // Official
    "USA" => "黄金太阳 - 开启的封印 (美)",
    "NOE" => "黄金太阳 - 开启的封印 (德)",
    "ESP" => "黄金太阳 - 开启的封印 (西)",
    "FRA" => "黄金太阳 - 开启的封印 (法)",
    "ITA" => "黄金太阳 - 开启的封印 (意)",
    // Fan translation - Chinese (old)
    "CHS_TC_Xmas" | "CHS_TC" => "黄金太阳 - 开启的封印 (简) [CGP]",
    "CHS_TCnE_O" | "CHS_TCnE_TB" | "CHS_TCnE" => "黄金太阳 - 开启的封印 (简) [CGP&elffinal]",
    "CHS_CM_TB" | "CHS_CM" | "CHS_CM_QO" | "CHS_CM_QFC" | "CHS_CM_QFC2" => "黄金太阳 - 开启的封印 (简) (润色美化版) [冷血无情Keroro]",
    "CHS_CM_U_Pre" => "黄金太阳 - 开启的封印 (简) (美版汉化预览版) [冷血无情Keroro]",
    // Fan translation - Other languages
    "POL" => "黄金太阳 - 开启的封印 (波兰文) [Rykuzushi]",
    "KOR_A1" => "黄金太阳 - 开启的封印 (韩) (A1) [pjs0493]",
    "KOR_A2" => "黄金太阳 - 开启的封印 (韩) (A2) [pjs0493]",
    "KOR_M1" => "黄金太阳 - 开启的封印 (韩) (M1) [pjs0493]",
    "KOR_M2" => "黄金太阳 - 开启的封印 (韩) (M2) [pjs0493]",
    _ => "待填写……",
  }
}
