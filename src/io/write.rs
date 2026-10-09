use std::{
  fmt,
  fs::{self, File},
  io::Write,
  path::Path,
};

use crate::{
  core::{
    enums::{GameRegion, PasswordVersion},
    password::text::dmp_to_txt,
  },
  io::render::{cheat_code_text, password_text_beautify},
};

#[derive(Debug, Clone)]
pub enum OutputError {
  NotADirectory(String),
  CreateFileFailed(String),
  CreateDirectoryFailed(String),
  WriteFailed(String),
}

impl fmt::Display for OutputError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      OutputError::NotADirectory(dir) => write!(f, "路径不是目录：{dir}"),
      OutputError::CreateFileFailed(file) => write!(f, "文件创建失败：{file}"),
      OutputError::CreateDirectoryFailed(dir) => write!(f, "目录创建失败：{dir}"),
      OutputError::WriteFailed(file) => write!(f, "文件写入失败：{file}"),
    }
  }
}

fn ensure_dir(dir: &Path) -> Result<(), OutputError> {
  if dir.is_dir() {
    return Ok(());
  }

  if dir.exists() {
    return Err(OutputError::NotADirectory(dir.to_string_lossy().into()));
  }

  match fs::create_dir_all(dir) {
    Ok(_) => Ok(()),
    Err(_) => Err(OutputError::CreateDirectoryFailed(
      dir.to_string_lossy().into(),
    )),
  }
}

fn write_text(dir: &Path, file_name: &str, text: &str) -> Result<(), OutputError> {
  ensure_dir(dir)?;
  let file_path = dir.join(file_name);

  let Ok(mut file) = File::create(&file_path) else {
    return Err(OutputError::CreateFileFailed(
      file_path.to_string_lossy().into(),
    ));
  };

  match file.write_all(text.as_bytes()) {
    Ok(_) => Ok(()),
    Err(_) => Err(OutputError::WriteFailed(file_path.to_string_lossy().into())),
  }
}

fn write_bytes(dir: &Path, file_name: &str, bytes: &[u8]) -> Result<(), OutputError> {
  ensure_dir(dir)?;
  let file_path = dir.join(file_name);

  let Ok(mut file) = File::create(&file_path) else {
    return Err(OutputError::CreateFileFailed(
      file_path.to_string_lossy().into(),
    ));
  };

  match file.write_all(bytes) {
    Ok(_) => Ok(()),
    Err(_) => Err(OutputError::WriteFailed(file_path.to_string_lossy().into())),
  }
}

pub fn write_password_text(
  bytes: &[u8],
  version: PasswordVersion,
  dir: &Path,
) -> Result<(), OutputError> {
  write_converted_password_text(&dmp_to_txt(bytes, version), version, dir)
}

pub fn write_converted_password_text(
  text: &str,
  version: PasswordVersion,
  dir: &Path,
) -> Result<(), OutputError> {
  let output_text = password_text_beautify(text, version);
  write_text(dir, "密码文本.txt", &output_text)
}

/// Write password bytes to a binary file.
/// After you go to password input screen in GS2, you can import it via emulator's memory viewer, so that means you don't have to input password manually.
/// Though you have to choose the correct address and import it, you can check the address in `get_cheat_address` function of "enums.rs".
pub fn write_memory_dump(bytes: &[u8], dir: &Path) -> Result<(), OutputError> {
  write_bytes(dir, "密码内存转储.dmp", bytes)
}

/// Well, cheat file is much easier to use,
/// After you go to password input screen in GS2, you can copy and paste cheat codes in emulators.
/// But you have to remove the cheat codes you just entered immediately after applying it, otherwise it may cause problems.
/// Though I'm not sure, maybe you can use this kind of raw cheat code on your phone as well?
pub fn write_cheat_code(bytes: &[u8], version: GameRegion, dir: &Path) -> Result<(), OutputError> {
  let output_text = cheat_code_text(bytes, version);
  write_text(dir, "密码金手指.txt", &output_text)
}

pub fn write_password_data_text(text: &str, dir: &Path) -> Result<(), OutputError> {
  write_text(dir, "密码数据.txt", text)
}
