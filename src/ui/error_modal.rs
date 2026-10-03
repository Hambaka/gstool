use egui::{Id, Modal, Sides};

use crate::App;

impl App {
  pub fn show_error_modal(&mut self, ui: &mut egui::Ui) {
    let modal = Modal::new(Id::unique("ID_MODAL_ERROR_MSG")).show(ui, |ui| {
      ui.set_width(250.0);
      ui.heading("错误！");
      ui.add_space(16.0);
      ui.label(self.err_msg.clone());
      ui.separator();
      Sides::new().show(
        ui,
        |_ui| {},
        |ui| {
          if ui.button("关闭").clicked() {
            // 新版本中应使用
            // ui.close()
            self.err_modal_open = false;
          }
        },
      );
    });

    if modal.should_close() {
      self.err_modal_open = false;
    }
  }
}
