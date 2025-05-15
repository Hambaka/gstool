use core::f32;
use std::{fmt::Write as _, fs::File, io::Read};

use egui::{Checkbox, ComboBox, Grid, Label, RichText, Ui, Widget};
use rfd::FileDialog;

use crate::{
  App,
  app::GAME_VERSIONS,
  components::buttons::{reset_btn, select_dir_btn, select_file_btn},
  core::{
    convert,
    enums::PasswordGrade,
    output,
    save::{self, BitArray},
  },
};

use super::shared_view::{PASSWORD_GRADES, PASSWORD_VERSIONS};

const DESC_C1S3: &str = r"“密码级别”，“密码文本版本”，“生成密码金手指”，“导出密码数据至文本文件”。
以上四项虽然都是可以留空或者不勾选的可选选项，但是必须要有其中一项有选择值或者已勾选。";

impl App {
  pub fn show_c1s3(&mut self, ui: &mut Ui) {
    Label::new(DESC_C1S3).selectable(false).ui(ui);

    ui.add_space(16.0);

    Grid::new("（1.3）使用内存转储文件")
      .num_columns(2)
      .spacing([20.0, 5.0])
      .striped(true)
      .show(ui, |ui| {
        // 第1行
        Label::new("输入文件路径").selectable(false).ui(ui).on_hover_text("黄金太阳的密码内存转储文件");
        ui.horizontal(|ui| {
          if ui.add(select_file_btn()).clicked() {
            if let Some(file) = FileDialog::new().pick_file(){
              let new_file_path = file.to_string_lossy().into_owned();
              // 验证内存转储文件
              let result = validate_input_file(&new_file_path);
              match result {
                  Ok(o) => {
                    self.input_file_path_c1s3 = new_file_path;
                    self.dmp_bytes = o.0;
                    self.dmp_bits = Some(o.1);
                    self.dmp_src_grade = Some(o.2);

                    self.pwd_grade_c1s3 = None;
                    self.pwd_text_ver_c1s3 = None;
                    self.pwd_cheat_ver_c1s3 = None;
                    self.to_export_pwd_data_text_c1s3 = false;
                  },
                  Err(e) => {
                    e.clone_into(&mut self.err_msg);
                    self.modal_open = true;
                  },
              }
            }
          }
          if self.input_file_path_c1s3.is_empty() {
            ui.add_enabled(false, reset_btn());
          } else if ui.add(reset_btn()).clicked() {
            self.input_file_path_c1s3.clear();
            self.dmp_bytes.clear();
            self.dmp_bits = None;
            self.dmp_src_grade = None;

            self.pwd_grade_c1s3 = None;
            self.pwd_text_ver_c1s3 = None;
            self.pwd_cheat_ver_c1s3 = None;
            self.to_export_pwd_data_text_c1s3 = false;
          }
          ui.add(
            egui::TextEdit::singleline(&mut self.input_file_path_c1s3)
              .interactive(false)
              .hint_text("内存转储文件路径")
              .desired_width(f32::INFINITY),
          );
        });
        ui.end_row();
        // 第2行
        if let Some(source_grade) = self.dmp_src_grade {
          Label::new("密码级别").selectable(false).ui(ui).on_hover_text("目标密码级别（仅用于降级），指定级别不可高于输入文件的级别。若两者级别相同则忽略；若可降级则生成降级后的密码文本文件（及相关文件）；若留空，则目标级别同输入文件。");
          ComboBox::from_id_salt("（1.3）密码级别")
            .width(100.0)
            .selected_text(match &self.pwd_grade_c1s3 {
              Some(password_grade_enum) => password_grade_enum.get_name(),
              None => "无",
            })
            .show_ui(ui, |ui| {
              ui.selectable_value(&mut self.pwd_grade_c1s3, None, "无");
              for ele in PASSWORD_GRADES {
                ui.add_enabled_ui(source_grade.get_priority() < ele.get_priority(), |ui|{
                  ui.selectable_value(&mut self.pwd_grade_c1s3, Some(ele), ele.get_name());
                });
              }
            });
        } else {
          Label::new(RichText::new("密码级别").weak()).selectable(false).ui(ui).on_hover_text("目标密码级别（仅用于降级），指定级别不可高于输入文件的级别。若两者级别相同则忽略；若可降级则生成降级后的密码文本文件（及相关文件）；若留空，则目标级别同输入文件。");
          ui.add_enabled_ui(false, |ui|{
            ComboBox::from_id_salt("（1.3）密码级别（不可用）").width(100.0).selected_text("无").show_ui(ui, |_ui|{});
          });
        }
        ui.end_row();
        // 第3行
        if self.dmp_src_grade.is_some(){
          Label::new("密码文本版本").selectable(false).ui(ui).on_hover_text("若选择了值，则生成指定版本的密码文本文件。");
          ComboBox::from_id_salt("（1.3）密码文本版本")
            .width(100.0)
            .selected_text(match &self.pwd_text_ver_c1s3 {
              Some(password_version_enum) => password_version_enum.get_name(),
              None => "无",
            })
            .show_ui(ui, |ui| {
              ui.selectable_value(&mut self.pwd_text_ver_c1s3, None, "无");
              for ele in PASSWORD_VERSIONS {
                ui.selectable_value(&mut self.pwd_text_ver_c1s3, Some(ele), ele.get_name());
              }
            });
        } else {
          Label::new(RichText::new("密码文本版本").weak()).selectable(false).ui(ui).on_hover_text("若选择了值，则生成指定版本的密码文本文件。");
          ui.add_enabled_ui(false, |ui|{
            ComboBox::from_id_salt("（1.3）密码文本版本（不可用）").width(100.0).selected_text("无").show_ui(ui, |_ui|{});
          });
        }
        ui.end_row();
        // 第4行
        if self.dmp_src_grade.is_some(){
          Label::new("生成密码金手指").selectable(false).ui(ui).on_hover_text("若选择了值，则生成指定版本的金手指文本文件。");
          ComboBox::from_id_salt("（1.3）生成密码金手指")
            .width(100.0)
            .selected_text(match &self.pwd_cheat_ver_c1s3 {
              Some(version_enum) => version_enum.get_name(),
              None => "无",
            })
            .show_ui(ui, |ui| {
              ui.selectable_value(&mut self.pwd_cheat_ver_c1s3, None, "无");
              for ele in GAME_VERSIONS {
                ui.selectable_value(&mut self.pwd_cheat_ver_c1s3, Some(ele), ele.get_name());
              }
            });
        } else {
          Label::new(RichText::new("生成密码金手指").weak()).selectable(false).ui(ui).on_hover_text("若选择了值，则生成指定版本的金手指文本文件。");
          ui.add_enabled_ui(false, |ui|{
            ComboBox::from_id_salt("（1.3）生成密码金手指（不可用）").width(100.0).selected_text("无").show_ui(ui, |_ui|{});
          });
        }
        ui.end_row();
        // 第5行
        if self.dmp_src_grade.is_some(){
          Label::new("导出密码数据至文本文件").selectable(false).ui(ui).on_hover_text("若勾选，则导出适用于 Dyrati 的“Golden Sun Password Generator”表格的密码数据文本文件。");
          Checkbox::without_text(&mut self.to_export_pwd_data_text_c1s3).ui(ui);
        } else {
          Label::new(RichText::new("导出密码数据至文本文件").weak()).selectable(false).ui(ui).on_hover_text("若勾选，则导出适用于 Dyrati 的“Golden Sun Password Generator”表格的密码数据文本文件。");
          ui.add_enabled(false, Checkbox::without_text(&mut self.to_export_pwd_data_text_c1s3));
        }
        ui.end_row();
        // 第6行
        Label::new("指定输出文件保存目录").selectable(false).ui(ui).on_hover_text("自定义输出文件夹（推荐留空），若留空则在输入文件的同目录下创建文件夹保存生成的所有文件。");
        ui.horizontal(|ui| {
          if ui.add(select_dir_btn()).clicked() {
            if let Some(dir) = FileDialog::new().pick_folder(){
              self.out_dir_c1s3 = dir.to_string_lossy().into_owned();
            }
          }
          if self.out_dir_c1s3.is_empty() {
            ui.add_enabled(false, reset_btn());
          } else if ui.add(reset_btn()).clicked() {
            self.out_dir_c1s3.clear();
          }
          ui.add(
            egui::TextEdit::singleline(&mut self.out_dir_c1s3)
              .interactive(false)
              .hint_text("若留空，则输出目录与输入的内存转储文件所在目录相同")
              .desired_width(f32::INFINITY),
          );
        });
        ui.end_row();
      });
  }

