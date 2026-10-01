use core::f32;
use std::{fmt::Write as _, fs::File, io::Read};

use arboard::Clipboard;
use chardetng::{EncodingDetector, Iso2022JpDetection::Deny, Utf8Detection::Allow};
use egui::{Checkbox, ComboBox, Grid, Label, RichText, Ui, Widget};
use rfd::FileDialog;

use crate::{
  App,
  app::SubCommandEnum,
  core::{
    convert,
    enums::{GameVersion, PasswordGrade, PasswordVersion},
    output,
    save::{self, BitArray},
  },
  widgets::buttons::{clipboard_btn, pwd_info_btn, reset_btn, select_dir_btn, select_file_btn},
};

const DESC_C1S2: &str = r"“密码级别”，“转换密码版本”，“生成内存转储文件”，“生成密码金手指”，“导出密码数据至文本文件”。
以上五项虽然都是可以留空或者不勾选的可选选项，但是必须要有其中一项有选择值或者已勾选。";

impl App {
  pub fn show_c1s2(&mut self, ui: &mut Ui) {
    Label::new(DESC_C1S2).selectable(false).ui(ui);

    ui.add_space(16.0);

    Grid::new("ID_GRID_C1S2")
      .num_columns(2)
      .spacing([20.0, 5.0])
      .striped(true)
      .show(ui, |ui| {
        // 第1行
        Label::new("输入文件路径").selectable(false).ui(ui).on_hover_text("《黄金太阳》的密码文本文件。");
        ui.horizontal(|ui| {
          if ui.add(clipboard_btn()).clicked() {
            let result = validate_clipboard_text();
            match result {
                Ok(o) => {
                    self.input_file_path_c1s2 = String::new();
                    self.txt_file = o.0;
                    self.txt_bytes = o.1;
                    self.txt_bits = Some(o.2);
                    self.txt_src_grade_version = Some((o.3, o.4));

                    self.pwd_grade_c1s2 = None;
                    self.to_convert_pwd_text_ver = false;
                    self.to_export_pwd_bin_c1s2 = false;
                    self.pwd_cheat_ver_c1s2 = None;
                    self.to_export_pwd_data_text_c1s2 = false;
                },
                Err(e) => {
                    e.clone_into(&mut self.err_msg);
                    self.err_modal_open = true;
                },
            }
          }

          if ui.add(select_file_btn()).clicked() {
            if let Some(file) = FileDialog::new().pick_file(){
              let new_file_path = file.to_string_lossy().into_owned();
              // 验证密码文本文件
              let result = validate_input_file(&new_file_path);
              match result {
                  Ok(o) => {
                    self.input_file_path_c1s2 = new_file_path;
                    self.txt_file = o.0;
                    self.txt_bytes = o.1;
                    self.txt_bits = Some(o.2);
                    self.txt_src_grade_version = Some((o.3, o.4));

                    self.pwd_grade_c1s2 = None;
                    self.to_convert_pwd_text_ver = false;
                    self.to_export_pwd_bin_c1s2 = false;
                    self.pwd_cheat_ver_c1s2 = None;
                    self.to_export_pwd_data_text_c1s2 = false;
                  },
                  Err(e) => {
                    e.clone_into(&mut self.err_msg);
                    self.err_modal_open = true;
                  },
              }
            }
          }
          
          if self.txt_src_grade_version.is_none() {
            ui.add_enabled(false, reset_btn());
          } else if ui.add(reset_btn()).clicked() {
            self.input_file_path_c1s2.clear();
            self.txt_file.clear();
            self.txt_bytes.clear();
            self.txt_bits = None;
            self.txt_src_grade_version = None;

            self.pwd_grade_c1s2 = None;
            self.to_convert_pwd_text_ver = false;
            self.to_export_pwd_bin_c1s2 = false;
            self.pwd_cheat_ver_c1s2 = None;
            self.to_export_pwd_data_text_c1s2 = false;

            for i in 0..3 {
              if self.pwd_src[i] == Some(SubCommandEnum::Second) {
                self.pwd_src[i] = None;
                self.pwd_src_slot[i] = None;
              }
            }
          }
          ui.add(
            egui::TextEdit::singleline(&mut self.input_file_path_c1s2)
              .interactive(false)
              .hint_text("密码文本文件路径")
              .desired_width(f32::INFINITY),
          );
        });
        ui.end_row();
        // 第2行
        if self.txt_src_grade_version.is_some() {
          Label::new("密码级别").selectable(false).ui(ui).on_hover_text("目标密码级别（仅用于降级），指定级别不可高于输入文件的级别。若两者级别相同则忽略；若可降级则生成降级后的密码文本文件（及相关文件）；若留空，则目标级别同输入文件。");
          ComboBox::from_id_salt("ID_CB_C1S2_密码级别")
            .width(100.0)
            .selected_text(match &self.pwd_grade_c1s2 {
              Some(password_grade_enum) => password_grade_enum.get_name(),
              None => "无",
            })
            .show_ui(ui, |ui| {
              ui.selectable_value(&mut self.pwd_grade_c1s2, None, "无");
              if let Some((source_grade, _)) = self.txt_src_grade_version {
                for ele in PasswordGrade::PASSWORD_GRADES {
                  ui.add_enabled_ui(source_grade.get_priority() < ele.get_priority(), |ui|{
                    ui.selectable_value(&mut self.pwd_grade_c1s2, Some(ele), ele.get_name());
                  });
                }
              }
            });
        } else {
          Label::new(RichText::new("密码级别").weak()).selectable(false).ui(ui).on_hover_text("目标密码级别（仅用于降级），指定级别不可高于输入文件的级别。若两者级别相同则忽略；若可降级则生成降级后的密码文本文件（及相关文件）；若留空，则目标级别同输入文件。");
          ui.add_enabled_ui(false, |ui|{
            ComboBox::from_id_salt("ID_CB_C1S2_密码级别（不可用）").width(100.0).selected_text("无").show_ui(ui, |_ui|{});
          });
        }
        ui.end_row();
        // 第3行
        if self.txt_src_grade_version.is_some(){
          Label::new("转换密码版本").selectable(false).ui(ui).on_hover_text("若勾选，则自动检测输入的密码文本的版本，并转换输出另外一个版本的密码文本文件，即日转英，英转日。");
          Checkbox::without_text(&mut self.to_convert_pwd_text_ver).ui(ui);
        } else {
          Label::new(RichText::new("转换密码版本").weak()).selectable(false).ui(ui).on_hover_text("若勾选，则自动检测输入的密码文本的版本，并转换输出另外一个版本的密码文本文件，即日转英，英转日。");
          ui.add_enabled(false, Checkbox::without_text(&mut self.to_convert_pwd_text_ver));
        }
        ui.end_row();
        // 第4行
        if self.txt_src_grade_version.is_some(){
          Label::new("生成内存转储文件").selectable(false).ui(ui).on_hover_text("若勾选，则生成密码的内存转储文件");
          Checkbox::without_text(&mut self.to_export_pwd_bin_c1s2).ui(ui);
        } else {
          Label::new(RichText::new("生成内存转储文件").weak()).selectable(false).ui(ui).on_hover_text("若勾选，则生成密码的内存转储文件");
          ui.add_enabled(false, Checkbox::without_text(&mut self.to_export_pwd_bin_c1s2));
        }
        ui.end_row();
        // 第5行
        if self.txt_src_grade_version.is_some(){
          Label::new("生成密码金手指").selectable(false).ui(ui).on_hover_text("若选择了值，则生成指定版本的金手指文本文件");
          ComboBox::from_id_salt("ID_CB_C1S2_生成密码金手指")
            .width(100.0)
            .selected_text(match &self.pwd_cheat_ver_c1s2 {
              Some(version_enum) => version_enum.get_name(),
              None => "无",
            })
            .show_ui(ui, |ui| {
              ui.selectable_value(&mut self.pwd_cheat_ver_c1s2, None, "无");
              for ele in GameVersion::GAME_VERSIONS {
                ui.selectable_value(&mut self.pwd_cheat_ver_c1s2, Some(ele), ele.get_name());
              }
            });
        } else {
          Label::new(RichText::new("生成密码金手指").weak()).selectable(false).ui(ui).on_hover_text("若选择了值，则生成指定版本的金手指文本文件");
          ui.add_enabled_ui(false, |ui|{
            ComboBox::from_id_salt("ID_CB_C1S2_生成密码金手指（不可用）").width(100.0).selected_text("无").show_ui(ui, |_ui|{});
          });
        }
        ui.end_row();
        // 第6行
        if self.txt_src_grade_version.is_some() {
          Label::new("导出密码数据至文本文件").selectable(false).ui(ui).on_hover_text("若勾选，则导出适用于 Dyrati 的“Golden Sun Password Generator”表格的密码数据文本文件");
          ui.horizontal(|ui|{
            Checkbox::without_text(&mut self.to_export_pwd_data_text_c1s2).ui(ui);
            if ui.add(pwd_info_btn()).clicked() {
              self.read_from(SubCommandEnum::Second);
            }
          });
        }else {
          Label::new(RichText::new("导出密码数据至文本文件").weak()).selectable(false).ui(ui).on_hover_text("若勾选，则导出适用于 Dyrati 的“Golden Sun Password Generator”表格的密码数据文本文件");
          ui.horizontal(|ui|{
            ui.add_enabled(false, Checkbox::without_text(&mut self.to_export_pwd_data_text_c1s2));
            ui.add_enabled(false, pwd_info_btn());
          });
        }
        ui.end_row();
        // 第7行
        Label::new("指定输出文件保存目录").selectable(false).ui(ui).on_hover_text("自定义输出文件夹（推荐留空），若留空则在输入文件的同目录下创建文件夹保存生成的所有文件。");
        ui.horizontal(|ui| {
          if ui.add(select_dir_btn()).clicked(){
            if let Some(dir) = FileDialog::new().pick_folder(){
              self.out_dir_c1s2 = dir.to_string_lossy().into_owned();
            }
          }
          if self.out_dir_c1s2.is_empty() {
            ui.add_enabled(false, reset_btn());
          } else if ui.add(reset_btn()).clicked(){
            self.out_dir_c1s2.clear();
          }
          ui.add(
            egui::TextEdit::singleline(&mut self.out_dir_c1s2)
              .interactive(false)
              .hint_text("若留空，则输出目录与输入的密码文本文件所在目录相同")
              .desired_width(f32::INFINITY),
          );
        });
        ui.end_row();
      });
  }

