//! 顶部面板（菜单栏）。

use egui::{Image, ThemePreference, Widget};

use crate::{
  ui::page::FrameCtx,
  widgets::icons::{DISPLAY, DOOR, INFO, LIST, MOON, SUN},
};

/// 绘制顶部面板（菜单栏）
pub fn show_top_panel(ui: &mut egui::Ui, about_open: &mut bool, f_ctx: FrameCtx) {
  // 顶部面板，主要用于实现菜单栏
  egui::Panel::top("ID_TOP_PANEL").show(ui, |ui| {
    // 菜单栏
    egui::MenuBar::new().ui(ui, |ui| {
      let (svg_icon_color, svg_icon_size) = (f_ctx.svg_color(), f_ctx.svg_icon_size);

      ui.menu_button(
        (
          Image::new(LIST)
            .tint(svg_icon_color)
            .max_height(svg_icon_size),
          "文件",
        ),
        |ui| {
          if ui
            .button((
              Image::new(INFO)
                .tint(svg_icon_color)
                .max_height(svg_icon_size),
              "关于",
            ))
            .clicked()
          {
            *about_open = !*about_open;
            ui.close();
          }

          ui.separator();

          if egui::Button::new((
            Image::new(DOOR)
              .tint(svg_icon_color)
              .max_height(svg_icon_size),
            "退出",
          ))
          .shortcut_text(ui.format_shortcut(&egui::KeyboardShortcut::new(
            egui::Modifiers::COMMAND,
            egui::Key::Q,
          )))
          .ui(ui)
          .clicked()
          {
            ui.send_viewport_cmd(egui::ViewportCommand::Close);
          }
        },
      );

      ui.separator();

      // egui::widgets::global_theme_preference_buttons(ui);
      let mut theme_preference = ui.options(|opt| opt.theme_preference);
      ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;

        ui.selectable_value(
          &mut theme_preference,
          ThemePreference::System,
          Image::new(DISPLAY)
            .tint(svg_icon_color)
            .max_height(svg_icon_size),
        )
        .on_hover_text("跟随系统");

        ui.selectable_value(
          &mut theme_preference,
          ThemePreference::Dark,
          Image::new(MOON)
            .tint(svg_icon_color)
            .max_height(svg_icon_size),
        )
        .on_hover_text("深色主题");

        ui.selectable_value(
          &mut theme_preference,
          ThemePreference::Light,
          Image::new(SUN)
            .tint(svg_icon_color)
            .max_height(svg_icon_size),
        )
        .on_hover_text("浅色主题");
      });

      ui.ctx().set_theme(theme_preference);
    });
  });
}
