//! `密码导出`功能的视图：说明+子页标签+子页视图+日志区。
//!
//! 各子页视图只画widget、登记request（`Vec<AppRequest>`），不做任何实际的IO操作。

mod from_dump;
mod from_save;
mod from_text;

use egui::{Label, Ui, Widget};

use crate::{
  logic::{
    command::SubCommandEnum, password_export::PasswordExportState, sources::PasswordSources,
  },
  ui::{
    page::{AppRequest, FrameCtx},
    widgets::atoms::sub_command_atoms,
  },
};

const DESC_C1: &str = r"使用该功能可导出密码，可导出以下几类文件：
1. 密码的文本文件（支持导出日文版/英文版密码）
2. 密码的内存转储文件
3. 密码的金手指文本文件，可以在《黄金太阳 失落的时代》中使用（支持日/美/德/西/法/意版本）
4. 密码数据的文本文件，可结合 Dyrati 的“Golden Sun Password Generator”表格使用

注：Mobile&elffinal/SOMA 的《黄金太阳 失落的时代》汉化版基于美版制作，因此金手指版本选择美版即可。";

/// 画`密码导出`功能页（说明+子页标签+子页视图）；
/// 返回需要`App`处理的跨功能请求（各子页自己的待办request登记进各自的`state`）
pub fn show(
  ui: &mut Ui,
  state: &mut PasswordExportState,
  sources: &mut PasswordSources,
  f_ctx: FrameCtx,
) -> Vec<AppRequest> {
  Label::new(DESC_C1).selectable(false).ui(ui);
  ui.separator();
  ui.columns_const(|columns: &mut [egui::Ui; SubCommandEnum::ALL.len()]| {
    for (col, cmd) in columns.iter_mut().zip(SubCommandEnum::ALL) {
      col.selectable_value(&mut state.subcommand, cmd, sub_command_atoms(cmd, f_ctx));
    }
  });

  match state.subcommand {
    SubCommandEnum::First => from_save::show(ui, &mut state.from_save, sources, f_ctx),
    SubCommandEnum::Second => from_text::show(ui, &mut state.from_text, sources, f_ctx),
    SubCommandEnum::Third => from_dump::show(ui, &mut state.from_dump, sources, f_ctx),
  }
}

/// 显示输出日志（根据当前选择的子页进行显示）
pub fn show_log(ui: &mut Ui, state: &PasswordExportState) {
  match state.subcommand {
    SubCommandEnum::First => Label::new(state.from_save.log()).selectable(false).ui(ui),
    SubCommandEnum::Second => Label::new(state.from_text.log()).selectable(false).ui(ui),
    SubCommandEnum::Third => Label::new(state.from_dump.log()).selectable(false).ui(ui),
  };
}
