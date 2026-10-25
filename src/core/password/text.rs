//! 与密码字符和字节相关的数据与方法，并包含了由构建脚本在编译期生成的数据表（见 `build/gen_pwd.rs`）。
//!
//! 生成的内容：
//! - `VALID_PASSWORD_CHARS_JP_EN_MAP`：用于日文密码字符转换为英文密码字符
//! - `VALID_PASSWORD_CHARS_EN_JP_MAP`：用于英文密码字符转换为日文密码字符
//! - `VALID_PASSWORD_CHARS_JP_BYTE_MAP`：用于日文密码字符转换为字节值
//! - `VALID_PASSWORD_CHARS_EN_BYTE_MAP`：用于英文密码字符转换为字节值
use std::fmt;

use crate::{
  core::{
    bits::BitVec,
    enums::{PasswordGrade, PasswordVersion},
    password::bits_from_bytes,
  },
  util::whitespace,
};

include!(concat!(env!("OUT_DIR"), "/codegen_pwd.rs"));

/// Japanese Hiragana characters in Japanese version password.
///
/// Please note that 'を' and 'ん' are arranged in reverse order, because 'を' is 0x2D and 'ん' is 0x2C.
#[rustfmt::skip]
const VALID_PASSWORD_CHARS_JP: [char; 64] = [
  'あ', 'い', 'う', 'え', 'お',
  'か', 'き', 'く', 'け', 'こ',
  'さ', 'し', 'す', 'せ', 'そ',
  'た', 'ち', 'つ', 'て', 'と',
  'な', 'に', 'ぬ', 'ね', 'の',
  'は', 'ひ', 'ふ', 'へ', 'ほ',
  'ま', 'み', 'む', 'め', 'も',
  'や', 'ゆ', 'よ',
  'ら', 'り', 'る', 'れ', 'ろ',
  'わ', 'ん', 'を',
  'が', 'ぎ', 'ぐ', 'げ', 'ご',
  'ざ', 'じ', 'ず', 'ぜ', 'ぞ',
  'だ', 'で', 'ど',
  'ば', 'び', 'ぶ', 'べ', 'ぼ'
];

/// English letters, numbers, signs in English version password.
#[rustfmt::skip]
const VALID_PASSWORD_CHARS_EN: [char; 64] = [
  'A', 'B', 'C', 'D', 'E',
  'F', 'G', 'H', 'J', 'K',
  'L', 'M', 'N', 'P', 'Q',
  'R',
  'S', 'T', 'U', 'V', 'W',
  'X', 'Y', 'Z', '2', '3',
  '4', '5', '6', '7', '8',
  '9',
  'a', 'b', 'c', 'd', 'e',
  'f', 'g', 'h', 'i', 'j',
  'k', 'm', 'n', 'p', 'q',
  'r',
  's', 't', 'u', 'v', 'w',
  'x', 'y', 'z', '!', '?',
  '#', '&', '$', '%', '+',
  '='
];

#[derive(Debug)]
pub enum PasswordTextError {
  Empty,
  BadLength(usize),
  InvalidChar(PasswordVersion),
  InvalidPassword,
}

impl fmt::Display for PasswordTextError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Empty => write!(f, "文本中没有任何内容！"),
      Self::BadLength(len) => write!(f, "密码的长度应为16/61/260字符！（去空白后实际{len}字符）"),
      Self::InvalidChar(password_version) => write!(
        f,
        "密码文本包含无效字符！（{}版密码字符）",
        password_version.name()
      ),
      Self::InvalidPassword => write!(f, "密码无效！"),
    }
  }
}

#[derive(Debug, Clone)]
pub struct ParsedPassword {
  /// 去掉全部空白字符后的密码文本
  pub text: String,
  /// 密码文本的内存转储
  pub bytes: Vec<u8>,
  /// 已通过有效性校验的解码后的位数组
  pub bits: BitVec,
  pub grade: PasswordGrade,
  pub version: PasswordVersion,
}