  pub fn run_c1s2(&mut self) {
    let mut password_bits = BitArray::from(&self.txt_bits.as_ref().unwrap().get_bits());
    let password_version = self.txt_src_grade_version.unwrap().1;

    // Generate save data, and set target password bytes
    let save_data = save::get_save_data_with_password_bits(
      &mut password_bits,
      self.txt_src_grade_version.unwrap().0,
    );
    let target_password_bytes = match self.pwd_grade_c1s2 {
      Some(password_grade) => save::get_password_bytes_with_save_data(&save_data, password_grade),
      None => self.txt_bytes.clone(),
    };

    // Create output directory.
    let output_dir_str = if self.out_dir_c1s2.is_empty() {
      match output::create_output_dir(&self.input_file_path_c1s2, false) {
        Ok(o) => o,
        Err(e) => {
          e.clone_into(&mut self.err_msg);
          self.err_modal_open = true;
          return;
        }
      }
    } else {
      match output::create_output_dir(&self.out_dir_c1s2, true) {
        Ok(o) => o,
        Err(e) => {
          e.clone_into(&mut self.err_msg);
          self.err_modal_open = true;
          return;
        }
      }
    };

    // Write files.
    if self.to_convert_pwd_text_ver {
      if self.pwd_grade_c1s2.is_some() {
        if let Err(e) = output::write_password_text_file_with_bytes(
          &target_password_bytes,
          password_version.rev(),
          &output_dir_str,
        ) {
          e.clone_into(&mut self.err_msg);
          self.err_modal_open = true;
          return;
        }
      } else if let Err(e) = output::write_converted_password_text_file(
        &convert::txt_to_another_version(&self.txt_file, password_version),
        password_version.rev(),
        &output_dir_str,
      ) {
        e.clone_into(&mut self.err_msg);
        self.err_modal_open = true;
        return;
      }
    } else if self.pwd_grade_c1s2.is_some() {
      if let Err(e) = output::write_password_text_file_with_bytes(
        &target_password_bytes,
        password_version,
        &output_dir_str,
      ) {
        e.clone_into(&mut self.err_msg);
        self.err_modal_open = true;
        return;
      }
    }

    if self.to_export_pwd_bin_c1s2 {
      if let Err(e) = output::write_memory_dump_file(&target_password_bytes, &output_dir_str) {
        e.clone_into(&mut self.err_msg);
        self.err_modal_open = true;
        return;
      }
    }

    if let Some(cheat_version) = self.pwd_cheat_ver_c1s2 {
      if let Err(e) =
        output::write_cheat_file(&target_password_bytes, cheat_version, &output_dir_str)
      {
        e.clone_into(&mut self.err_msg);
        self.err_modal_open = true;
        return;
      }
    }

    if self.to_export_pwd_data_text_c1s2 {
      if self.pwd_grade_c1s2.is_none() {
        if let Err(e) = output::write_game_data_text_file(
          &save::get_exported_data_for_dyrati_sheet_by_save_data(&save_data),
          &output_dir_str,
        ) {
          e.clone_into(&mut self.err_msg);
          self.err_modal_open = true;
          return;
        }
      } else if let Err(e) = output::write_game_data_text_file(
        &save::get_exported_data_for_dyrati_sheet_by_bytes(
          &target_password_bytes,
          self.pwd_grade_c1s2.unwrap(),
        ),
        &output_dir_str,
      ) {
        e.clone_into(&mut self.err_msg);
        self.err_modal_open = true;
        return;
      }
    }

    // More detailed output logs, to improve user experience.
    // Sh*t code, but at least it works.
    self.out_log_c1s2.clear();
    let _ = write!(self.out_log_c1s2, "输出目录：\n{output_dir_str}\n\n");
    self.out_log_c1s2.push_str("运行概况：\n");

    let mut result_index = 1;
    let source_password_grade_cn = self.txt_src_grade_version.unwrap().0.get_name();
    let source_password_version_cn = password_version.get_name();

    let _ = writeln!(
      self.out_log_c1s2,
      "{result_index}. 输入的源文件为{source_password_grade_cn}级别的{source_password_version_cn}版密码文本文件。"
    );

    let convert_flag_str_option = if self.to_convert_pwd_text_ver {
      Some(password_version.rev().get_name())
    } else {
      None
    };

    let mut downgrade_str_option = None;
    if let Some(target_password_grade) = self.pwd_grade_c1s2 {
      let target_password_grade_cn = target_password_grade.get_name();
      downgrade_str_option = Some(target_password_grade_cn);
    }

    match convert_flag_str_option {
      Some(convert_flag_str) => {
        if let Some(downgrade_str) = downgrade_str_option {
          result_index += 1;
          let _ = writeln!(
            self.out_log_c1s2,
            "{result_index}. 已生成降级为{downgrade_str}级别的、转换为{convert_flag_str}版的“密码文本.txt”文件。"
          );
        } else {
          result_index += 1;
          let _ = writeln!(
            self.out_log_c1s2,
            "{result_index}. 已生成与源文件同级的{source_password_grade_cn}级别的、转换为{convert_flag_str}版的“密码文本.txt”文件。"
          );
        }
      }
      None => {
        if let Some(downgrade_str) = downgrade_str_option {
          result_index += 1;
          let _ = writeln!(
            self.out_log_c1s2,
            "{result_index}. 已生成降级为{downgrade_str}级别的、未经版本转换的{source_password_version_cn}版“密码文本.txt”文件。"
          );
        }
      }
    }

    if self.to_export_pwd_bin_c1s2 {
      result_index += 1;
      if let Some(target_password_grade_cn) = downgrade_str_option {
        let _ = writeln!(
          self.out_log_c1s2,
          "{result_index}. 已生成降级为{target_password_grade_cn}级别的“密码内存转储.dmp”文件。"
        );
      } else {
        let _ = writeln!(
          self.out_log_c1s2,
          "{result_index}. 已生成与源文件同级的{source_password_grade_cn}级别的“密码内存转储.dmp”文件。"
        );
      }
    }

    if let Some(cheat_version) = self.pwd_cheat_ver_c1s2 {
      result_index += 1;
      let cheat_version_cn = cheat_version.get_name();
      if let Some(target_password_grade_cn) = downgrade_str_option {
        let _ = writeln!(
          self.out_log_c1s2,
          "{result_index}. 已生成降级为{target_password_grade_cn}级别的、适用于{cheat_version_cn}版《黄金太阳 失落的时代》的“密码金手指.txt”文件。"
        );
      } else {
        let _ = writeln!(
          self.out_log_c1s2,
          "{result_index}. 已生成与源文件同级的{source_password_grade_cn}级别的、适用于{cheat_version_cn}版《黄金太阳 失落的时代》的“密码金手指.txt”文件。"
        );
      }
    }

    if self.to_export_pwd_data_text_c1s2 {
      result_index += 1;
      if let Some(target_password_grade_cn) = downgrade_str_option {
        let _ = writeln!(
          self.out_log_c1s2,
          "{result_index}. 已生成降级为{target_password_grade_cn}级别的“密码数据.txt”文件"
        );
      } else {
        let _ = writeln!(
          self.out_log_c1s2,
          "{result_index}. 已生成与源文件同级的{source_password_grade_cn}级别的“密码数据.txt”文件"
        );
      }
    }
  }
}

