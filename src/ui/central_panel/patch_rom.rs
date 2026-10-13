use std::{fmt::Write as _, path::Path};

use egui::{Label, Ui, Widget};
use rfd::FileDialog;

use crate::{
  App,
  core::rom::{
    format::MAX_ROM_SIZE,
    patch::{PatchSource, apply_chinese_patch, rom_display_name},
  },
  io::{
    classify_rom_file, patched_rom_file_name, read_bytes, resolve_output_dir, write_named_bytes,
  },
  widgets::buttons::{reset_btn, select_dir_btn, select_file_btn},
};

const DESC_C4: &str = r"使用该功能可为日版的《黄金太阳 开启的封印》应用最新版的汉化补丁。
除了使用日版，该功能也对其他的部分版本做了支持：
1. 其他的所有官方版本ROM（美/德/西/法/意）
2. 部分网络公开的且流传度较高的其他较老的中文汉化版ROM（CGP汉化组/CGP+elffinal/润色美化版）

注：
1. 其他汉化版的来源多种多样，无法做到全部支持。
2. 不支持使用各类基于汉化版制作的改版ROM。";

impl App {
  pub fn show_c4(&mut self, ui: &mut Ui, svg_icon_size: f32) {
    // 正式界面
    Label::new(DESC_C4).selectable(false).ui(ui);

    ui.add_space(16.0);

    egui::Grid::new("ID_GRID_C4")
      .num_columns(2)
      .spacing([40.0, 4.0])
      .striped(true)
      .show(ui, |ui| {
        // 第1行 -> 输入文件路径  【选择文件】【重置】
        Label::new("输入文件路径")
          .selectable(false)
          .ui(ui)
          .on_hover_text("《黄金太阳 开启的封印》的ROM文件");
        ui.horizontal(|ui| {
          if ui.add(select_file_btn(svg_icon_size)).clicked() {
            if let Some(file) = FileDialog::new().pick_file() {
              let new_file_path = file.to_string_lossy().into_owned();
              // 验证ROM文件
              let result = classify_rom_file(Path::new(&new_file_path));
              match result {
                Ok(PatchSource::Patchable { crc32 }) => {
                  self.input_file_crc32 = Some(crc32);
                  self.input_file_path_c4 = new_file_path;
                }
                Ok(PatchSource::AlreadyLatestChinese) => {
                  self.fail("输入的是已经应用了最新版汉化补丁的ROM文件！".to_owned());
                }
                Ok(PatchSource::Unsupported) => {
                  self.fail("不支持输入的 ROM 文件！".to_owned());
                }
                Err(e) => self.fail(e),
              }
            }
          }
          if self.input_file_path_c4.is_empty() {
            ui.add_enabled(false, reset_btn(svg_icon_size));
          } else if ui.add(reset_btn(svg_icon_size)).clicked() {
            self.input_file_path_c4.clear();
            self.input_file_crc32 = None;
          }
          ui.add(
            egui::TextEdit::singleline(&mut self.input_file_path_c4)
              .interactive(false)
              .hint_text("ROM文件路径")
              .desired_width(f32::INFINITY),
          );
        });
        ui.end_row();

        // 第2行 -> 指定输出文件保存目录  【选择目录】【重置】
        Label::new("指定输出文件保存目录")
          .selectable(false)
          .ui(ui)
          .on_hover_text("自定义输出的汉化版ROM文件路径（推荐留空），若留空则将转换后的ROM文件保存到输入文件的同目录下。");
        ui.horizontal(|ui| {
          if ui.add(select_dir_btn(svg_icon_size)).clicked() {
            if let Some(file) = FileDialog::new().pick_folder() {
              self.out_dir_c4 = file.to_string_lossy().into_owned();
            }
          }
          if self.out_dir_c4.is_empty() {
            ui.add_enabled(false, reset_btn(svg_icon_size));
          } else if ui.add(reset_btn(svg_icon_size)).clicked() {
            self.out_dir_c4.clear();
          }
          ui.add(
            egui::TextEdit::singleline(&mut self.out_dir_c4)
              .interactive(false)
              .hint_text("最新版汉化ROM输出路径")
              .desired_width(f32::INFINITY),
          );
        });
        ui.end_row();
      });
  }

  pub fn run_c4(&mut self) {
    let input_path = Path::new(&self.input_file_path_c4);

    let rom = match read_bytes(input_path, Some(MAX_ROM_SIZE)) {
      Ok(bytes) => bytes,
      Err(e) => return self.fail(e.to_string()),
    };

    let patched = match apply_chinese_patch(rom) {
      Ok(patched) => patched,
      Err(e) => return self.fail(e.to_string()),
    };

    let out_dir = resolve_output_dir(
      input_path,
      (!self.out_dir_c4.is_empty()).then(|| Path::new(&self.out_dir_c4)),
    );
    if let Err(e) = write_named_bytes(&out_dir, &patched_rom_file_name(), &patched.bytes) {
      return self.fail(e.to_string());
    }

    // More detailed output logs, to improve user experience.
    // Sh*t code, but at least it works.
    self.out_log_c4.clear();
    let _ = write!(
      self.out_log_c4,
      "输出文件：\n{}\n\n",
      out_dir.join(patched_rom_file_name()).display()
    );
    self.out_log_c4.push_str("运行概况：\n");

    let mut result_index = 0;

    // input_file_crc32 与 input_file_path_c4 总是成对地设置与清除，
    // 运行按钮只在路径非空时可用，所以此处 CRC32 必然存在
    let stored_crc32 = self
      .input_file_crc32
      .expect("input_file_crc32 与 input_file_path_c4 总是成对设置与清除");

    if stored_crc32 != patched.source_crc32 {
      result_index += 1;
      let _ = writeln!(
        self.out_log_c4,
        "{result_index}. 输入的源文件（“{}”）和运行前提供的源文件（“{}”）不一致！",
        rom_display_name(stored_crc32).unwrap(),
        rom_display_name(patched.source_crc32).unwrap()
      )
      .unwrap();
    }

    result_index += 1;
    let _ = writeln!(
      self.out_log_c4,
      "{result_index}. 输入的源文件版本信息：{}",
      rom_display_name(stored_crc32).unwrap()
    );
  }

  pub fn show_log_c4(&mut self, ui: &mut Ui) {
    Label::new(&self.out_log_c4).selectable(false).ui(ui);
  }
}