  pub fn run_c1s3(&mut self) {
    let mut password_bits = BitArray::from(&self.dmp_bits.as_ref().unwrap().get_bits());

    // Generate save data, and set target password bytes
    let save_data =
      save::get_save_data_with_password_bits(&mut password_bits, self.dmp_src_grade.unwrap());
    let target_password_bytes = match self.pwd_grade_c1s3 {
      Some(password_grade) => save::get_password_bytes_with_save_data(&save_data, password_grade),
      None => self.dmp_bytes.clone(),
    };

    // Create output directory.
    let output_dir_str = if self.out_dir_c1s3.is_empty() {
      match output::create_output_dir(&self.input_file_path_c1s3, false) {
        Ok(o) => o,
        Err(e) => {
          e.clone_into(&mut self.err_msg);
          self.modal_open = true;
          return;
        }
      }
    } else {
      match output::create_output_dir(&self.out_dir_c1s3, true) {
        Ok(o) => o,
        Err(e) => {
          e.clone_into(&mut self.err_msg);
          self.modal_open = true;
          return;
        }
      }
    };

    // Write files.
    if self.pwd_grade_c1s3.is_some() {
      if let Err(e) = output::write_memory_dump_file(&target_password_bytes, &output_dir_str) {
        e.clone_into(&mut self.err_msg);
        self.modal_open = true;
        return;
      }
    }

    if let Some(password_version) = self.pwd_text_ver_c1s3 {
      if let Err(e) = output::write_password_text_file_with_bytes(
        &target_password_bytes,
        password_version,
        &output_dir_str,
      ) {
        e.clone_into(&mut self.err_msg);
        self.modal_open = true;
        return;
      }
    }

    if let Some(cheat_version) = self.pwd_cheat_ver_c1s3 {
      if let Err(e) =
        output::write_cheat_file(&target_password_bytes, cheat_version, &output_dir_str)
      {
        e.clone_into(&mut self.err_msg);
        self.modal_open = true;
        return;
      }
    }

    if self.to_export_pwd_data_text_c1s3 {
      if self.pwd_grade_c1s3.is_none() {
        if let Err(e) = output::write_game_data_text_file(
          &save::get_exported_data_for_dyrati_sheet_by_save_data(&save_data),
          &output_dir_str,
        ) {
          e.clone_into(&mut self.err_msg);
          self.modal_open = true;
          return;
        }
      } else if let Err(e) = output::write_game_data_text_file(
        &save::get_exported_data_for_dyrati_sheet_by_bytes(
          &target_password_bytes,
          self.pwd_grade_c1s3.unwrap(),
        ),
        &output_dir_str,
      ) {
        e.clone_into(&mut self.err_msg);
        self.modal_open = true;
        return;
      }
    }

    // More detailed output logs, to improve user experience.
    // Sh*t code, but at least it works.
    self.out_log_c1s3.clear();
    let _ = write!(self.out_log_c1s3, "输出目录：\n{output_dir_str}\n\n");
    self.out_log_c1s3.push_str("运行概况：\n");

    let mut result_index = 1;
    let source_password_grade_cn = self.dmp_src_grade.unwrap().get_name();
    let _ = writeln!(
      self.out_log_c1s3,
      "{result_index}. 输入的源文件为{source_password_grade_cn}级别的密码内存转储文件。"
    );

    let mut downgrade_str_option = None;
    if let Some(target_password_grade) = self.pwd_grade_c1s3 {
      result_index += 1;
      let target_password_grade_cn = target_password_grade.get_name();
      downgrade_str_option = Some(target_password_grade_cn);
      let _ = writeln!(
        self.out_log_c1s3,
        "{result_index}. 已生成降级为{target_password_grade_cn}级别的“密码内存转储.dmp”文件。"
      );
    }

    if let Some(password_version) = self.pwd_text_ver_c1s3 {
      result_index += 1;
      let password_version_cn = password_version.get_name();
      if let Some(target_password_grade_cn) = downgrade_str_option {
        let _ = writeln!(
          self.out_log_c1s3,
          "{result_index}. 已生成降级为{target_password_grade_cn}级别的{password_version_cn}版“密码文本.txt”文件。"
        );
      } else {
        let _ = writeln!(
          self.out_log_c1s3,
          "{result_index}. 已生成与源文件同级的{source_password_grade_cn}级别的{password_version_cn}版“密码文本.txt”文件。"
        );
      }
    }

    if let Some(cheat_version) = self.pwd_cheat_ver_c1s3 {
      result_index += 1;
      let cheat_version_cn = cheat_version.get_name();
      if let Some(target_password_grade_cn) = downgrade_str_option {
        let _ = writeln!(
          self.out_log_c1s3,
          "{result_index}. 已生成降级为{target_password_grade_cn}级别的、适用于{cheat_version_cn}版《黄金太阳 失落的时代》的“密码金手指.txt”文件。"
        );
      } else {
        let _ = writeln!(
          self.out_log_c1s3,
          "{result_index}. 已生成与源文件同级的{source_password_grade_cn}级别的、适用于{cheat_version_cn}版《黄金太阳 失落的时代》的“密码金手指.txt”文件。"
        );
      }
    }

    if self.to_export_pwd_data_text_c1s3 {
      result_index += 1;
      if let Some(target_password_grade_cn) = downgrade_str_option {
        let _ = writeln!(
          self.out_log_c1s3,
          "{result_index}. 已生成降级为{target_password_grade_cn}级别的“密码数据.txt”文件"
        );
      } else {
        let _ = writeln!(
          self.out_log_c1s3,
          "{result_index}. 已生成与源文件同级的{source_password_grade_cn}级别的“密码数据.txt”文件"
        );
      }
    }
  }
}

