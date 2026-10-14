use egui::{
  Button, ColorImage, Image, Label, RichText, TextureHandle, TextureOptions, Ui, Vec2, Widget,
  load::SizedTexture,
};
use image::ImageReader;

use std::io::Cursor;

use crate::{
  App,
  core::enums::GameTitle,
  core::save::pc_name::{EncodedName, MAX_NAME_CHARS, base_character_code, encode_pc_name},
  io::render::name_encode_log,
  widgets::colors::{DARK, INFO},
};

// 键盘按键图（由构建脚本生成，见 "gen_name" in "build.rs"）
include!(concat!(env!("OUT_DIR"), "/codegen_name_kbd.rs"));

/// 主角团 7 位成员的中文名（界面标签）
const PC_NAMES: [&str; 7] = [
  "罗宾",
  "杰拉德",
  "伊万",
  "梅雅莉",
  "加西亚",
  "加斯敏",
  "西芭",
];

const DESC_C3: &str = r"该功能仅支持《黄金太阳 开启的封印》新汉化版，主要面向自定义中文名（但也支持中日英混合）。
请在输入框中输入至少一个角色的名字，再点击“运行”即可在下方的输出区域展示如何输入自定义名。

注：
1. 在“运行”后点击“复制”按钮，输出内容将会复制到剪贴板，之后便可粘贴到记事本进行查看。
2. 若想命名初代主角团其他的三位成员，请在起名界面连按三下“Select”键，成功后可听到提示音。
3. 若想命名最后的三位角色，请接着上条继续输入“↑ ↓ ↑ ↓ ← → ← → ↑ → ↓ ← ↑”和“Select”，成功后可听到提示音。";

impl App {
  pub fn show_c3(&mut self, ui: &mut Ui) {
    Label::new(DESC_C3).selectable(false).ui(ui);

    ui.add_space(16.0);

    egui::Grid::new("ID_GRID_C3")
      .num_columns(2)
      .spacing([40.0, 4.0])
      .striped(true)
      .show(ui, |ui| {
        for (i, ele) in PC_NAMES.iter().enumerate() {
          Label::new(*ele)
            .selectable(false)
            .ui(ui)
            .on_hover_text(format!("{ele}的自定义角色名。"));
          ui.add(
            egui::TextEdit::singleline(self.new_pc_names.get_mut(i).unwrap())
              .char_limit(MAX_NAME_CHARS)
              .hint_text("请填写名字")
              .desired_width(f32::INFINITY),
          );
          ui.end_row();
        }
      });
  }

  pub fn run_c3(&mut self) {
    // 该功能仅支持《开启的封印》新汉化版，角色名所在存档为 GS1，
    // 故金手指基址由 GS1 的 `SaveLayout` 推导（原 `NAME_ADDRESS = 0x0200_0500` 即 `pc_name_ram_address(0)`）
    let layout = GameTitle::TheBrokenSeal.save_layout();

    let mut owned_entries: Vec<(&str, u32, EncodedName)> = Vec::new();
    for (index, pc_name_new) in self.new_pc_names.iter().enumerate() {
      if pc_name_new.is_empty() {
        continue;
      }

      let encoded = encode_pc_name(pc_name_new);
      let base_address = layout.pc_name_ram_address(index);
      owned_entries.push((PC_NAMES[index], base_address, encoded));
    }

    let entries: Vec<(&str, u32, &EncodedName)> = owned_entries
      .iter()
      .map(|(name, addr, enc)| (*name, *addr, enc))
      .collect();

    self.out_log_c3 = name_encode_log(&entries);
  }

  pub fn show_log_c3(&mut self, ui: &mut Ui) {
    let log_vec: Vec<&str> = self.out_log_c3.split('\n').collect();
    for single_line in log_vec {
      if single_line.contains("：")
        && (single_line.chars().count() == 4 || single_line.chars().count() == 3)
      {
        let char_vec: Vec<&str> = single_line.split('：').collect();
        ui.horizontal(|ui| {
          Label::new(format!("{}：", char_vec.first().unwrap()))
            .selectable(false)
            .ui(ui);
          for char_btn in char_vec.get(1).unwrap().chars() {
            ui.add(Button::new(RichText::new(char_btn).color(DARK)).fill(INFO))
              .on_hover_ui(|ui| {
                if let Some(char_index) = base_character_code(char_btn) {
                  let (tex, size) = get_name_keyboard_img(ui, char_index);
                  let width = size[0] * 2.0;
                  let height = size[1] * 2.0;
                  let a = SizedTexture::new(tex.id(), Vec2::new(width, height));
                  ui.add(Image::new(a).max_height(208.0));
                }
              });
          }
        });
      } else {
        Label::new(single_line).selectable(false).ui(ui);
      }
    }
  }
}

fn get_name_keyboard_img(ui: &egui::Ui, char_index: u8) -> (TextureHandle, [f32; 2]) {
  let raw_data = *KEYBOARD_IMG_MAP.get(&char_index).unwrap();
  let mut reader = ImageReader::new(Cursor::new(raw_data));
  reader.set_format(image::ImageFormat::Png);
  let image = reader.decode().unwrap();
  let size = [image.width() as _, image.height() as _];
  let image_buffer = image.to_rgba8();
  let pixels = image_buffer.as_flat_samples();

  let color_image = ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
  (
    ui.load_texture(
      format!("name_kbd_{char_index:02X}"),
      color_image,
      TextureOptions::NEAREST,
    ),
    [size[0] as f32, size[1] as f32],
  )
}
