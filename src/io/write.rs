use std::{
  fmt,
  fs::{self, File},
  io::Write,
  path::{Path, PathBuf},
};

use crate::{
  core::{
    enums::{GameRegion, PasswordGrade, PasswordVersion},
    password::text::dmp_to_txt,
  },
  io::render::{cheat_code_text, password_text_beautify},
};

#[derive(Debug)]
pub enum OutputError {
  NotADirectory(PathBuf),
  CreateFileFailed(PathBuf),
  CreateDirectoryFailed(PathBuf),
  WriteFailed(PathBuf),
}

impl fmt::Display for OutputError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::NotADirectory(path) => write!(f, "路径不是目录：{}", path.display()),
      Self::CreateFileFailed(file_path) => {
        write!(f, "文件创建失败：{}", file_path.display())
      }
      Self::CreateDirectoryFailed(dir_path) => {
        write!(f, "目录创建失败：{}", dir_path.display())
      }
      Self::WriteFailed(file_path) => write!(f, "文件写入失败：{}", file_path.display()),
    }
  }
}

fn ensure_dir(dir: &Path) -> Result<(), OutputError> {
  if dir.is_dir() {
    return Ok(());
  }

  if dir.exists() {
    return Err(OutputError::NotADirectory(dir.to_path_buf()));
  }

  fs::create_dir_all(dir).map_err(|_| OutputError::CreateDirectoryFailed(dir.to_path_buf()))?;

  Ok(())
}

fn write_bytes(dir: &Path, file_name: &str, bytes: &[u8]) -> Result<(), OutputError> {
  ensure_dir(dir)?;
  let file_path = dir.join(file_name);

  let mut file =
    File::create(&file_path).map_err(|_| OutputError::CreateFileFailed(file_path.to_path_buf()))?;

  file
    .write_all(bytes)
    .map_err(|_| OutputError::WriteFailed(file_path.to_path_buf()))?;

  Ok(())
}

fn write_text(dir: &Path, file_name: &str, text: &str) -> Result<(), OutputError> {
  write_bytes(dir, file_name, text.as_bytes())
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
pub fn write_cheat_code(
  bytes: &[u8],
  grade: PasswordGrade,
  version: GameRegion,
  dir: &Path,
) -> Result<(), OutputError> {
  let output_text = cheat_code_text(bytes, grade, version);
  write_text(dir, "密码金手指.txt", &output_text)
}

pub fn write_password_data_text(text: &str, dir: &Path) -> Result<(), OutputError> {
  write_text(dir, "密码数据.txt", text)
}
