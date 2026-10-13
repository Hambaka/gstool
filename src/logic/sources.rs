//! `密码导出`三个子页共享的数据：已载入的密码来源。
//!
//! `1.存档`/`2.文本`/`3. 内存转储`各写入自己的槽位，
//! `注入密码`与密码信息预览窗口只是读方——读方一律经这里的派生方法访问，

use crate::{
  core::{
    bits::BitVec,
    enums::{PasswordGrade, PasswordVersion},
    save::SaveSlot,
  },
  ui::window::pwd_info::SaveSlotEnum,
};

/// 一个已载入的密码（文本 / 内存转储来源统一形态；不含文件路径——路径在各页状态里）
#[derive(Debug, Clone)]
pub struct LoadedPassword {
  pub bytes: Vec<u8>,
  pub bits: BitVec,
  pub grade: PasswordGrade,
  /// 密码所使用的字符集版本，区分日文/英文版密码。仅`密码文本`来源能判断（内存转储无法区分）
  pub version: Option<PasswordVersion>,
  /// 去空白后的原文。仅「密码文本」来源有（c1s2「转换版本」用）
  pub text: Option<String>,
}

#[derive(Debug, Default)]
pub struct PasswordSources {
  /// 来自存档文件（下标 = 槽位号）
  pub from_save: [Option<SaveSlot>; 3],
  /// 来自密码文本（文件 / 剪贴板）
  pub from_text: Option<LoadedPassword>,
  /// 来自内存转储
  pub from_dump: Option<LoadedPassword>,
}

impl PasswordSources {
  /// 存档来源是否已载入（三个槽位任一有数据）
  pub fn has_save(&self) -> bool {
    self.from_save.iter().any(Option::is_some)
  }

  /// 第一个有数据的槽位（按下标顺序）；无存档来源时为 None
  pub fn first_available_slot(&self) -> Option<SaveSlotEnum> {
    self
      .from_save
      .iter()
      .position(Option::is_some)
      .map(SaveSlotEnum::from)
  }

  /// 密码文本来源是否已载入
  pub fn has_text(&self) -> bool {
    self.from_text.is_some()
  }

  /// 内存转储来源是否已载入
  pub fn has_dump(&self) -> bool {
    self.from_dump.is_some()
  }

  /// 密码文本来源的级别
  pub fn txt_grade(&self) -> Option<PasswordGrade> {
    self.from_text.as_ref().map(|p| p.grade)
  }

  /// 内存转储来源的级别
  pub fn dmp_grade(&self) -> Option<PasswordGrade> {
    self.from_dump.as_ref().map(|p| p.grade)
  }

  /// 重置存档来源（三个槽位全清）
  pub fn reset_save(&mut self) {
    self.from_save = [None, None, None];
  }

  /// 重置密码文本来源
  pub fn reset_text(&mut self) {
    self.from_text = None;
  }

  /// 重置内存转储来源
  pub fn reset_dump(&mut self) {
    self.from_dump = None;
  }
}
