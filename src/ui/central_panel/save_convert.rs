use std::{
  fmt::Write as _,
  fs::{File, create_dir_all},
  io::Write,
  path::{Path, PathBuf},
};

use egui::{ComboBox, Label, RichText, Ui, Widget};
use rfd::FileDialog;

use crate::{
  App,
  core::{
    enums::{GameRegion, GameTitle, NameType},
    save::{GameInfo, convert_save, raw::parse_convertible_save},
  },
  io::read_bytes,
  widgets::buttons::{reset_btn, select_dir_btn, select_file_btn},
};

const DESC_C2: &str = r"使用该功能可转换《黄金太阳 开启的封印》或《黄金太阳 失落的时代》存档的版本：
1. 修改主角团所有成员的名字为指定语言版本的默认名
2. 通过修改存档中的游戏构建日期来转换存档版本

注：
1. 如果存档中的构建日期与游戏 ROM 中的构建日期不匹配，游戏将强制玩家从神殿开始游戏。
2. Mobile&elffinal/SOMA 的《黄金太阳 失落的时代》汉化版基于美版制作，所以 ROM 中的构建日期同美版。
3. 《黄金太阳 开启的封印》新汉化版基于日版制作，所以 ROM 中的构建日期同日版。";

impl App {
  pub fn show_c2(&mut self, ui: &mut Ui, svg_icon_size: f32) {
    // 正式界面
    Label::new(DESC_C2).selectable(false).ui(ui);

    ui.add_space(16.0);

    egui::Grid::new("ID_GRID_C2")
      .num_columns(2)
      .spacing([40.0, 4.0])
      .striped(true)
      .show(ui, |ui| {
        // 第1行 -> 输入文件路径  【选择文件】【重置】
        Label::new("输入文件路径")
          .selectable(false)
          .ui(ui)
          .on_hover_text("《黄金太阳 开启的封印》或《黄金太阳 失落的时代》的存档文件");
        ui.horizontal(|ui| {
          if ui.add(select_file_btn(svg_icon_size)).clicked() {
            if let Some(file) = FileDialog::new().pick_file() {
              // 验证存档文件
              let result = validate_input_file(&file);
              match result {
                Ok((save_vec, game_info)) => {
                  self.input_file_path_c2 = file.to_string_lossy().to_string();
                  self.save_file = save_vec;
                  self.save_src_type_index = Some(game_info);

                  self.build_date_ver = None;
                  self.default_name_ver = None;
                }
                Err(e) => {
                  e.clone_into(&mut self.err_msg);
                  self.err_modal_open = true;
                }
              }
            }
          }
          if self.input_file_path_c2.is_empty() {
            ui.add_enabled(false, reset_btn(svg_icon_size));
          } else if ui.add(reset_btn(svg_icon_size)).clicked() {
            self.input_file_path_c2.clear();
            self.save_file.clear();
            self.save_src_type_index = None;

            self.build_date_ver = None;
            self.default_name_ver = None;
          }
          ui.add(
            egui::TextEdit::singleline(&mut self.input_file_path_c2)
              .interactive(false)
              .hint_text("存档文件路径")
              .desired_width(f32::INFINITY),
          );
        });
        ui.end_row();

        // 第2行 -> 名字版本
        if let Some(val_result) = self.save_src_type_index {
          Label::new("名字版本")
            .selectable(false)
            .ui(ui)
            .on_hover_text("若选择了值，则将存档中的主角团所有成员的名字转换至指定版本的默认名；若留空则不修改。");
          ComboBox::from_id_salt("ID_CB_C2_名字版本")
            .width(100.0)
            .selected_text(match &self.default_name_ver {
              Some(name_type) => name_type.name(),
              None => "无",
            })
            .show_ui(ui, |ui| {
              ui.selectable_value(&mut self.default_name_ver, None, "无");
              ui.selectable_value(&mut self.default_name_ver, Some(NameType::Japanese), NameType::Japanese.name());
              ui.selectable_value(&mut self.default_name_ver, Some(NameType::English), NameType::English.name());
              ui.selectable_value(&mut self.default_name_ver, Some(NameType::German), NameType::German.name());
              ui.selectable_value(&mut self.default_name_ver, Some(NameType::Spanish), NameType::Spanish.name());
              ui.selectable_value(&mut self.default_name_ver, Some(NameType::French), NameType::French.name());
              ui.selectable_value(&mut self.default_name_ver, Some(NameType::Italian), NameType::Italian.name());
              ui.add_enabled_ui(matches!(val_result.title, GameTitle::TheBrokenSeal), |ui| {
                ui.selectable_value(
                  &mut self.default_name_ver,
                  Some(NameType::ChineseFanTranslation2023Team),
                  NameType::ChineseFanTranslation2023Team.name(),
                );
              });
              ui.add_enabled_ui(matches!(val_result.title, GameTitle::TheLostAge), |ui| {
                ui.selectable_value(
                  &mut self.default_name_ver,
                  Some(NameType::ChineseFanTranslationMobileTeam),
                  NameType::ChineseFanTranslationMobileTeam.name(),
                );
              });
            });
        } else {
          Label::new(RichText::new("名字版本").weak())
            .selectable(false)
            .ui(ui)
            .on_hover_text("若选择了值，则将存档中的主角团所有成员的名字转换至指定版本的默认名；若留空则不修改。");
          ui.add_enabled_ui(false, |ui| {
            ComboBox::from_id_salt("ID_CB_C2_名字版本（不可用）")
              .width(100.0)
              .selected_text("无")
              .show_ui(ui, |_ui| {});
          });
        }
        ui.end_row();

        // 第3行 -> 构建日期版本
        if let Some(val_result) = self.save_src_type_index {
          Label::new("构建日期版本")
            .selectable(false)
            .ui(ui)
            .on_hover_text("若选择了值，则将存档中的构建日期转换至指定版本的构建日期；若留空则不修改。");
          ComboBox::from_id_salt("ID_CB_C2_构建日期版本")
            .width(100.0)
            .selected_text(match &self.build_date_ver {
              Some(version_enum) => version_enum.short_name(),
              None => "无",
            })
            .show_ui(ui, |ui| {
              ui.selectable_value(&mut self.build_date_ver, None, "无");
              for ele in GameRegion::ALL {
                ui.add_enabled_ui(ele != val_result.region, |ui| {
                  ui.selectable_value(&mut self.build_date_ver, Some(ele), ele.short_name());
                });
              }
            });
        } else {
          Label::new(RichText::new("构建日期版本").weak())
            .selectable(false)
            .ui(ui)
            .on_hover_text("若选择了值，则将存档中的构建日期转换至指定版本的构建日期；若留空则不修改。");
          ui.add_enabled_ui(false, |ui| {
            ComboBox::from_id_salt("ID_CB_C2_构建日期版本（不可用）")
              .width(100.0)
              .selected_text("无")
              .show_ui(ui, |_ui| {});
          });
        }
        ui.end_row();

        // 第4行 -> 指定输出文件保存目录  【选择目录】【重置】
        Label::new("指定输出文件保存目录")
          .selectable(false)
          .ui(ui)
          .on_hover_text("自定义输出的存档文件路径（推荐留空），若留空则将转换后的存档文件保存到输入文件的同目录下。");
        ui.horizontal(|ui| {
          if ui.add(select_dir_btn(svg_icon_size)).clicked() {
            if let Some(file) = FileDialog::new().pick_folder() {
              self.out_dir_c2 = file.to_string_lossy().into_owned();
            }
          }
          if self.out_dir_c2.is_empty() {
            ui.add_enabled(false, reset_btn(svg_icon_size));
          } else if ui.add(reset_btn(svg_icon_size)).clicked() {
            self.out_dir_c2.clear();
          }
          ui.add(
            egui::TextEdit::singleline(&mut self.out_dir_c2)
              .interactive(false)
              .hint_text("存档文件输出路径")
              .desired_width(f32::INFINITY),
          );
        });
        ui.end_row();
      });
  }

