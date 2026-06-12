use egui::Align2;

use crate::App;

impl App {
  pub fn show_about_window(&mut self, ctx: &egui::Context) {
    egui::Window::new("关于")
      .collapsible(false)
      .resizable(false)
      .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
      .open(&mut self.about_window_open)
      .show(ctx, |ui| {
        ui.vertical_centered(|ui| {
          ui.heading(format!("GSTool - {}", env!("CARGO_PKG_VERSION")));
          ui.label("为 GBA 的《黄金太阳》系列游戏准备的小工具！");
          ui.separator();
          ui.label("鸣谢");

          ui.horizontal(|ui| {
            ui.label("密码算法：");
            ui.hyperlink_to(
              "Golden-Sun-Password-Transfer",
              "https://github.com/Dyrati/Golden-Sun-Password-Transfer",
            );
          });

          ui.horizontal(|ui| {
            ui.label("界面字体：");
            ui.hyperlink_to("更纱黑体", "https://github.com/be5invis/Sarasa-Gothic");
          });

          ui.separator();
          ui.label("© 2025 ♥ Hambaka");
          ui.hyperlink_to("Powered by egui", "https://www.egui.rs/");
        })
      });
  }
}