fn validate_input_file(
  path_str: &str,
) -> Result<(String, Vec<u8>, BitArray, PasswordGrade, PasswordVersion), &'static str> {
  // Read password text file.
  let input_file_result = File::open(path_str);
  let mut input_file;
  if input_file_result.is_err() {
    return Err("打开密码文本文件时发生错误！");
  }
  input_file = input_file_result.unwrap();

  let file_size = input_file.metadata().unwrap().len();
  if file_size == 0 {
    return Err("文本文件内没有任何内容！");
  } else if file_size > 0x10_0000 {
    return Err("文本文件的大小过大！");
  }

  let mut file_bytes = Vec::new();
  input_file.read_to_end(&mut file_bytes).unwrap();

  let mut detector = EncodingDetector::new(Deny);
  detector.feed(&file_bytes, true);
  let encoding = detector.guess(None, Allow);

  let (cow_str, had_errors) = encoding.decode_with_bom_removal(&file_bytes);
  if had_errors {
    return Err("输入的不是有效的文本文件！");
  }

  let password = cow_str.into_owned();
  validate_password_string(&password)
}

fn validate_clipboard_text()
-> Result<(String, Vec<u8>, BitArray, PasswordGrade, PasswordVersion), &'static str> {
  let clipboard_result = Clipboard::new();
  if clipboard_result.is_err() {
    return Err("访问剪贴板时出现错误！");
  }
  let mut clipboard = clipboard_result.unwrap();

  let clipboard_text_result = clipboard.get_text();
  if clipboard_text_result.is_err() {
    return Err("获取剪贴板的文本时出现错误！");
  }

  let password = clipboard_text_result.unwrap();
  validate_password_string(&password)
}

