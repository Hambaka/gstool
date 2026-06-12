use std::env;
use std::fs::{File, read_dir};
use std::io::{Read, Write};
use std::path::Path;

use crate::MAIN_DIR;

pub fn run(level: i32) {
  // Find font
  // let paths = read_dir("../assets/font").unwrap();
  let paths = read_dir(Path::new(MAIN_DIR).join("assets/font")).unwrap();
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
