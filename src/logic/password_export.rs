//! `密码导出`功能的状态结构体：三个子页（游戏存档/密码文本/内存转储）。
//!
//! 而每个子页又有各自的状态结构体（各自的字段与待办request）；
//! 跨功能共享的密码源数据`PasswordSources`在`App`上，
//! 各子页状态只写自己在`PasswordSources`里的槽位。

pub mod from_dump;
pub mod from_save;
pub mod from_text;
pub mod shared;

use crate::{
  logic::command::SubCommandEnum,
  logic::sources::PasswordSources,
};

/// `密码导出`功能的三个子页状态合而为一
#[derive(Debug)]
pub struct PasswordExportState {
  /// 当前激活的子页
  pub subcommand: SubCommandEnum,
  /// ① 游戏存档子页
  pub from_save: from_save::FromSaveState,
  /// ② 密码文本子页
  pub from_text: from_text::FromTextState,
  /// ③ 内存转储子页
  pub from_dump: from_dump::FromDumpState,
}

impl PasswordExportState {
  /// 本功能是否可运行（按子页判断能否点击运行按钮）
  pub fn can_run(&self, sources: &PasswordSources) -> bool {
    match self.subcommand {
      SubCommandEnum::First => self.from_save.can_run(sources),
      SubCommandEnum::Second => self.from_text.can_run(sources),
      SubCommandEnum::Third => self.from_dump.can_run(sources),
    }
  }

  /// 是否有输出日志（以此判断是否可复制内容到剪贴板）
  pub fn has_output(&self) -> bool {
    match self.subcommand {
      SubCommandEnum::First => self.from_save.has_output(),
      SubCommandEnum::Second => self.from_text.has_output(),
      SubCommandEnum::Third => self.from_dump.has_output(),
    }
  }

  /// 日志文本（使用对应子页的日志）
  pub fn log(&self) -> &str {
    match self.subcommand {
      SubCommandEnum::First => self.from_save.log(),
      SubCommandEnum::Second => self.from_text.log(),
      SubCommandEnum::Third => self.from_dump.log(),
    }
  }

  /// 处理各子页自己的待办（读文件、读剪贴板等IO操作）。
  ///
  /// 无论当前激活哪个子页，ui侧登记的待办request都由下一帧的logic处理。
  pub fn logic(&mut self, sources: &mut PasswordSources) -> Result<(), String> {
    self.from_save.logic(sources)?;
    self.from_text.logic(sources)?;
    self.from_dump.logic(sources)
  }

  /// 一键运行（按子页运行对应的方法；由`AppRequest::Run`调用）
  pub fn run(&mut self, sources: &mut PasswordSources) -> Result<(), String> {
    match self.subcommand {
      SubCommandEnum::First => self.from_save.run(sources),
      SubCommandEnum::Second => self.from_text.run(sources),
      SubCommandEnum::Third => self.from_dump.run(sources),
    }
  }
}

impl Default for PasswordExportState {
  /// 初始化当前选择子页为`First`，三个子状态各自设置默认值
  fn default() -> Self {
    Self {
      subcommand: SubCommandEnum::First,
      from_save: from_save::FromSaveState::default(),
      from_text: from_text::FromTextState::default(),
      from_dump: from_dump::FromDumpState::default(),
    }
  }
}
