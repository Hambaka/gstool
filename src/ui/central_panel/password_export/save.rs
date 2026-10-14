use std::{
  fmt::Write as _,
  path::{Path, PathBuf},
};

use egui::{Checkbox, Grid, Label, Ui, Widget};

use crate::{
  App,
  core::{
    enums::{GameRegion, PasswordGrade, PasswordVersion},
    password::encode::from_raw_save,
    save::raw::{SaveSlots, parse_save_slots},
  },
  io::{
    derived_output_dir, read_bytes, sheet_from_password_bytes, sheet_from_save_slot, slot_dir_name,
    write_cheat_code, write_memory_dump, write_password_data_text, write_password_text,
  },
  logic::command::SubCommandEnum,
  widgets::{
    buttons::pwd_info_btn,
    rows::{PathAction, checkbox_row, combo_row, dir_row, file_row, row_label},
  },
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
        row_label(
          ui,
          "存档导出选项",
          "根据需求进行勾选。",
          self.input_file_path_c1s1.is_empty(),
        );
        ui.add_enabled_ui(self.input_file_path_c1s1.is_empty(), |ui| {
          ui.radio_value(&mut self.to_export_all_save, false, "仅导出通关存档");
          ui.radio_value(&mut self.to_export_all_save, true, "导出全部存档（无论是否通关）");
        });
        ui.end_row();
        // 第2行
        match file_row(
          ui,
          "输入文件路径",
          "《黄金太阳 开启的封印》的存档文件。",
          "存档文件路径",
          &mut self.input_file_path_c1s1,
          svg_icon_size,
        ) {
          PathAction::Picked(file) => {
            // 验证存档文件
            match validate_input_file(&file, self.to_export_all_save) {
              Ok(save_slots) => {
                self.input_file_path_c1s1 = file.to_string_lossy().to_string();
                self.sources.from_save = save_slots.slots;

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
          PathAction::Reset => {
            self.to_export_all_save = false;

            self.input_file_path_c1s1.clear();
            self.sources.reset_save();

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
          PathAction::None => {}
        }

        // 第3行
        combo_row(
          ui,
          "密码级别",
          "目标密码级别，默认值为“金”",
          self.sources.has_save(),
          "ID_CB_C1S1_密码级别",
          self.pwd_grade_c1s1.short_name(),
          |ui| {
            for ele in PasswordGrade::ALL {
              ui.selectable_value(&mut self.pwd_grade_c1s1, ele, ele.short_name());
            }
          },
        );

        // 第4行
        combo_row(
          ui,
          "密码文本版本",
          "生成指定版本的密码文本文件，若留空则自动指定默认值“英文”。",
          self.sources.has_save(),
          "ID_CB_C1S1_密码文本版本",
          match &self.pwd_text_ver_c1s1 {
            Some(password_version_enum) => password_version_enum.name(),
            None => "无",
          },
          |ui| {
            ui.selectable_value(&mut self.pwd_text_ver_c1s1, None, "无");
            for ele in PasswordVersion::ALL {
              ui.selectable_value(&mut self.pwd_text_ver_c1s1, Some(ele), ele.name());
            }
          },
        );

        // 第5行
        checkbox_row(
          ui,
          "生成内存转储文件",
          "若勾选，则生成密码的内存转储文件。",
          self.sources.has_save(),
          &mut self.to_export_pwd_bin_c1s1,
        );

        // 第6行
        combo_row(
          ui,
          "生成密码金手指",
          "若选择了值，则生成指定版本的金手指文本文件。",
          self.sources.has_save(),
          "ID_CB_C1S1_生成密码金手指",
          match &self.pwd_cheat_ver_c1s1 {
            Some(version_enum) => version_enum.short_name(),
            None => "无",
          },
          |ui| {
            ui.selectable_value(&mut self.pwd_cheat_ver_c1s1, None, "无");
            for ele in GameRegion::ALL {
              ui.selectable_value(&mut self.pwd_cheat_ver_c1s1, Some(ele), ele.short_name());
            }
          },
        );

        // 第7行（复选框与预览按钮同处一行，未用 checkbox_row）
        row_label(
          ui,
          "导出密码数据至文本文件",
          "若勾选，则导出适用于 Dyrati 的“Golden Sun Password Generator”表格的密码数据文本文件。",
          self.sources.has_save(),
        );
        ui.horizontal(|ui| {
          ui.add_enabled(
            self.sources.has_save(),
            Checkbox::without_text(&mut self.to_export_pwd_data_text_c1s1),
          );
          if ui
            .add_enabled(self.sources.has_save(), pwd_info_btn(svg_icon_size))
            .clicked()
          {
            self.read_from(SubCommandEnum::First);
          }
        });
        ui.end_row();

        // 第8行
        match dir_row(
          ui,
          "指定输出文件保存目录",
          "自定义输出文件夹（推荐留空），若留空则在输入文件的同目录下创建文件夹保存生成的所有文件。",
          "若留空，则输出目录与输入的存档文件所在目录相同",
          &mut self.out_dir_c1s1,
          svg_icon_size,
        ) {
          PathAction::Picked(dir) => {
            self.out_dir_c1s1 = dir.to_string_lossy().into_owned();
          }
          PathAction::Reset => {
            self.out_dir_c1s1.clear();
          }
          PathAction::None => {}
        }
      });
  }

  pub fn run_c1s1(&mut self) {
    // Create main output directory.
    let output_dir_str = if self.out_dir_c1s1.is_empty() {
      derived_output_dir(Path::new(&self.input_file_path_c1s1))
    } else {
      PathBuf::from(&self.out_dir_c1s1)
    };

    for (key, val_option) in self.sources.from_save.iter().enumerate() {
      if let Some(val) = val_option {
        let target_password_bytes = from_raw_save(&val.data, self.pwd_grade_c1s1);
        // Key is save slot number: 0, 1, 2 -> 1, 2, 3
        let sub_dir_name = slot_dir_name(key, val.is_clear_data);
        let sub_dir_str = output_dir_str.join(sub_dir_name);

        if let Some(password_version) = self.pwd_text_ver_c1s1 {
          if let Err(e) =
            write_password_text(&target_password_bytes, password_version, &sub_dir_str)
          {
            self.err_msg = e.to_string();
            self.err_modal_open = true;
            return;
          }
        }

        if self.to_export_pwd_bin_c1s1 {
          if let Err(e) = write_memory_dump(&target_password_bytes, &sub_dir_str) {
            self.err_msg = e.to_string();
            self.err_modal_open = true;
            return;
          }
        }

        if let Some(cheat_version) = self.pwd_cheat_ver_c1s1 {
          if let Err(e) = write_cheat_code(
            &target_password_bytes,
            self.pwd_grade_c1s1,
            cheat_version,
            &sub_dir_str,
          ) {
            self.err_msg = e.to_string();
            self.err_modal_open = true;
            return;
          }
        }

        if self.to_export_pwd_data_text_c1s1 {
          if self.pwd_grade_c1s1 == PasswordGrade::Gold {
            if let Err(e) = write_password_data_text(&sheet_from_save_slot(&val.data), &sub_dir_str)
            {
              self.err_msg = e.to_string();
              self.err_modal_open = true;
              return;
            }
          } else if let Err(e) = write_password_data_text(
            &sheet_from_password_bytes(&target_password_bytes, self.pwd_grade_c1s1),
            &sub_dir_str,
          ) {
            self.err_msg = e.to_string();
            self.err_modal_open = true;
            return;
          }
        }

        // More detailed output logs, to improve user experience.
        // Sh*t code, but at least it works.
        self.out_log_c1s1.clear();
        let _ = write!(
          self.out_log_c1s1,
          "输出目录：\n{}\n\n",
          output_dir_str.to_string_lossy()
        );
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

        let password_grade_cn = self.pwd_grade_c1s1.short_name();

        if let Some(password_version) = self.pwd_text_ver_c1s1 {
          result_index += 1;
          let password_version_cn = password_version.name();
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
          let cheat_version_cn = cheat_version.short_name();
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

fn validate_input_file(path: &Path, include_non_clear: bool) -> Result<SaveSlots, String> {
  let bytes = read_bytes(path, None).map_err(|e| e.to_string())?;
  parse_save_slots(&bytes, include_non_clear).map_err(|e| e.to_string())
}
