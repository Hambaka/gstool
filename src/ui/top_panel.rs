use egui::{Image, ThemePreference, Widget};

use crate::{
  App,
  widgets::{
    colors::get_default_svg_color,
    icons::{DISPLAY, DOOR, INFO, LIST, MOON, SUN},
  },
};

impl App {
  pub fn show_top_panel(&mut self, ui: &mut egui::Ui, svg_icon_size: f32, is_dark: bool) {
    let svg_icon_color = get_default_svg_color(is_dark);

    // 顶部面板，主要用于实现菜单栏
    egui::Panel::top("ID_TOP_PANEL").show(ui, |ui| {
      // 定义快捷键
      if ui.input_mut(|input| {
        input.consume_shortcut(&egui::KeyboardShortcut::new(
          egui::Modifiers::COMMAND,
          egui::Key::Q,
        ))
      }) {
        ui.send_viewport_cmd(egui::ViewportCommand::Close);
      }

      // 菜单栏
      egui::MenuBar::new().ui(ui, |ui| {
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
              self.about_window_open = !self.about_window_open;
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
}
