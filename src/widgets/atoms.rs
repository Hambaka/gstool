//! 功能命令 / 子命令可选项的渲染：(图标 + 中文名称) 原子。
//!
//! 数据枚举在 `logic/command.rs`；
//! 与 `buttons.rs` 的 `save_slot_atoms` 同一规则。

use egui::Image;

use crate::{
  logic::command::{CommandEnum, SubCommandEnum},
  widgets::{
    colors::get_default_svg_color,
    icons::{
      ARROW_DOWN, ARROW_LEFT_RIGHT, ARROW_UP, NUMBER_1_SQUARE_FILL, NUMBER_2_SQUARE_FILL,
      NUMBER_3_SQUARE_FILL, PENCIL, TOOLS,
    },
  },
};

/// 功能（命令）可选项的渲染
pub fn command_atoms(
  command: CommandEnum,
  svg_icon_size: f32,
  is_dark: bool,
) -> (Image<'static>, &'static str) {
  let color = get_default_svg_color(is_dark);

  (
    Image::new(match command {
      CommandEnum::First => ARROW_UP,
      CommandEnum::Second => ARROW_LEFT_RIGHT,
      CommandEnum::Third => PENCIL,
      CommandEnum::Fourth => TOOLS,
      CommandEnum::Fifth => ARROW_DOWN,
    })
    .tint(color)
    .max_height(svg_icon_size),
    command.name(),
  )
}

/// 子命令（子页）可选项的渲染
pub fn sub_command_atoms(
  sub_command: SubCommandEnum,
  svg_icon_size: f32,
  is_dark: bool,
) -> (Image<'static>, &'static str) {
  let color = get_default_svg_color(is_dark);

  (
    Image::new(match sub_command {
      SubCommandEnum::First => NUMBER_1_SQUARE_FILL,
      SubCommandEnum::Second => NUMBER_2_SQUARE_FILL,
      SubCommandEnum::Third => NUMBER_3_SQUARE_FILL,
    })
    .tint(color)
    .max_height(svg_icon_size),
    sub_command.name(),
  )
}
