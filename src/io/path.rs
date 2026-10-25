use std::path::{Path, PathBuf};

use crate::core::rom::patch::PATCHED_FILE_STEM;

pub fn slot_dir_name(slot_index: usize, is_clear_data: bool) -> String {
  // 0, 1, 2 -> 1, 2, 3
  let slot_num = slot_index + 1;

  if is_clear_data {
    format!("存档{slot_num:02}（通关）")
  } else {
    format!("存档{slot_num:02}")
  }
}

pub fn derived_output_dir(input_file: &Path) -> PathBuf {
  let parent = input_file.parent().unwrap();
  let file_stem = input_file.file_stem().unwrap().to_string_lossy();
  let new_dir_name = format!("{file_stem}_输出");

  parent.join(new_dir_name)
}

/// 输出目录：用户显式指定了就用它，否则用输入文件所在目录
pub fn resolve_output_dir(input_file: &Path, out_dir: Option<&Path>) -> PathBuf {
  out_dir.map_or_else(
    || derived_output_dir(input_file),
    std::path::Path::to_path_buf,
  )
}

/// 在输入文件名的基础上加后缀：`gs2.gba` + `_金银铜` -> `gs2_金银铜.gba`（保留原扩展名）
pub fn suffixed_file_name(input_file: &Path, suffix: &str) -> String {
  let file_stem = input_file.file_stem().unwrap();

  let file_extension = if input_file.extension().is_none() {
    ""
  } else {
    input_file.extension().unwrap().to_str().unwrap()
  };

  let ext = if file_extension.is_empty() {
    file_extension.to_string()
  } else {
    format!(".{file_extension}")
  };

  format!("{}_{suffix}{}", file_stem.display(), ext)
}

/// 《开启的封印》新汉化版 ROM 的输出文件名
pub fn patched_rom_file_name() -> String {
  format!("{PATCHED_FILE_STEM}.gba")
}
