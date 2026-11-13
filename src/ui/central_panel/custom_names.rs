use std::io::Cursor;

use egui::{
  Button, Checkbox, ColorImage, Image, Label, RichText, TextureHandle, TextureOptions, Ui, Vec2,
  Widget, load::SizedTexture,
};
use image::{ImageFormat, ImageReader};

use crate::{
  app::KeyboardCache,
  core::save::pc_name::{MAX_NAME_CHARS, base_character_code},
  logic::custom_name::{CustomNameRequest, CustomNameState, PC_NAMES},
  ui::{
    page::{AppRequest, FrameCtx},
    widgets::{
      buttons::{default_name_btn, reset_name_btn, save_as_btn, store_name_btn},
      colors::{DARK, INFO},
      rows::{PathAction, dir_row, file_row},
    },
  },
};

const DESC_C3: &str = r"使用该功能可辅助修改《黄金太阳 开启的封印》主角团的名字：
左侧：输入自定义的角色名，点击“运行”可查看在起名界面输入这些名字的方式
右侧：可读取存档并将左侧运行后的自定义名字写入其中

注：
1. 非日版存档载入时会自动转换为新汉化版默认名 + 日版构建日期（只影响输出文件）
2. 若想命名初代主角团其他的三位成员，请在起名界面连按三下“Select”键，成功后可听到提示音
3. 若想命名最后的三位角色，请接着上条继续输入“↑ ↓ ↑ ↓ ← → ← → ↑ → ↓ ← ↑”和“Select”，成功后可听到提示音";

/// 画“自定义角色名”功能页（说明 + 左栏名字输入 + 右栏存档修改）；
///
/// 因为这一页只有自己状态的请求而没有跨功能的请求，
/// 所以返回值是空的Vec。
pub fn show(ui: &mut Ui, state: &mut CustomNameState, f_ctx: FrameCtx) -> Vec<AppRequest> {
  Label::new(DESC_C3).selectable(false).ui(ui);

  ui.add_space(16.0);

  ui.columns_const(|[left, right]| {
    show_name_inputs(left, state, f_ctx);
    show_save_panel(right, state, f_ctx);
  });

  // 因为无跨功能请求，所以是空的Vec
  Vec::new()
}

/// 左半边：7 个角色名输入框
fn show_name_inputs(ui: &mut Ui, state: &mut CustomNameState, f_ctx: FrameCtx) {
  ui.group(|ui| {
    egui::Grid::new("ID_GRID_C3_LT")
      .num_columns(2)
      .min_row_height(25.0)
      .spacing([16.0, 4.0])
      .show(ui, |ui| {
        for (i, ele) in PC_NAMES.iter().enumerate() {
          Label::new(*ele)
            .selectable(false)
            .ui(ui)
            .on_hover_text(format!("{ele}的自定义角色名。"));
          let name_ref = &mut state.names[i];
          ui.add(
            egui::TextEdit::singleline(name_ref)
              .char_limit(MAX_NAME_CHARS)
              .hint_text("请填写名字（留空则不修改）")
              .desired_width(f32::INFINITY),
          );
          ui.end_row();
        }
      });

    egui::Grid::new("ID_GRID_C3_LB")
      .num_columns(1)
      .min_row_height(25.0)
      .spacing([16.0, 4.0])
      .show(ui, |ui| {
        ui.vertical_centered(|ui| {
          if ui.add(default_name_btn(f_ctx.svg_icon_size)).clicked() {
            state.set_default_name();
          };
        });
        ui.end_row();

        ui.vertical_centered(|ui| {
          if ui.add(reset_name_btn(f_ctx.svg_icon_size)).clicked() {
            state.reset_names();
          };
        });
        ui.end_row();
      });
  });
}

/// 右半边：写入存档
fn show_save_panel(ui: &mut Ui, state: &mut CustomNameState, f_ctx: FrameCtx) {
  ui.group(|ui| {

    egui::Grid::new("ID_GRID_C3_R")
      .num_columns(1)
      .min_row_height(25.0)
      .spacing([16.0, 4.0])
      .show(ui, |ui| {
        // 第1、2行 -> 输入文件路径
        match file_row(
          ui,
          "输入文件：",
          "《黄金太阳 开启的封印》的存档文件",
          "存档文件路径",
          &mut state.input_file_path,
          f_ctx.svg_icon_size,
          false,
        ) {
          PathAction::Picked(file) => state.request(CustomNameRequest::LoadSave(file)),
          PathAction::Reset => state.reset_save(),
          PathAction::None => {}
        }

        Label::new("要修改的存档槽位：")
          .selectable(false)
          .ui(ui)
          .on_hover_text("只修改勾选的槽位。");
        ui.end_row();

        ui.horizontal(|ui| {
          for i in 0..3 {
            let is_clear_data = state.save_slots.slots[i]
              .as_ref()
              .map(|slot| slot.is_clear_data);
            let (enabled, label) = match is_clear_data {
              Some(true) => (true, format!("{}号（已通关）", i + 1)),
              Some(false) => (true, format!("{}号　　　　　", i + 1)),
              None => (false, format!("{}号（无数据）", i + 1)),
            };
            ui.add_enabled(enabled, Checkbox::new(&mut state.selected_slots[i], label));
          }
        });
        ui.end_row();

        match dir_row(
          ui,
          "输出目录：",
          "自定义输出的存档文件路径（推荐留空），若留空则将转换后的存档文件保存到输入文件的同目录下。",
          "存档文件输出目录",
          &mut state.out_dir,
          f_ctx.svg_icon_size,
          false,
        ) {
          PathAction::Picked(dir) => state.out_dir = dir.to_string_lossy().into_owned(),
          PathAction::Reset => state.out_dir.clear(),
          PathAction::None => {}
        }

        Label::new(RichText::new(save_hint(state)).weak())
          .selectable(false)
          .ui(ui);
        ui.end_row();
      });

    egui::Grid::new("ID_GRID_C3_RB")
      .num_columns(1)
      .min_row_height(25.0)
      .spacing([16.0, 4.0])
      .show(ui, |ui| {
        ui.vertical_centered(|ui| {
          let can_modify = state.can_store_names();
          if ui
            .add_enabled(can_modify, store_name_btn(f_ctx.svg_icon_size))
            .clicked()
          {
            state.request(CustomNameRequest::StoreNames);
          }
        });
        ui.end_row();

        ui.vertical_centered(|ui| {
          if ui
            .add_enabled(state.has_stored_name, save_as_btn(f_ctx.svg_icon_size))
            .clicked()
          {
            state.request(CustomNameRequest::WriteSave);
          }
        });
        ui.end_row();
      });

  });
}