  pub fn run_c2(&mut self) {
    let mut new_save_file = self.save_file.clone();

    // Convert save data.
    convert_save(
      &mut new_save_file,
      self.save_src_type_index.unwrap().title,
      self.save_src_type_index.unwrap().first_slot_index,
      self.default_name_ver,
      self.build_date_ver,
    );

    // Start to create and write output save file.
    let output_path;

    let input_path = Path::new(&self.input_file_path_c2);
    let file_stem = input_path.file_stem().unwrap().to_str().unwrap();
    let file_extension = if input_path.extension().is_none() {
      ""
    } else {
      input_path.extension().unwrap().to_str().unwrap()
    };
    let mut file_name_str = String::new();
    file_name_str.push_str(file_stem);
    file_name_str.push_str("_转换");

    if !file_extension.is_empty() {
      file_name_str.push('.');
      file_name_str.push_str(file_extension);
    }

    if self.out_dir_c2.is_empty() {
      let parent = input_path.parent().unwrap().to_str().unwrap();
      output_path = PathBuf::from(parent).join(file_name_str);
    } else {
      if create_dir_all(self.out_dir_c2.clone()).is_err() {
        "创建目录失败！".clone_into(&mut self.err_msg);
        self.err_modal_open = true;
        return;
      }
      output_path = PathBuf::from(self.out_dir_c2.clone()).join(file_name_str);
    }

    let output_file = File::create(output_path.clone());
    if let Ok(mut file) = output_file {
      if file.write_all(&new_save_file).is_err() {
        self.err_msg = format!("创建 “{}” 失败！", output_path.to_str().unwrap());
        self.err_modal_open = true;
        return;
      }
    } else {
      self.err_msg = format!("创建 “{}” 失败！", output_path.to_str().unwrap());
      self.err_modal_open = true;
      return;
    }

    // More detailed output logs, to improve user experience.
    // Sh*t code, but at least it works.
    self.out_log_c2.clear();
    let _ = write!(
      self.out_log_c2,
      "输出文件：\n{}\n\n",
      output_path.to_str().unwrap()
    );
    self.out_log_c2.push_str("运行概况：\n");

    let mut result_index = 0;
    result_index += 1;
    let game_type_cn = match self.save_src_type_index.unwrap().title {
      GameTitle::TheBrokenSeal => "《黄金太阳 开启的封印》",
      GameTitle::TheLostAge => "《黄金太阳 失落的时代》",
    };
    let _ = writeln!(
      self.out_log_c2,
      "{result_index}. 输入的源文件为{game_type_cn}的存档文件。"
    );

    if let Some(name_type) = self.default_name_ver {
      result_index += 1;
      let name_type_cn = name_type.name();
      let _ = writeln!(
        self.out_log_c2,
        "{result_index}. 生成的存档文件已将主角团成员的名字转换为{name_type_cn}版的默认名。"
      );

      if let Some(build_date_type) = self.build_date_ver {
        result_index += 1;
        let build_date_type_cn = build_date_type.short_name();
        let _ = writeln!(
          self.out_log_c2,
          "{result_index}. 生成的存档文件已将构建日期转换为{build_date_type_cn}版。"
        );
      }
    }
  }

  pub fn show_log_c2(&mut self, ui: &mut Ui) {
    Label::new(&self.out_log_c2).selectable(false).ui(ui);
  }
}

fn validate_input_file(path: &Path) -> Result<(Vec<u8>, GameInfo), String> {
  let bytes = read_bytes(path, None).map_err(|e| e.to_string())?;
  let info = parse_convertible_save(&bytes).map_err(|e| e.to_string())?;
  Ok((bytes, info))
}
