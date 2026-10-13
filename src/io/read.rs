use std::{
  fmt,
  fs::File,
  io::Read,
  path::{Path, PathBuf},
};

use chardetng::{EncodingDetector, Iso2022JpDetection::Deny, Utf8Detection::Allow};

use crate::core::rom::{
  format::MAX_ROM_SIZE,
  inject::{TlaRomAnalysis, analyze},
  patch::{PatchSource, classify},
};

#[derive(Debug)]
pub enum InputError {
  Open(PathBuf),
  Read(PathBuf),
  Empty(PathBuf),
  TooLarge { file_path: PathBuf, max: u64 },
  Decode(PathBuf),
}

impl fmt::Display for InputError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Open(file_path) => write!(f, "打开文件时发生错误：{}", file_path.display()),
      Self::Read(file_path) => write!(f, "读取文件时发生错误：{}", file_path.display()),
      Self::Empty(file_path) => write!(f, "文件内容为空：{}", file_path.display()),
      Self::TooLarge { file_path, max } => {
        write!(f, "文件过大（上限 {max} 字节）：{}", file_path.display())
      }
      Self::Decode(file_path) => {
        write!(f, "输入的不是有效的文本文件：{}", file_path.display())
      }
    }
  }
}

pub fn read_bytes(path: &Path, max_size: Option<u64>) -> Result<Vec<u8>, InputError> {
  let mut file = File::open(path).map_err(|_| InputError::Open(path.to_path_buf()))?;

  let size = file
    .metadata()
    .map_err(|_| InputError::Read(path.to_path_buf()))?
    .len();
  if size == 0 {
    return Err(InputError::Empty(path.to_path_buf()));
  }

  if let Some(max) = max_size {
    if size > max {
      return Err(InputError::TooLarge {
        file_path: path.to_path_buf(),
        max,
      });
    }
  }

  let mut bytes = Vec::with_capacity(size as usize);
  file
    .read_to_end(&mut bytes)
    .map_err(|_| InputError::Read(path.to_path_buf()))?;

  Ok(bytes)
}

pub fn read_text(path: &Path, max_size: u64) -> Result<String, InputError> {
  let bytes = read_bytes(path, Some(max_size))?;

  let mut detector = EncodingDetector::new(Deny);
  detector.feed(&bytes, true);
  let encoding = detector.guess(None, Allow);

  let (text, has_error) = encoding.decode_with_bom_removal(&bytes);
  if has_error {
    return Err(InputError::Decode(path.to_path_buf()));
  }

  Ok(text.to_string())
}

/// 读取《开启的封印》ROM（上限 32MB）并分类
pub fn classify_rom_file(path: &Path) -> Result<PatchSource, String> {
  let bytes = read_bytes(path, Some(MAX_ROM_SIZE)).map_err(|e| e.to_string())?;
  Ok(classify(&bytes))
}

/// 读取《失落的时代》ROM（上限 32MB）并分析（连同字节一起返回，供注入时直接使用）
pub fn analyze_rom_file(path: &Path) -> Result<(Vec<u8>, TlaRomAnalysis), String> {
  let bytes = read_bytes(path, Some(MAX_ROM_SIZE)).map_err(|e| e.to_string())?;
  let analysis = analyze(&bytes).map_err(|e| e.to_string())?;
  Ok((bytes, analysis))
}