fn validate_password_string(
  password: &str,
) -> Result<(String, Vec<u8>, BitArray, PasswordGrade, PasswordVersion), &'static str> {
  // If the text file is empty...
  if password.is_empty() {
    return Err("文本中没有任何内容！");
  }

  // Remove all line break and whitespace.
  let password_without_whitespace = convert::remove_whitespace(&password);
  // Check password's length, and get its grade.
  let chars_count = password_without_whitespace.chars().count();
  let source_password_grade = match chars_count {
    260 => PasswordGrade::Gold,
    61 => PasswordGrade::Silver,
    16 => PasswordGrade::Bronze,
    0 => return Err("文本中没有任何内容！"),
    _ => return Err("密码长度不正确！"),
  };
  // Check its password version.
  let password_version = convert::get_password_version(&password_without_whitespace);
  // Check if it contains invalid char
  let has_invalid_char = match password_version {
    PasswordVersion::Japanese => convert::contains_invalid_char_jp(&password_without_whitespace),
    PasswordVersion::English => convert::contains_invalid_char_en(&password_without_whitespace),
  };
  if has_invalid_char {
    return Err("密码文本包含无效字符！");
  }

  // Check if the password is valid.
  let password_bytes = convert::txt_to_dmp(&password_without_whitespace, password_version);
  let password_bits;
  if let Some(bits) = save::get_valid_password_bits_option(&password_bytes, true) {
    password_bits = bits;
  } else {
    return Err("密码无效！");
  }

  Ok((
    password_without_whitespace,
    password_bytes,
    password_bits,
    source_password_grade,
    password_version,
  ))
}
