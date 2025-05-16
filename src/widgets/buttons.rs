use egui::{Button, RichText};

use crate::widgets::colors::{DARK, LIGHT, PRIMARY, WARNING};

pub fn select_file_btn() -> Button<'static> {
  Button::new(RichText::new("🖹 选择文件").color(LIGHT)).fill(PRIMARY)
}

pub fn select_dir_btn() -> Button<'static> {
  Button::new(RichText::new("🗁 选择目录").color(LIGHT)).fill(PRIMARY)
}

pub fn reset_btn() -> Button<'static> {
  Button::new(RichText::new("⟳ 重置").color(DARK)).fill(WARNING)
}
