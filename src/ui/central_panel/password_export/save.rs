use std::{fmt::Write as _, fs::File, io::Read};

use egui::{Checkbox, ComboBox, Grid, Label, RichText, Ui, Widget};
use rfd::FileDialog;

use crate::{
  App,
  app::SubCommandEnum,
  core::{
    enums::{GameVersion, PasswordGrade, PasswordVersion},
    output,
    password::dyrati::from_password_bytes,
    save::{SaveSlot, collect_slots, find_first_slot_index},
  },
  widgets::buttons::{pwd_info_btn, reset_btn, select_dir_btn, select_file_btn},
};

const DESC_C1S1: &str = r"“生成内存转储文件”，“生成密码金手指”，“导出密码数据至文本文件”。
程序默认密码级别为“金”，密码文本版本为“英文”，因此以上三项都是可以留空或者不勾选的可选选项。";

impl App {
  pub fn show_c1s1(&mut self, ui: &mut Ui, svg_icon_size: f32) {
    Label::new(DESC_C1S1).selectable(false).ui(ui);

    ui.add_space(16.0);

    Grid::new("ID_GRID_C1S1")
      .num_columns(2)
      .spacing([20.0, 5.0])
      .striped(true)
      .show(ui, |ui| {
        // 第1行
        if self.input_file_path_c1s1.is_empty() {
          Label::new("存档导出选项").selectable(false).ui(ui).on_hover_text("根据需求进行勾选。");
        } else {
          Label::new(RichText::new("存档导出选项").weak())
            .selectable(false)
            .ui(ui)
            .on_hover_text("根据需求进行勾选。");
        }
        ui.add_enabled_ui(self.input_file_path_c1s1.is_empty(), |ui| {
          ui.radio_value(&mut self.to_export_all_save, false, "仅导出通关存档");
          ui.radio_value(&mut self.to_export_all_save, true, "导出全部存档（无论是否通关）");
        });
        ui.end_row();
        // 第2行
        Label::new("输入文件路径")
          .selectable(false)
          .ui(ui)
          .on_hover_text("《黄金太阳 开启的封印》的存档文件。");
        ui.horizontal(|ui| {
          if ui.add(select_file_btn(svg_icon_size)).clicked() {
            if let Some(file) = FileDialog::new().pick_file() {
              let new_file_path = file.to_string_lossy().into_owned();
              // 验证存档文件
              let result = validate_input_file(&new_file_path, self.to_export_all_save);
              match result {
                Ok((raw_save_data, min_slot)) => {
                  self.input_file_path_c1s1 = new_file_path;
                  self.raw_save_data = raw_save_data;
                  self.save_is_available = Some(min_slot);

                  self.pwd_grade_c1s1 = PasswordGrade::Gold;
                  self.pwd_text_ver_c1s1 = Some(PasswordVersion::English);
                  self.to_export_pwd_bin_c1s1 = false;
                  self.pwd_cheat_ver_c1s1 = None;
                  self.to_export_pwd_data_text_c1s1 = false;
                }
                Err(e) => {
                  e.clone_into(&mut self.err_msg);
                  self.err_modal_open = true;
                }
              }
            }
          }
          if self.input_file_path_c1s1.is_empty() {
            ui.add_enabled(false, reset_btn(svg_icon_size));
          } else if ui.add(reset_btn(svg_icon_size)).clicked() {
            self.to_export_all_save = false;

            self.input_file_path_c1s1.clear();
            self.raw_save_data = [None, None, None];
            self.save_is_available = None;

            self.pwd_grade_c1s1 = PasswordGrade::Gold;
            self.pwd_text_ver_c1s1 = Some(PasswordVersion::English);
            self.to_export_pwd_bin_c1s1 = false;
            self.pwd_cheat_ver_c1s1 = None;
            self.to_export_pwd_data_text_c1s1 = false;

            for i in 0..3 {
              if self.pwd_src[i] == Some(SubCommandEnum::First) {
                self.pwd_src[i] = None;
                self.pwd_src_slot[i] = None;
              }
            }
          }
          ui.add(
            egui::TextEdit::singleline(&mut self.input_file_path_c1s1)
              .interactive(false)
              .hint_text("存档文件路径")
              .desired_width(f32::INFINITY),
          );
        });
        ui.end_row();
        // 第3行
        if self.save_is_available.is_some() {
          Label::new("密码级别")
            .selectable(false)
            .ui(ui)
            .on_hover_text("目标密码级别，默认值为“金”");
          ComboBox::from_id_salt("ID_CB_C1S1_密码级别")
            .width(100.0)
            .selected_text(self.pwd_grade_c1s1.get_name())
            .show_ui(ui, |ui| {
              for ele in PasswordGrade::PASSWORD_GRADES {
                ui.selectable_value(&mut self.pwd_grade_c1s1, ele, ele.get_name());
              }
            });
        } else {
          Label::new(RichText::new("密码级别").weak())
            .selectable(false)
            .ui(ui)
            .on_hover_text("目标密码级别，默认值为“金”");
          ui.add_enabled_ui(false, |ui| {
            ComboBox::from_id_salt("ID_CB_C1S1_密码级别（不可用）")
              .width(100.0)
              .selected_text("无")
              .show_ui(ui, |_ui| {});
          });
        }
        ui.end_row();
        // 第4行
        if self.save_is_available.is_some() {
          Label::new("密码文本版本")
            .selectable(false)
            .ui(ui)
            .on_hover_text("生成指定版本的密码文本文件，若留空则自动指定默认值“英文”。");
          ComboBox::from_id_salt("ID_CB_C1S1_密码文本版本")
            .width(100.0)
            .selected_text(match &self.pwd_text_ver_c1s1 {
              Some(password_version_enum) => password_version_enum.get_name(),
              None => "无",
            })
            .show_ui(ui, |ui| {
              ui.selectable_value(&mut self.pwd_text_ver_c1s1, None, "无");
              for ele in PasswordVersion::PASSWORD_VERSIONS {
                ui.selectable_value(&mut self.pwd_text_ver_c1s1, Some(ele), ele.get_name());
              }
            });
        } else {
          Label::new(RichText::new("密码文本版本").weak())
            .selectable(false)
            .ui(ui)
            .on_hover_text("生成指定版本的密码文本文件，若留空则自动指定默认值“英文”。");
          ui.add_enabled_ui(false, |ui| {
            ComboBox::from_id_salt("ID_CB_C1S1_密码文本版本（不可用）")
              .width(100.0)
              .selected_text("无")
              .show_ui(ui, |_ui| {});
          });
        }
        ui.end_row();
        // 第5行
        if self.save_is_available.is_some() {
          Label::new("生成内存转储文件")
            .selectable(false)
            .ui(ui)
            .on_hover_text("若勾选，则生成密码的内存转储文件。");
          Checkbox::without_text(&mut self.to_export_pwd_bin_c1s1).ui(ui);
        } else {
          Label::new(RichText::new("生成内存转储文件").weak())
            .selectable(false)
            .ui(ui)
            .on_hover_text("若勾选，则生成密码的内存转储文件。");
          ui.add_enabled(false, Checkbox::without_text(&mut self.to_export_pwd_bin_c1s1));
        }
        ui.end_row();
        // 第6行
        if self.save_is_available.is_some() {
          Label::new("生成密码金手指")
            .selectable(false)
            .ui(ui)
            .on_hover_text("若选择了值，则生成指定版本的金手指文本文件。");
          ComboBox::from_id_salt("ID_CB_C1S1_生成密码金手指")
            .width(100.0)
            .selected_text(match &self.pwd_cheat_ver_c1s1 {
              Some(version_enum) => version_enum.get_name(),
              None => "无",
            })
            .show_ui(ui, |ui| {
              ui.selectable_value(&mut self.pwd_cheat_ver_c1s1, None, "无");
              for ele in GameVersion::GAME_VERSIONS {
                ui.selectable_value(&mut self.pwd_cheat_ver_c1s1, Some(ele), ele.get_name());
              }
            });
        } else {
          Label::new(RichText::new("生成密码金手指").weak())
            .selectable(false)
            .ui(ui)
            .on_hover_text("若选择了值，则生成指定版本的金手指文本文件。");
          ui.add_enabled_ui(false, |ui| {
            ComboBox::from_id_salt("ID_CB_C1S1_生成密码金手指（不可用）")
              .width(100.0)
              .selected_text("无")
              .show_ui(ui, |_ui| {});
          });
        }
        ui.end_row();
        // 第7行
        if self.save_is_available.is_some() {
          Label::new("导出密码数据至文本文件")
            .selectable(false)
            .ui(ui)
            .on_hover_text("若勾选，则导出适用于 Dyrati 的“Golden Sun Password Generator”表格的密码数据文本文件。");
          ui.horizontal(|ui| {
            Checkbox::without_text(&mut self.to_export_pwd_data_text_c1s1).ui(ui);
            if ui.add(pwd_info_btn(svg_icon_size)).clicked() {
              self.read_from(SubCommandEnum::First);
            }
          });
        } else {
          Label::new(RichText::new("导出密码数据至文本文件").weak())
            .selectable(false)
            .ui(ui)
            .on_hover_text("若勾选，则导出适用于 Dyrati 的“Golden Sun Password Generator”表格的密码数据文本文件。");
          ui.horizontal(|ui| {
            ui.add_enabled(false, Checkbox::without_text(&mut self.to_export_pwd_data_text_c1s1));
            ui.add_enabled(false, pwd_info_btn(svg_icon_size));
          });
        }
        ui.end_row();
        // 第8行
        Label::new("指定输出文件保存目录")
          .selectable(false)
          .ui(ui)
          .on_hover_text("自定义输出文件夹（推荐留空），若留空则在输入文件的同目录下创建文件夹保存生成的所有文件。");
        ui.horizontal(|ui| {
          if ui.add(select_dir_btn(svg_icon_size)).clicked() {
            if let Some(dir) = FileDialog::new().pick_folder() {
              self.out_dir_c1s1 = dir.to_string_lossy().into_owned();
            }
          }

          if self.out_dir_c1s1.is_empty() {
            ui.add_enabled(false, reset_btn(svg_icon_size));
          } else if ui.add(reset_btn(svg_icon_size)).clicked() {
            self.out_dir_c1s1.clear();
          }
          ui.add(
            egui::TextEdit::singleline(&mut self.out_dir_c1s1)
              .interactive(false)
              .hint_text("若留空，则输出目录与输入的存档文件所在目录相同")
              .desired_width(f32::INFINITY),
          );
        });
        ui.end_row();
      });
  }

