//! “关于”窗口。
//!
//! 窗口开关状态由`AboutState`管理。

use egui::{Align2, Image};

use crate::ui::{
  page::FrameCtx,
  widgets::{
    colors::DANGER,
    icons::{C_CIRCLE, HEART},
  },
};

/// “关于”窗口状态
#[derive(Debug, Clone)]
pub struct AboutState {
  /// 窗口是否打开
  pub window_open: bool,
}

/// 绘制“关于”窗口
pub fn show_about_window(ui: &mut egui::Ui, state: &mut AboutState, f_ctx: FrameCtx) {
  egui::Window::new("关于")
    .collapsible(false)
    .resizable(false)
    .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
    .open(&mut state.window_open)
    .show(ui, |ui| {
      ui.vertical_centered(|ui| {
        ui.heading(format!("GSTool - {}", env!("CARGO_PKG_VERSION")));
        ui.label("为 GBA 的《黄金太阳》系列游戏准备的小工具！");

        ui.separator();

        ui.heading("致谢");
        ui.label("感谢以下项目/作者所提供的直接或间接的帮助与支持！");

        ui.add_space(16.0);

        egui::Grid::new("ID_GRID_ABOUT")
          .num_columns(3)
          .spacing([40.0, 4.0])
          .striped(false)
          .show(ui, |ui| {
            ui.label("密码生成算法");
            ui.label("Dyrati");
            ui.hyperlink_to(
              "Golden-Sun-Password-Transfer",
              "https://github.com/Dyrati/Golden-Sun-Password-Transfer",
            );
            ui.end_row();

            ui.label("密码注入方法");
            ui.label("SA-1");
            ui.hyperlink_to(
              "根据密码级别写入不同数据",
              "https://www.bilibili.com/video/BV1na2rBvE71",
            );
            ui.end_row();

            ui.label("初代存档编辑");
            ui.label("90Kirsdarke");
            ui.hyperlink_to(
              "Save Game Hacking Guide",
              "https://gamefaqs.gamespot.com/gba/468548-golden-sun/faqs/43776",
            );
            ui.end_row();

            ui.label("二代存档编辑");
            ui.label("kaitia");
            ui.hyperlink_to(
              "Save Game Hacking Guide",
              "https://gamefaqs.gamespot.com/gba/561356-golden-sun-the-lost-age/faqs/30811",
            );
            ui.end_row();
          });

        ui.separator();
        ui.hyperlink_to("Powered by egui", "https://www.egui.rs/");
      });

      ui.columns(3, |columns| {
        columns[1].horizontal(|ui| {
          ui.add(
            Image::new(C_CIRCLE)
              .tint(f_ctx.text_color())
              .max_height(f_ctx.svg_icon_size),
          );
          ui.label("2026");
          ui.add(
            Image::new(HEART)
              .tint(DANGER)
              .max_height(f_ctx.svg_icon_size),
          );
          ui.label("Hambaka");
        });
      });
    });
}
