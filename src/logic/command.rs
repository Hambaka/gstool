//! 功能命令 / 子命令的纯数据枚举（跨功能共享数据）。
//!
//! 只放数据与中文名；
//! 而`command_atoms`和`sub_command_atoms`因为返回的是`egui::Image`，
//! 所以在`ui`侧的`widgets/atoms.rs`中。

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommandEnum {
  First = 0,
  Second = 1,
  Third = 2,
  Fourth = 3,
  Fifth = 4,
}

impl CommandEnum {
  pub const ALL: [Self; 5] = [
    Self::First,
    Self::Second,
    Self::Third,
    Self::Fourth,
    Self::Fifth,
  ];

  pub fn name(self) -> &'static str {
    match self {
      // ⏏、⮉、⤴、⬆
      Self::First => "密码导出功能",
      // ⇆
      Self::Second => "存档转换功能",
      // 📝、🖊
      Self::Third => "自定义角色名",
      // 🌐、🔨、🛠
      Self::Fourth => "应用汉化补丁",
      // ⤵、⤓、⮋、⬇
      Self::Fifth => "注入密码功能",
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SubCommandEnum {
  /// ❶ 密码导出功能 -> 🅰 游戏存档
  First = 0,
  /// ❶ 密码导出功能 -> 🅱 密码文本
  Second = 1,
  /// ❶ 密码导出功能 -> 🅲 内存转储
  Third = 2,
}

impl SubCommandEnum {
  pub const ALL: [Self; 3] = [Self::First, Self::Second, Self::Third];

  pub fn name(&self) -> &'static str {
    match self {
      Self::First => "游戏存档",
      Self::Second => "密码文本",
      Self::Third => "内存转储",
    }
  }
}