  pub fn run_c1s1(&mut self) {
    // Create main output directory.
    let output_dir_str = if self.out_dir_c1s1.is_empty() {
      match output::create_output_dir(&self.input_file_path_c1s1, false) {
        Ok(o) => o,
        Err(e) => {
          e.clone_into(&mut self.err_msg);
          self.err_modal_open = true;
          return;
        }
      }
    } else {
      match output::create_output_dir(&self.out_dir_c1s1, true) {
        Ok(o) => o,
        Err(e) => {
          e.clone_into(&mut self.err_msg);
          self.err_modal_open = true;
          return;
        }
      }
    };

    for (key, val_option) in self.raw_save_data.iter().enumerate() {
      if let Some(val) = val_option {
        let password_bytes =
          crate::core::password::encode::from_raw_save(val.get_data(), self.pwd_grade_c1s1);
        // Key is save slot number: 0, 1, 2 -> 1, 2, 3
        let sub_dir_str =
          match output::create_sav_sub_dir(key + 1, val.get_is_clear(), &output_dir_str) {
            Ok(o) => o,
            Err(e) => {
              e.clone_into(&mut self.err_msg);
              self.err_modal_open = true;
              return;
            }
          };

        if let Some(password_version) = self.pwd_text_ver_c1s1 {
          if let Err(e) = output::write_password_text_file_with_bytes(
            &password_bytes,
            password_version,
            &sub_dir_str,
          ) {
            e.clone_into(&mut self.err_msg);
            self.err_modal_open = true;
            return;
          }
        }

        if self.to_export_pwd_bin_c1s1 {
          if let Err(e) = output::write_memory_dump_file(&password_bytes, &sub_dir_str) {
            e.clone_into(&mut self.err_msg);
            self.err_modal_open = true;
            return;
          }
        }

        if let Some(cheat_version) = self.pwd_cheat_ver_c1s1 {
          if let Err(e) = output::write_cheat_file(&password_bytes, cheat_version, &sub_dir_str) {
            e.clone_into(&mut self.err_msg);
            self.err_modal_open = true;
            return;
          }
        }

        if self.to_export_pwd_data_text_c1s1 {
          if self.pwd_grade_c1s1.is_gold() {
            if let Err(e) = output::write_game_data_text_file(
              &crate::core::password::dyrati::from_raw_save(val.get_data()),
              &sub_dir_str,
            ) {
              e.clone_into(&mut self.err_msg);
              self.err_modal_open = true;
              return;
            }
          } else if let Err(e) = output::write_game_data_text_file(
            &from_password_bytes(&password_bytes, self.pwd_grade_c1s1),
            &sub_dir_str,
          ) {
            e.clone_into(&mut self.err_msg);
            self.err_modal_open = true;
            return;
          }
        }

        // More detailed output logs, to improve user experience.
        // Sh*t code, but at least it works.
        self.out_log_c1s1.clear();
        let _ = write!(self.out_log_c1s1, "输出目录：\n{output_dir_str}\n\n");
        self.out_log_c1s1.push_str("运行概况：\n");

        let mut result_index = 1;
        if self.to_export_all_save {
          let _ = writeln!(
            self.out_log_c1s1,
            "{result_index}. 已勾选“导出所有存档”，导出包含未通关存档在内的所有存档的数据。"
          );
        } else {
          let _ = writeln!(
            self.out_log_c1s1,
            "{result_index}. 未勾选“导出所有存档”，仅导出通关存档的数据。"
          );
        }

        let password_grade_cn = self.pwd_grade_c1s1.get_name();

        if let Some(password_version) = self.pwd_text_ver_c1s1 {
          result_index += 1;
          let password_version_cn = password_version.get_name();
          if self.to_export_all_save {
            let _ = writeln!(
              self.out_log_c1s1,
              "{result_index}. 已生成存档记录所对应的{password_grade_cn}级别、{password_version_cn}版的“密码文本.txt”文件。"
            );
          } else {
            let _ = writeln!(
              self.out_log_c1s1,
              "{result_index}. 已生成通关的存档记录所对应的{password_grade_cn}级别、{password_version_cn}版的“密码文本.txt”文件。"
            );
          }
        }

        if self.to_export_pwd_bin_c1s1 {
          result_index += 1;
          if self.to_export_all_save {
            let _ = writeln!(
              self.out_log_c1s1,
              "{result_index}. 已生成存档记录所对应的{password_grade_cn}级别“密码内存转储.dmp”文件。"
            );
          } else {
            let _ = writeln!(
              self.out_log_c1s1,
              "{result_index}. 已生成通关的存档记录所对应的{password_grade_cn}级别“密码内存转储.dmp”文件。"
            );
          }
        }

        if let Some(cheat_version) = self.pwd_cheat_ver_c1s1 {
          result_index += 1;
          let cheat_version_cn = cheat_version.get_name();
          if self.to_export_all_save {
            let _ = writeln!(
              self.out_log_c1s1,
              "{result_index}. 已生成存档记录所对应的{password_grade_cn}级别的、适用于{cheat_version_cn}版《黄金太阳 失落的时代》的“密码金手指.txt”文件。"
            );
          } else {
            let _ = writeln!(
              self.out_log_c1s1,
              "{result_index}. 已生成通关的存档记录所对应的{password_grade_cn}级别的、适用于{cheat_version_cn}版《黄金太阳 失落的时代》的“密码金手指.txt”文件。"
            );
          }
        }

        if self.to_export_pwd_data_text_c1s1 {
          result_index += 1;
          if self.to_export_all_save {
            let _ = writeln!(
              self.out_log_c1s1,
              "{result_index}. 已生成存档记录所对应的{password_grade_cn}级别“密码数据.txt”文件"
            );
          } else {
            let _ = writeln!(
              self.out_log_c1s1,
              "{result_index}. 已生成通关的存档记录所对应的{password_grade_cn}级别“密码数据.txt”文件"
            );
          }
        }
      }
    }
  }
}

