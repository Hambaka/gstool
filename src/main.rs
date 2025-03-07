#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

fn main() -> eframe::Result {
  let native_options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default()
      .with_inner_size([800.0, 600.0])
      .with_min_inner_size([740.0, 500.0]),
    ..Default::default()
  };
  eframe::run_native(
    &gstool::App::name(),
    native_options,
    Box::new(|cc| Ok(Box::new(gstool::App::new(cc)))),
  )
}
