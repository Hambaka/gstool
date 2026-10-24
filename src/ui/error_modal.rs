use egui::{Id, Modal, Sides};

/// 显示错误信息Modal窗口。
pub fn show_error_modal(ui: &mut egui::Ui, err_modal_open: &mut bool, err_msg: &str) {
  let modal = Modal::new(Id::unique("ID_MODAL_ERROR_MSG")).show(ui, |ui| {
    ui.set_width(250.0);
    ui.heading("错误！");
    ui.add_space(16.0);
    ui.label(err_msg);
    ui.separator();
    Sides::new().show(
      ui,
      |_ui| {},
      |ui| {
        if ui.button("关闭").clicked() {
          // 新版本中应使用
          // ui.close()
          *err_modal_open = false;
        }
      },
    );
  });

  if modal.should_close() {
    *err_modal_open = false;
  }
}