fn validate_input_file(
  path_str: &str,
  to_export_all_save: bool,
) -> Result<([Option<SaveSlot>; 3], usize), &'static str> {
  // Read save file.
  let input_file_result = File::open(path_str);
  let mut sav_file;
  if input_file_result.is_err() {
    return Err("打开存档文件时发生错误！");
  }
  sav_file = input_file_result.unwrap();

  /* Check the size of save file.
  The size of save file should be 64KB,
  though the .SaveRAM file created by Bizhawk is 128KB.
  The size of the .srm file created by the libretro fork of VBA-M, VBA-Next and
  the .4gs file created by 44VBA is 136KB.
  Although the size of those files is over 64KB, seems it only use first 64KB space to store save data? */
  let file_size = sav_file.metadata().unwrap().len();
  if file_size != 0x10000 && file_size != 0x20000 && file_size != 0x22000 {
    return Err("存档文件的大小应为 64/128/136 KB！");
  }

  // Get raw save.
  let mut raw_save_file = Vec::new();
  sav_file.read_to_end(&mut raw_save_file).unwrap();

  let loop_start_index;
  if let Some(index) = find_first_slot_index(&raw_save_file) {
    loop_start_index = index;
  } else {
    return Err("输入的不是《黄金太阳 开启的封印》的存档文件！或者存档文件中没有数据！");
  }

  /* Get save data from save file with slot number.
  Also check if the save data is clear data. */
  let save_data_map = collect_slots(to_export_all_save, &raw_save_file, loop_start_index);
  if save_data_map.is_empty() {
    if to_export_all_save {
      return Err("存档文件中没有数据！");
    }
    return Err("存档文件中没有通关数据！");
  }

  let mut raw_save_data = [None, None, None];
  let mut min_slot = 0;

  for (key, val) in save_data_map {
    if key < min_slot {
      min_slot = key;
    }
    raw_save_data[key as usize] = Some(val);
  }

  Ok((raw_save_data, min_slot as usize))
}
