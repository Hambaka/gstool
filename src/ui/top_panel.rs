use egui::Widget;

use crate::App;

impl App {
  pub fn show_top_panel(&mut self, ui: &mut egui::Ui) {
    // 顶部面板，主要用于实现菜单栏
    egui::Panel::top("ID_TOP_PANEL").show_inside(ui, |ui| {
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
        ui.menu_button("🗀 文件", |ui| {
          if ui.button("❓ 关于").clicked() {
            self.about_window_open = !self.about_window_open;
            ui.close();
          }

          ui.separator();

          if egui::Button::new("🚪 退出")
            .shortcut_text(ui.format_shortcut(&egui::KeyboardShortcut::new(
              egui::Modifiers::COMMAND,
              egui::Key::Q,
            )))
            .ui(ui)
            .clicked()
          {
            ui.send_viewport_cmd(egui::ViewportCommand::Close);
          }
        });

        ui.add_space(16.0);
      });
    });
  }
}
