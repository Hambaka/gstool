use egui::{Label, Ui, Widget};

use crate::{
  logic::patch_rom::{PatchRomRequest, PatchRomState},
  ui::{
    page::{AppRequest, FrameCtx},
    widgets::rows::{PathAction, dir_row, file_row},
  },
};

const DESC_C4: &str = r"使用该功能可为日版的《黄金太阳 开启的封印》应用最新版的汉化补丁。
除了使用日版，该功能也对其他的部分版本做了支持：
1. 其他的所有官方版本ROM（美/德/西/法/意）
2. 部分网络公开的且流传度较高的其他较老的中文汉化版ROM（CGP汉化组/CGP+elffinal/润色美化版）

注：
1. 其他汉化版的来源多种多样，无法做到全部支持。
2. 不支持使用各类基于汉化版制作的改版ROM。";

/// 画“应用汉化补丁”功能页（说明+视图）；
///
/// 因为这一页只有自己状态的请求而没有跨功能的请求，
/// 所以返回值是空的Vec。
pub fn show(ui: &mut Ui, state: &mut PatchRomState, f_ctx: FrameCtx) -> Vec<AppRequest> {
  Label::new(DESC_C4).selectable(false).ui(ui);
  ui.add_space(16.0);

  egui::Grid::new("ID_GRID_C4")
    .num_columns(2)
    .spacing([40.0, 4.0])
    .striped(true)
    .show(ui, |ui| {
      // 第1行 -> 输入文件路径
      match file_row(
        ui,
        "输入文件路径",
        "《黄金太阳 开启的封印》的ROM文件",
        "ROM文件路径",
        &mut state.display_path,
        f_ctx.svg_icon_size,
      ) {
        PathAction::Picked(file) => state.request(PatchRomRequest::Load(file)),
        PathAction::Reset => state.reset(),
        PathAction::None => {},
      }

      // 第2行 -> 指定输出文件保存目录
      match dir_row(
        ui,
        "指定输出文件保存目录",
        "自定义输出的汉化版ROM文件路径（推荐留空），若留空则将转换后的ROM文件保存到输入文件的同目录下。",
        "最新版汉化ROM输出路径",
        &mut state.out_dir,
        f_ctx.svg_icon_size,
      ) {
        PathAction::Picked(dir) => state.out_dir = dir.to_string_lossy().into_owned(),
        PathAction::Reset => state.out_dir.clear(),
        PathAction::None => {},
      }
    });

  // 因为无跨功能请求，所以是空的Vec
  Vec::new()
}

/// 显示输出日志（概要）
pub fn show_log(ui: &mut Ui, state: &PatchRomState) {
  Label::new(state.log()).selectable(false).ui(ui);
}
