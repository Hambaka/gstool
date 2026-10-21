//! 存档槽位的纯数据枚举（跨功能共享数据）。
//!
//! 只放数据与中文命名，用于渲染
//! 与该枚举相关的`save_slot_atoms`在`ui`侧的`widgets/buttons.rs`。

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SaveSlotEnum {
  First = 0,
  Second = 1,
  Third = 2,
}

impl From<usize> for SaveSlotEnum {
  fn from(value: usize) -> Self {
    match value {
      0 => Self::First,
      1 => Self::Second,
      _ => Self::Third,
    }
  }
}

impl SaveSlotEnum {
  pub const ALL: [SaveSlotEnum; 3] = [
    SaveSlotEnum::First,
    SaveSlotEnum::Second,
    SaveSlotEnum::Third,
  ];

  /// 槽位序号：
  /// `0/1/2`对应`一/二/三`号槽位
  pub fn index(self) -> usize {
    self as usize
  }

  /// `N号存档`的显示名；`is_password_preview_window` 为真且是第一槽位时多带`/密码数据`文本。
  pub fn name(&self, is_password_preview_window: bool) -> &'static str {
    if is_password_preview_window {
      match self {
        Self::First => "号存档/密码数据",
        _ => "号存档",
      }
    } else {
      "号存档"
    }
  }
}