/// 把用户输入的密码文本源解析成可用的密码数据
pub fn parse_password_text(input: &str) -> Result<ParsedPassword, PasswordTextError> {
  let text = whitespace::remove_all(input);
  if text.is_empty() {
    return Err(PasswordTextError::Empty);
  }

  let char_count = text.chars().count();
  let grade =
    PasswordGrade::from_size(char_count).ok_or(PasswordTextError::BadLength(char_count))?;

  // 能运行到这里时文本肯定不为空，所以一定能获得密码版本的值
  let version = detect_password_version(&text).ok_or(PasswordTextError::Empty)?;

  let has_invalid_char = match version {
    PasswordVersion::Japanese => contains_invalid_char_jp(&text),
    PasswordVersion::English => contains_invalid_char_en(&text),
  };
  if has_invalid_char {
    return Err(PasswordTextError::InvalidChar(version));
  }

  let bytes = txt_to_dmp(&text, version);
  let bits = bits_from_bytes(&bytes, grade, true).ok_or(PasswordTextError::InvalidPassword)?;

  Ok(ParsedPassword {
    text,
    bytes,
    bits,
    grade,
    version,
  })
}

/// A very stupid and lazy way to detect password version by comparing the first char in `password_str` to 'z' in ASCII.
///
/// 'z' is the "largest" char in English version password. If the first char is larger than 'z', then it is Japanese version password, since Japanese version password characters are all Hiragana.
pub fn detect_password_version(password_str: &str) -> Option<PasswordVersion> {
  password_str.chars().next().map(|ch| {
    if ch <= 'z' {
      PasswordVersion::English
    } else {
      PasswordVersion::Japanese
    }
  })
}

/// Returns `true` if this `password_str` contains char that does not exist in `VALID_PASSWORD_CHARS_EN`.
pub fn contains_invalid_char_en(password_str: &str) -> bool {
  password_str
    .chars()
    .any(|ch| !VALID_PASSWORD_CHARS_EN.contains(&ch))
}

/// Returns `true` if this `password_str` contains char that does not exist in `VALID_PASSWORD_CHARS_JP`.
pub fn contains_invalid_char_jp(password_str: &str) -> bool {
  password_str
    .chars()
    .any(|ch| !VALID_PASSWORD_CHARS_JP.contains(&ch))
}

/// Returns `true` if this `password_bytes` contains value that does not exist in `VALID_PASSWORD_BYTES`.
pub fn contains_invalid_byte(password_bytes: &[u8]) -> bool {
  password_bytes.iter().any(|x| *x > 63)
}

/// Convert password text string to bytes (memory dump).
pub fn txt_to_dmp(password: &str, password_version: PasswordVersion) -> Vec<u8> {
  match password_version {
    PasswordVersion::Japanese => password
      .chars()
      .map(|x| *(VALID_PASSWORD_CHARS_JP_BYTE_MAP.get(&x).unwrap()))
      .collect(),
    PasswordVersion::English => password
      .chars()
      .map(|x| *(VALID_PASSWORD_CHARS_EN_BYTE_MAP.get(&x).unwrap()))
      .collect(),
  }
}

/// Convert password text string to another version.
pub fn txt_to_another_version(password: &str, password_version: PasswordVersion) -> String {
  match password_version {
    PasswordVersion::English => password
      .chars()
      .map(|x| VALID_PASSWORD_CHARS_EN_JP_MAP.get(&x).unwrap())
      .collect(),
    PasswordVersion::Japanese => password
      .chars()
      .map(|x| VALID_PASSWORD_CHARS_JP_EN_MAP.get(&x).unwrap())
      .collect(),
  }
}

/// Convert password bytes (memory dump) to text string.
pub fn dmp_to_txt(password_bytes: &[u8], password_version: PasswordVersion) -> String {
  match password_version {
    PasswordVersion::Japanese => password_bytes
      .iter()
      .map(|x| VALID_PASSWORD_CHARS_JP[usize::from(*x)])
      .collect(),
    PasswordVersion::English => password_bytes
      .iter()
      .map(|x| VALID_PASSWORD_CHARS_EN[usize::from(*x)])
      .collect(),
  }
}
