use egui::{Button, RichText};

use crate::widgets::colors::{DARK, LIGHT, PRIMARY, SECONDARY, WARNING};

pub fn select_file_btn() -> Button<'static> {
  Button::new(RichText::new("🖹 选择文件").color(LIGHT)).fill(PRIMARY)
}

pub fn select_dir_btn() -> Button<'static> {
  Button::new(RichText::new("🗁 选择目录").color(LIGHT)).fill(PRIMARY)
}

pub fn reset_btn() -> Button<'static> {
  Button::new(RichText::new("⟳ 重置").color(DARK)).fill(WARNING)
}

pub fn pwd_info_btn() -> Button<'static> {
  Button::new(RichText::new("🔍 查看信息").color(LIGHT)).fill(SECONDARY)
}

pub fn clipboard_btn() -> Button<'static> {
  Button::new(RichText::new("📋 剪贴板").color(LIGHT)).fill(PRIMARY)
}