/// 右栏的禁用原因提示；全部满足时返回 `None`
fn save_hint(state: &CustomNameState) -> &'static str {
  if !state.has_run() {
    "请先在左侧填写名字并点击“运行”。"
  } else if state.names_changed_since_run() {
    // 如果名字修改了，上次的编码结果已经过期，所以先提示重新运行一遍
    "名字已修改，请重新运行。"
  } else if !state.has_valid_run() {
    "存在无法使用的字符，请查看下方输出。"
  } else if !state.has_save() {
    "请选择存档文件。"
  } else if !state.any_slot_selected() {
    "请勾选要修改的存档槽位。"
  } else if !state.has_stored_name {
    "先暂存名字，暂存后即可输出存档。"
  } else {
    "可随时输出存档。"
  }
}

/// 显示输出日志
///
/// 因为要把字符变成会显示对应键位图的按钮，所以需要解析原本存储的日志文本，
/// 格式与`io::render::name_encode_log`的格式关联，因此改动时必须同步检查。
pub fn show_log(ui: &mut Ui, state: &CustomNameState, cache: &mut KeyboardCache) {
  let log_text = state.log().to_string();
  for single_line in log_text.split('\n') {
    if single_line.contains("：")
      && (single_line.chars().count() == 4 || single_line.chars().count() == 3)
    {
      let parts = single_line.split('：').collect::<Vec<_>>();
      let label = parts.first().unwrap().to_string();
      let chars = parts
        .get(1)
        .map(|s| {
          s.chars()
            .filter_map(|c| base_character_code(c).map(|idx| (c, idx)))
            .collect::<Vec<_>>()
        })
        .unwrap_or_default();

      let mut key_handles = Vec::new();
      for &(c, idx) in &chars {
        key_handles.push((c, get_name_keyboard_img(ui, idx, cache)));
      }

      ui.horizontal(|ui| {
        Label::new(format!("{label}：")).selectable(false).ui(ui);
        for (char_btn, (tex, size)) in &key_handles {
          ui.add(Button::new(RichText::new(*char_btn).color(DARK)).fill(INFO))
            .on_hover_ui(|ui| {
              let width = size[0] * 2.0;
              let height = size[1] * 2.0;
              let s_tex = SizedTexture::new(tex.id(), Vec2::new(width, height));
              ui.add(Image::new(s_tex).max_height(208.0));
            });
        }
      });
    } else {
      Label::new(single_line).selectable(false).ui(ui);
    }
  }
}

// 游戏中起名界面的键盘按键图（由构建脚本生成，见"build.rs"中的"gen_name"）
mod codegen_name_kbd {
  #![allow(clippy::all, clippy::pedantic, clippy::nursery)]
  include!(concat!(env!("OUT_DIR"), "/codegen_name_kbd.rs"));
}

/// 从缓存取键盘图，若没有则创建并缓存
fn get_name_keyboard_img(
  ui: &egui::Ui,
  char_index: u8,
  cache: &mut KeyboardCache,
) -> (TextureHandle, [f32; 2]) {
  if let Some((handle, size)) = cache.0.get(&char_index) {
    return (handle.clone(), *size);
  }

  let raw_data = *codegen_name_kbd::KEYBOARD_IMG_MAP.get(&char_index).unwrap();
  let mut reader = ImageReader::new(Cursor::new(raw_data));
  reader.set_format(ImageFormat::Png);
  let image = reader.decode().unwrap();
  let size = [image.width() as _, image.height() as _];
  let image_buffer = image.to_rgba8();
  let pixels = image_buffer.as_flat_samples();

  let color_image = ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
  let handle = ui.load_texture(
    format!("name_kbd_{char_index:02X}"),
    color_image,
    TextureOptions::NEAREST,
  );

  cache.0.insert(
    char_index,
    (handle.clone(), [size[0] as f32, size[1] as f32]),
  );

  (handle, [size[0] as f32, size[1] as f32])
}
