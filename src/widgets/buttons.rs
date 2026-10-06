use egui::{Button, Image, RichText};

use crate::widgets::{
  colors::{DANGER, DARK, LIGHT, PRIMARY, SECONDARY, SUCCESS, WARNING},
  icons::{BAN, CLIPBOARD, COPY, FILE, FOLDER, GLASS, RESET, RUN},
};

pub fn select_file_btn(svg_icon_size: f32) -> Button<'static> {
  Button::new((
    Image::new(FILE).tint(LIGHT).max_height(svg_icon_size),
    RichText::new("选择文件").color(LIGHT),
  ))
  .fill(PRIMARY)
}

pub fn select_dir_btn(svg_icon_size: f32) -> Button<'static> {
  Button::new((
    Image::new(FOLDER).tint(LIGHT).max_height(svg_icon_size),
    RichText::new("选择目录").color(LIGHT),
  ))
  .fill(PRIMARY)
}

pub fn reset_btn(svg_icon_size: f32) -> Button<'static> {
  Button::new((
    Image::new(RESET).tint(DARK).max_height(svg_icon_size),
    RichText::new("重置").color(DARK),
  ))
  .fill(WARNING)
}

pub fn pwd_info_btn(svg_icon_size: f32) -> Button<'static> {
  Button::new((
    Image::new(GLASS).tint(LIGHT).max_height(svg_icon_size),
    RichText::new("查看信息").color(LIGHT),
  ))
  .fill(SECONDARY)
}

pub fn clipboard_btn(svg_icon_size: f32) -> Button<'static> {
  Button::new((
    Image::new(CLIPBOARD).tint(LIGHT).max_height(svg_icon_size),
    RichText::new("剪贴板").color(LIGHT),
  ))
  .fill(PRIMARY)
}

pub fn run_btn_unavailable(svg_icon_size: f32) -> Button<'static> {
  Button::new((
    Image::new(BAN).tint(LIGHT).max_height(svg_icon_size),
    RichText::new("运行").color(LIGHT),
  ))
  .fill(DANGER)
}

pub fn run_btn_available(svg_icon_size: f32) -> Button<'static> {
  Button::new((
    Image::new(RUN).tint(LIGHT).max_height(svg_icon_size),
    RichText::new("运行").color(LIGHT),
  ))
  .fill(SUCCESS)
}

pub fn copy_btn(svg_icon_size: f32) -> Button<'static> {
  Button::new((
    Image::new(COPY).tint(LIGHT).max_height(svg_icon_size),
    RichText::new("复制").color(LIGHT),
  ))
  .fill(SECONDARY)
}