fn validate_input_file(path_str: &str) -> Result<(Vec<u8>, BitArray, PasswordGrade), &'static str> {
  // Read password memory dump file.
  let input_file_result = File::open(path_str);
  let mut input_file;
  if input_file_result.is_err() {
    return Err("打开密码的内存转储二进制文件时发生错误！");
  }
  input_file = input_file_result.unwrap();

  // Check its file size, and get its grade.
  let file_size = input_file.metadata().unwrap().len();
  let source_password_grade = match file_size {
    260 => PasswordGrade::Gold,
    61 => PasswordGrade::Silver,
    16 => PasswordGrade::Bronze,
    _ => {
      return Err("文件大小应为 16 字节，61 字节或 260 字节！");
    }
  };

  let mut password_bytes = Vec::new();
  input_file.read_to_end(&mut password_bytes).unwrap();
  // Check if it has contains invalid byte
  let has_invalid_byte = convert::contains_invalid_byte(password_bytes.as_slice());
  if has_invalid_byte {
    return Err("密码包含无效字节！");
  }

  // Check if the password is valid;
  let password_bits;
  if let Some(bits) = save::get_valid_password_bits_option(password_bytes.as_slice(), true) {
    password_bits = bits;
  } else {
    return Err("密码无效！");
  }

  Ok((password_bytes, password_bits, source_password_grade))
}
