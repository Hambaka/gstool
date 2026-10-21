use egui::{Button, Image, RichText};

use crate::{
  logic::save_slot::SaveSlotEnum,
  ui::page::FrameCtx,
  widgets::{
    colors::{DANGER, DARK, LIGHT, PRIMARY, SECONDARY, SUCCESS, WARNING},
    icons::{
      BAN, CLIPBOARD, COPY, FILE, FOLDER, GLASS, NUMBER_1_CIRCLE_FILL, NUMBER_2_CIRCLE_FILL,
      NUMBER_3_CIRCLE_FILL, RESET, RUN,
    },
  },
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

/// 存档槽位可选项的渲染（编号图标 + 存档）。
/// 枚举在`logic/save_slot.rs`
pub fn save_slot_atoms(
  slot: SaveSlotEnum,
  f_ctx: FrameCtx,
  in_preview: bool,
) -> (Image<'static>, &'static str) {
  (
    Image::new(match slot {
      SaveSlotEnum::First => NUMBER_1_CIRCLE_FILL,
      SaveSlotEnum::Second => NUMBER_2_CIRCLE_FILL,
      SaveSlotEnum::Third => NUMBER_3_CIRCLE_FILL,
    })
    .tint(f_ctx.svg_color())
    .max_height(f_ctx.svg_icon_size),
    slot.name(in_preview),
  )
}
