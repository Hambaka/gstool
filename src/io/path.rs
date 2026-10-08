use std::path::{Path, PathBuf};

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
  let new_dir_name = format!("{}_输出", file_stem);

  parent.join(new_dir_name)
}
