use std::env;
use std::fs::{File, read_dir};
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::MAIN_DIR;

const LATEST_CHS_FILE_STEM: &str = "黄金太阳 - 开启的封印 (2024新汉化公测版)";
const JPN_ROM_CRC32: u32 = 0xFB96D9DE;

pub fn run(level: i32) {
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

  // let bps_main_dir_path = Path::new("resources/bps");
  let bps_main_dir_path = Path::new(MAIN_DIR).join("resources/bps");
  // Find all bps files.
  let main_dir = read_dir(bps_main_dir_path).unwrap();
  for ele in main_dir {
    let sub_dir_path = ele.unwrap().path();
    if sub_dir_path.is_dir() {
      let sub_dir = read_dir(sub_dir_path).unwrap();
      for ele in sub_dir {
        let bps_path = ele.unwrap().path();
        if bps_path.is_file()
          && let Some(os_str) = bps_path.extension()
        {
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

  // let compressed_bps = zstd::encode_all(all_bps.as_slice(), level).unwrap();
  let compressed_bps = structured_zstd::encoding::compress_slice_to_vec(
    all_bps.as_slice(),
    structured_zstd::encoding::CompressionLevel::from_level(level),
  );
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
    "pub const LATEST_CHS_CHECKSUM:u32 = 0x{:X};\n",
    all_target_crc32[latest_chinese_patch_index]
  )
  .unwrap();

  writeln!(
    &mut file,
    "pub const VALID_INPUT_ROM_CHECKSUM:[u32;{file_count}] = [",
  )
  .unwrap();
  for i in 0..file_count {
    writeln!(&mut file, "  {:#010X},", all_source_crc32[i]).unwrap();
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
      "  {:#010X}u32 => ({}, {}),",
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
      "  {:#010X}u32 => {:#010X}u32,",
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
        "  {crc32:#010X}u32 => \"黄金太阳 - 开启的封印 (日)\","
      )
      .unwrap();
    } else {
      writeln!(
        &mut file,
        "  {crc32:#010X}u32 => \"{}\",",
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
    "CHS_CM_TB" | "CHS_CM" | "CHS_CM_QO" | "CHS_CM_QFC" | "CHS_CM_QFC2" => {
      "黄金太阳 - 开启的封印 (简) (润色美化版) [冷血无情Keroro]"
    }
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
