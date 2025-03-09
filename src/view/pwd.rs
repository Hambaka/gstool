use core::f32;
use std::{collections::HashMap, fs::File, io::Read, path::Path};

use egui::{Checkbox, ComboBox, Grid, Label, Ui, Widget};
use rfd::FileDialog;

use crate::{
  App,
  app::CommandEnum,
  core::{
    convert,
    enums::{GameVersion, PasswordGrade, PasswordVersion},
    output,
    save::{self, BitArray, RawSaveData},
  },
};

impl App {
  pub fn show_pwd(&mut self, ui: &mut Ui) {
    Label::new("使用该功能可导出密码，可导出以下几类文件：\n\
        （1）密码的文本文件（支持导出日文版/英文版密码）\n\
        （2）密码的内存转储文件\n\
        （3）密码的金手指文本文件，可以在《黄金太阳 失落的时代》中使用（支持日/美/德/西/法/意版本）\n\
        （4）密码数据的文本文件，可结合 Dyrati 的“Golden Sun Password Generator”表格使用\n\n\
        注：Mobile&Elffinal/SOMA 的《黄金太阳 失落的时代》汉化版基于美版制作，因此金手指版本选择美版即可。"
    ).selectable(false).ui(ui);

    ui.separator();

    ui.columns(3, |columns| {
      columns[0].selectable_value(
        &mut self.subcommand,
        CommandEnum::First,
        "（1.1）使用存档文件",
      );
      columns[1].selectable_value(
        &mut self.subcommand,
        CommandEnum::Second,
        "（1.2）使用密码文本文件",
      );
      columns[2].selectable_value(
        &mut self.subcommand,
        CommandEnum::Third,
        "（1.3）使用内存转储文件",
      );
    });

    match self.subcommand {
      CommandEnum::First => self.show_sub_one(ui),
      CommandEnum::Second => self.show_sub_two(ui),
      CommandEnum::Third => self.show_sub_three(ui),
    }
  }

  fn show_sub_one(&mut self, ui: &mut Ui) {
    Label::new(
      "“生成内存转储文件”，“生成密码金手指”，“导出密码数据至文本文件”。\n\
      程序默认密码级别为“金”，密码文本版本为“英文”，因此以上三项都是可以留空或者不勾选的可选选项。",
    )
    .selectable(false)
    .ui(ui);

    Grid::new("(1.1)使用存档文件")
      .num_columns(2)
      .spacing([20.0, 5.0])
      .striped(true)
      .show(ui, |ui| {
        // 第1行
        Label::new("导出所有存档").selectable(false).ui(ui).on_hover_text("若不勾选，则仅导出通关存档的相关文件。");
        Checkbox::without_text(&mut self.to_export_all_save).ui(ui);
        ui.end_row();
        // 第2行
        Label::new("输入文件路径").selectable(false).ui(ui).on_hover_text("《黄金太阳 开启的封印》的存档文件。");
        ui.horizontal(|ui| {
          if ui.button("选择文件……").clicked(){
            if let Some(file) = FileDialog::new().pick_file(){
              self.to_export_all_save= false;
              self.save_data_map = None;

              self.pwd_grade_c1s1= Some(PasswordGrade::Gold);
              self.pwd_text_ver_c1s1= Some(PasswordVersion::English);
              self.to_export_pwd_bin_c1s1= false;
              self.pwd_cheat_ver_c1s1= None;
              self.to_export_pwd_data_text_c1s1= false;

              self.input_file_path_c1s1 = file.to_string_lossy().into_owned();
              // 验证存档文件
              let result = validate_save_pwd(&self.input_file_path_c1s1, self.to_export_all_save);
              match result {
                  Ok(o) => {
                    self.save_data_map = o;
                  },
                  Err(e) => {
                    e.clone_into(&mut self.err_msg);
                    self.modal_open = true;
                  },
              }
            }
          };
          if ui.button("清空").clicked(){
            self.to_export_all_save= false;
            self.input_file_path_c1s1.clear();
            self.save_data_map = None;

            self.pwd_grade_c1s1= Some(PasswordGrade::Gold);
            self.pwd_text_ver_c1s1= Some(PasswordVersion::English);
            self.to_export_pwd_bin_c1s1= false;
            self.pwd_cheat_ver_c1s1= None;
            self.to_export_pwd_data_text_c1s1= false;
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
        Label::new("密码级别").selectable(false).ui(ui).on_hover_text("目标密码级别，若留空则自动指定默认值“金”");
        ComboBox::from_id_salt("（1.1）密码级别")
          .width(100.0)
          .selected_text(match &self.pwd_grade_c1s1 {
            Some(password_grade_enum) => password_grade_enum.get_name(),
            None => "无",
          })
          .show_ui(ui, |ui| {
            if ui.selectable_value(&mut self.pwd_grade_c1s1, None, "无").changed(){
              self.pwd_text_ver_c1s1= None;
              self.to_export_pwd_bin_c1s1= false;
              self.pwd_cheat_ver_c1s1= None;
              self.to_export_pwd_data_text_c1s1= false;
            };
            ui.selectable_value(
              &mut self.pwd_grade_c1s1,
              Some(PasswordGrade::Gold),
              PasswordGrade::Gold.get_name(),
            );
            ui.selectable_value(
              &mut self.pwd_grade_c1s1,
              Some(PasswordGrade::Silver),
              PasswordGrade::Silver.get_name(),
            );
            ui.selectable_value(
              &mut self.pwd_grade_c1s1,
              Some(PasswordGrade::Bronze),
              PasswordGrade::Bronze.get_name(),
            );
          });
        ui.end_row();
        // 第4行
        Label::new("密码文本版本").selectable(false).ui(ui).on_hover_text("生成指定版本的密码文本文件，若留空则自动指定默认值“英文”。");
        ComboBox::from_id_salt("（1.1）密码文本版本")
          .width(100.0)
          .selected_text(match &self.pwd_text_ver_c1s1 {
            Some(password_version_enum) => password_version_enum.get_name(),
            None => "无",
          })
          .show_ui(ui, |ui| {
            ui.selectable_value(&mut self.pwd_text_ver_c1s1, None, "无");
            if self.save_data_map.is_some() && self.pwd_grade_c1s1.is_some(){
              ui.selectable_value(
                &mut self.pwd_text_ver_c1s1,
                Some(PasswordVersion::Japanese),
                PasswordVersion::Japanese.get_name(),
              );
              ui.selectable_value(
                &mut self.pwd_text_ver_c1s1,
                Some(PasswordVersion::English),
                PasswordVersion::English.get_name(),
              );
            }
          });
        ui.end_row();
        // 第5行
        Label::new("生成内存转储文件").selectable(false).ui(ui).on_hover_text("若勾选，则生成密码的内存转储文件。");
        if self.save_data_map.is_some() && self.pwd_grade_c1s1.is_some(){
          Checkbox::without_text(&mut self.to_export_pwd_bin_c1s1).ui(ui);
        } else {
          ui.add_enabled(false, Checkbox::without_text(&mut self.to_export_pwd_bin_c1s1));
        }
        ui.end_row();
        // 第6行
        Label::new("生成密码金手指").selectable(false).ui(ui).on_hover_text("若选择了值，则生成指定版本的金手指文本文件。");
        ComboBox::from_id_salt("（1.1）生成密码金手指")
          .width(100.0)
          .selected_text(match &self.pwd_cheat_ver_c1s1 {
            Some(version_enum) => version_enum.get_name(),
            None => "无",
          })
          .show_ui(ui, |ui| {
            ui.selectable_value(&mut self.pwd_cheat_ver_c1s1, None, "无");
            if self.save_data_map.is_some() && self.pwd_grade_c1s1.is_some(){
              ui.selectable_value(
                &mut self.pwd_cheat_ver_c1s1,
                Some(GameVersion::Japan),
                GameVersion::Japan.get_name(),
              );
              ui.selectable_value(
                &mut self.pwd_cheat_ver_c1s1,
                Some(GameVersion::USA),
                GameVersion::USA.get_name(),
              );
              ui.selectable_value(
                &mut self.pwd_cheat_ver_c1s1,
                Some(GameVersion::Germany),
                GameVersion::Germany.get_name(),
              );
              ui.selectable_value(
                &mut self.pwd_cheat_ver_c1s1,
                Some(GameVersion::Spain),
                GameVersion::Spain.get_name(),
              );
              ui.selectable_value(
                &mut self.pwd_cheat_ver_c1s1,
                Some(GameVersion::France),
                GameVersion::France.get_name(),
              );
              ui.selectable_value(
                &mut self.pwd_cheat_ver_c1s1,
                Some(GameVersion::Italy),
                GameVersion::Italy.get_name(),
              );
            }
          });
        ui.end_row();
        // 第7行
        Label::new("导出密码数据至文本文件").selectable(false).ui(ui).on_hover_text("若勾选，则导出适用于 Dyrati 的“Golden Sun Password Generator”表格的密码数据文本文件。");
        if self.save_data_map.is_some() && self.pwd_grade_c1s1.is_some(){
          Checkbox::without_text(&mut self.to_export_pwd_data_text_c1s1).ui(ui);
        }else {
            ui.add_enabled(false, Checkbox::without_text(&mut self.to_export_pwd_data_text_c1s1));
        }
        ui.end_row();
        // 第8行
        Label::new("指定输出文件保存目录").selectable(false).ui(ui).on_hover_text("自定义输出文件夹（推荐留空），若留空则在输入文件的同目录下创建文件夹保存生成的所有文件。");
        ui.horizontal(|ui| {
          if ui.button("选择目录……").clicked(){
            if let Some(dir) = FileDialog::new().pick_folder(){
              self.out_dir_c1s1 = dir.to_string_lossy().into_owned();
            }
          };
          if ui.button("清空").clicked(){
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

  fn show_sub_two(&mut self, ui: &mut Ui) {
    Label::new("“密码级别”，“转换密码版本”，“生成内存转储文件”，“生成密码金手指”，“导出密码数据至文本文件”。\n\
      以上五项虽然都是可以留空或者不勾选的可选选项，但是必须要有其中一项有选择值或者已勾选。"
    )
    .selectable(false)
    .ui(ui);

    Grid::new("(1.2)使用密码文本文件")
      .num_columns(2)
      .spacing([20.0, 5.0])
      .striped(true)
      .show(ui, |ui| {
        // 第1行
        Label::new("输入文件路径").selectable(false).ui(ui).on_hover_text("黄金太阳的密码文本文件。");
        ui.horizontal(|ui| {
          if ui.button("选择文件……").clicked(){
            if let Some(file) = FileDialog::new().pick_file(){
              self.txt_file.clear();
              self.txt_bytes.clear();
              self.txt_bits = None;

              self.pwd_grade_c1s2 = None;
              self.to_convert_pwd_text_ver = false;
              self.to_export_pwd_bin_c1s2 = false;
              self.pwd_cheat_ver_c1s2 = None;
              self.to_export_pwd_data_text_c1s2 = false;

              self.input_file_path_c1s2 = file.to_string_lossy().into_owned();
              // 验证密码文本文件
              let result = validate_txt(&self.input_file_path_c1s2);
              match result {
                  Ok(o) => {
                    self.txt_file = o.0;
                    self.txt_bytes = o.1;
                    self.txt_bits = Some(o.2);
                    self.for_run_c1s2 = Some((o.3, o.4));
                  },
                  Err(e) => {
                    self.for_run_c1s2 = None;
                    e.clone_into(&mut self.err_msg);
                    self.modal_open = true;
                  },
              }
            }
          };
          if ui.button("清空").clicked(){
            self.input_file_path_c1s2.clear();
            self.txt_file.clear();
            self.txt_bytes.clear();
            self.txt_bits = None;

            self.pwd_grade_c1s2 = None;
            self.to_convert_pwd_text_ver = false;
            self.to_export_pwd_bin_c1s2 = false;
            self.pwd_cheat_ver_c1s2 = None;
            self.to_export_pwd_data_text_c1s2 = false;
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
        Label::new("密码级别").selectable(false).ui(ui).on_hover_text("目标密码级别（仅用于降级），指定级别不可高于输入文件的级别。若两者级别相同则忽略；若可降级则生成降级后的密码文本文件（及相关文件）；若留空，则目标级别同输入文件。");
        ComboBox::from_id_salt("（1.2）密码级别")
          .width(100.0)
          .selected_text(match &self.pwd_grade_c1s2 {
            Some(password_grade_enum) => password_grade_enum.get_name(),
            None => "无",
          })
          .show_ui(ui, |ui| {
            ui.selectable_value(&mut self.pwd_grade_c1s2, None, "无");
            if let Some((source_grade,_)) = self.for_run_c1s2{
              if source_grade.is_gold(){
                ui.selectable_value(
                  &mut self.pwd_grade_c1s2,
                  Some(PasswordGrade::Silver),
                  PasswordGrade::Silver.get_name(),
                );
              }
              if source_grade.is_gold() || source_grade.is_silver(){
                ui.selectable_value(
                  &mut self.pwd_grade_c1s2,
                  Some(PasswordGrade::Bronze),
                  PasswordGrade::Bronze.get_name(),
                );
              }
            }
          });
        ui.end_row();
        // 第3行
        Label::new("转换密码版本").selectable(false).ui(ui).on_hover_text("若勾选，则自动检测输入的密码文本的版本，并转换输出另外一个版本的密码文本文件，即日转英，英转日。");
        if self.for_run_c1s2.is_some(){
          Checkbox::without_text(&mut self.to_convert_pwd_text_ver).ui(ui);
        } else {
          ui.add_enabled(false, Checkbox::without_text(&mut self.to_convert_pwd_text_ver));
        }
        ui.end_row();
        // 第4行
        Label::new("生成内存转储文件").selectable(false).ui(ui).on_hover_text("若勾选，则生成密码的内存转储文件");
        if self.for_run_c1s2.is_some(){
          Checkbox::without_text(&mut self.to_export_pwd_bin_c1s2).ui(ui);
        } else {
            ui.add_enabled(false, Checkbox::without_text(&mut self.to_export_pwd_bin_c1s2));
        }
        ui.end_row();
        // 第5行
        Label::new("生成密码金手指").selectable(false).ui(ui).on_hover_text("若选择了值，则生成指定版本的金手指文本文件");
        ComboBox::from_id_salt("（1.2）生成密码金手指")
          .width(100.0)
          .selected_text(match &self.pwd_cheat_ver_c1s2 {
            Some(version_enum) => version_enum.get_name(),
            None => "无",
          })
          .show_ui(ui, |ui| {
            ui.selectable_value(&mut self.pwd_cheat_ver_c1s2, None, "无");
            if self.for_run_c1s2.is_some(){
              ui.selectable_value(
                &mut self.pwd_cheat_ver_c1s2,
                Some(GameVersion::Japan),
                GameVersion::Japan.get_name(),
              );
              ui.selectable_value(
                &mut self.pwd_cheat_ver_c1s2,
                Some(GameVersion::USA),
                GameVersion::USA.get_name(),
              );
              ui.selectable_value(
                &mut self.pwd_cheat_ver_c1s2,
                Some(GameVersion::Germany),
                GameVersion::Germany.get_name(),
              );
              ui.selectable_value(
                &mut self.pwd_cheat_ver_c1s2,
                Some(GameVersion::Spain),
                GameVersion::Spain.get_name(),
              );
              ui.selectable_value(
                &mut self.pwd_cheat_ver_c1s2,
                Some(GameVersion::France),
                GameVersion::France.get_name(),
              );
              ui.selectable_value(
                &mut self.pwd_cheat_ver_c1s2,
                Some(GameVersion::Italy),
                GameVersion::Italy.get_name(),
              );
            }
          });
        ui.end_row();
        // 第6行
        Label::new("导出密码数据至文本文件").selectable(false).ui(ui).on_hover_text("若勾选，则导出适用于 Dyrati 的“Golden Sun Password Generator”表格的密码数据文本文件");
        if self.for_run_c1s2.is_some(){
          Checkbox::without_text(&mut self.to_export_pwd_data_text_c1s2).ui(ui);
        }else {
            ui.add_enabled(false, Checkbox::without_text(&mut self.to_export_pwd_data_text_c1s2));
        }
        ui.checkbox(&mut self.to_export_pwd_data_text_c1s2, "");
        ui.end_row();
        // 第7行
        Label::new("指定输出文件保存目录").selectable(false).ui(ui).on_hover_text("自定义输出文件夹（推荐留空），若留空则在输入文件的同目录下创建文件夹保存生成的所有文件。");
        ui.horizontal(|ui| {
          if ui.button("选择目录……").clicked(){
            if let Some(dir) = FileDialog::new().pick_folder(){
              self.out_dir_c1s2 = dir.to_string_lossy().into_owned();
            }
          };
          if ui.button("清空").clicked(){
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

  fn show_sub_three(&mut self, ui: &mut Ui) {
    Label::new(
      "“密码级别”，“密码文本版本”，“生成密码金手指”，“导出密码数据至文本文件”。\n\
      以上四项虽然都是可以留空或者不勾选的可选选项，但是必须要有其中一项有选择值或者已勾选。",
    )
    .selectable(false)
    .ui(ui);

    Grid::new("（1.3）使用内存转储文件")
      .num_columns(2)
      .spacing([20.0, 5.0])
      .striped(true)
      .show(ui, |ui| {
        // 第1行
        Label::new("输入文件路径").selectable(false).ui(ui).on_hover_text("黄金太阳的密码内存转储文件");
        ui.horizontal(|ui| {
          if ui.button("选择文件……").clicked(){
            if let Some(file) = FileDialog::new().pick_file(){
              self.dmp_bytes.clear();
              self.dmp_bits = None;

              self.pwd_grade_c1s3 = None;
              self.pwd_text_ver_c1s3 = None;
              self.pwd_cheat_ver_c1s3 = None;
              self.to_export_data_text_3 = false;

              self.input_file_path_c1s3 = file.to_string_lossy().into_owned();
              // 验证内存转储文件
              let result = validate_dmp(&self.input_file_path_c1s3);
              match result {
                  Ok(o) => {
                    self.dmp_bytes = o.0;
                    self.dmp_bits = Some(o.1);
                    self.for_run_c1s3 = Some(o.2);
                  },
                  Err(e) => {
                    self.for_run_c1s3 = None;
                    e.clone_into(&mut self.err_msg);
                    self.modal_open = true;
                  },
              }
            }
          };
          if ui.button("清空").clicked(){
            self.input_file_path_c1s3.clear();
            self.dmp_bits = None;
            self.for_run_c1s3 = None;

            self.pwd_grade_c1s3 = None;
            self.pwd_text_ver_c1s3 = None;
            self.pwd_cheat_ver_c1s3 = None;
            self.to_export_data_text_3 = false;
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
        Label::new("密码级别").selectable(false).ui(ui).on_hover_text("目标密码级别（仅用于降级），指定级别不可高于输入文件的级别。若两者级别相同则忽略；若可降级则生成降级后的密码文本文件（及相关文件）；若留空，则目标级别同输入文件。");
        ComboBox::from_id_salt("（1.3）密码级别")
          .width(100.0)
          .selected_text(match &self.pwd_grade_c1s3 {
            Some(password_grade_enum) => password_grade_enum.get_name(),
            None => "无",
          })
          .show_ui(ui, |ui| {
            ui.selectable_value(&mut self.pwd_grade_c1s3, None, "无");
            if let Some(source_grade) = self.for_run_c1s3{
              if source_grade.is_gold(){
                ui.selectable_value(
                  &mut self.pwd_grade_c1s3,
                  Some(PasswordGrade::Silver),
                  PasswordGrade::Silver.get_name(),
                );
              }
              if source_grade.is_gold() || source_grade.is_silver(){
                ui.selectable_value(
                  &mut self.pwd_grade_c1s3,
                  Some(PasswordGrade::Bronze),
                  PasswordGrade::Bronze.get_name(),
                );
              }
            }
          });
        ui.end_row();
        // 第3行
        Label::new("密码文本版本").selectable(false).ui(ui).on_hover_text("若选择了值，则生成指定版本的密码文本文件。");
        ComboBox::from_id_salt("（1.3）密码文本版本")
          .width(100.0)
          .selected_text(match &self.pwd_text_ver_c1s3 {
            Some(password_version_enum) => password_version_enum.get_name(),
            None => "无",
          })
          .show_ui(ui, |ui| {
            ui.selectable_value(&mut self.pwd_text_ver_c1s3, None, "无");
            if self.for_run_c1s3.is_some(){
              ui.selectable_value(
                &mut self.pwd_text_ver_c1s3,
                Some(PasswordVersion::Japanese),
                PasswordVersion::Japanese.get_name(),
              );
              ui.selectable_value(
                &mut self.pwd_text_ver_c1s3,
                Some(PasswordVersion::English),
                PasswordVersion::English.get_name(),
              );
            }
          });
        ui.end_row();
        // 第4行
        Label::new("生成密码金手指").selectable(false).ui(ui).on_hover_text("若选择了值，则生成指定版本的金手指文本文件。");
        ComboBox::from_id_salt("（1.3）生成密码金手指")
          .width(100.0)
          .selected_text(match &self.pwd_cheat_ver_c1s3 {
            Some(version_enum) => version_enum.get_name(),
            None => "无",
          })
          .show_ui(ui, |ui| {
            ui.selectable_value(&mut self.pwd_cheat_ver_c1s3, None, "无");
            if self.for_run_c1s3.is_some(){
              ui.selectable_value(
                &mut self.pwd_cheat_ver_c1s3,
                Some(GameVersion::Japan),
                GameVersion::Japan.get_name(),
              );
              ui.selectable_value(
                &mut self.pwd_cheat_ver_c1s3,
                Some(GameVersion::USA),
                GameVersion::USA.get_name(),
              );
              ui.selectable_value(
                &mut self.pwd_cheat_ver_c1s3,
                Some(GameVersion::Germany),
                GameVersion::Germany.get_name(),
              );
              ui.selectable_value(
                &mut self.pwd_cheat_ver_c1s3,
                Some(GameVersion::Spain),
                GameVersion::Spain.get_name(),
              );
              ui.selectable_value(
                &mut self.pwd_cheat_ver_c1s3,
                Some(GameVersion::France),
                GameVersion::France.get_name(),
              );
              ui.selectable_value(
                &mut self.pwd_cheat_ver_c1s3,
                Some(GameVersion::Italy),
                GameVersion::Italy.get_name(),
              );
            }
          });
        ui.end_row();
        // 第5行
        Label::new("导出密码数据至文本文件").selectable(false).ui(ui).on_hover_text("若勾选，则导出适用于 Dyrati 的“Golden Sun Password Generator”表格的密码数据文本文件。");
        if self.for_run_c1s3.is_some(){
          Checkbox::without_text(&mut self.to_export_data_text_3).ui(ui);
        } else {
          ui.add_enabled(false, Checkbox::without_text(&mut self.to_export_data_text_3));
        }
        ui.end_row();
        // 第6行
        Label::new("指定输出文件保存目录").selectable(false).ui(ui).on_hover_text("自定义输出文件夹（推荐留空），若留空则在输入文件的同目录下创建文件夹保存生成的所有文件。");
        ui.horizontal(|ui| {
          if ui.button("选择目录……").clicked(){
            if let Some(dir) = FileDialog::new().pick_folder(){
              self.out_dir_c1s3 = dir.to_string_lossy().into_owned();
            }
          };
          if ui.button("清空").clicked(){
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

  pub fn run_pwd(&mut self) {
    match self.subcommand {
      CommandEnum::First => self.run_sub_one(),
      CommandEnum::Second => self.run_sub_two(),
      CommandEnum::Third => self.run_sub_three(),
    }
  }

  fn run_sub_one(&mut self) {
    // Create main output directory.
    let output_dir_str = if self.out_dir_c1s1.is_empty() {
      match output::create_output_dir(Path::new(&self.input_file_path_c1s1), false) {
        Ok(o) => o,
        Err(e) => {
          self.err_msg = e.to_owned();
          self.modal_open = true;
          return;
        }
      }
    } else {
      match output::create_output_dir(Path::new(&self.out_dir_c1s1), true) {
        Ok(o) => o,
        Err(e) => {
          self.err_msg = e.to_owned();
          self.modal_open = true;
          return;
        }
      }
    };

    for (key, val) in self.save_data_map.as_ref().unwrap() {
      let password_bytes =
        save::get_password_bytes_with_raw_save_bytes(val.get_data(), self.pwd_grade_c1s1.unwrap());
      // Key is save slot number: 0, 1, 2 -> 1, 2, 3
      let sub_dir_str =
        match output::create_sav_sub_dir(*key + 1, val.get_is_clear(), &output_dir_str) {
          Ok(o) => o,
          Err(e) => {
            e.clone_into(&mut self.err_msg);
            self.modal_open = true;
            return;
          }
        };

      if let Err(e) = output::write_password_text_file_with_bytes(
        &password_bytes,
        self.pwd_text_ver_c1s1.unwrap(),
        &sub_dir_str,
      ) {
        e.clone_into(&mut self.err_msg);
        self.modal_open = true;
        return;
      }

      if self.to_export_pwd_bin_c1s1 {
        if let Err(e) = output::write_memory_dump_file(&password_bytes, &sub_dir_str) {
          e.clone_into(&mut self.err_msg);
          self.modal_open = true;
          return;
        }
      }

      if let Some(cheat_version) = self.pwd_cheat_ver_c1s1 {
        if let Err(e) = output::write_cheat_file(&password_bytes, cheat_version, &sub_dir_str) {
          e.clone_into(&mut self.err_msg);
          self.modal_open = true;
          return;
        }
      }

      if self.to_export_pwd_data_text_c1s1 {
        if self.pwd_grade_c1s1.unwrap().is_gold() {
          if let Err(e) = output::write_game_data_text_file(
            &save::get_exported_data_for_dyrati_sheet_with_raw_save_bytes(val.get_data()),
            &sub_dir_str,
          ) {
            e.clone_into(&mut self.err_msg);
            self.modal_open = true;
            return;
          }
        } else if let Err(e) = output::write_game_data_text_file(
          &save::get_exported_data_for_dyrati_sheet_by_bytes(
            &password_bytes,
            self.pwd_grade_c1s1.unwrap(),
          ),
          &sub_dir_str,
        ) {
          e.clone_into(&mut self.err_msg);
          self.modal_open = true;
          return;
        }
      }

      // More detailed output logs, to improve user experience.
      // Sh*t code, but at least it works.
      self.out_log_c1s1.clear();
      self
        .out_log_c1s1
        .push_str(&format!("输出目录：\n{output_dir_str}\n\n"));
      self.out_log_c1s1.push_str("运行概况：\n");

      let mut result_index = 1;
      if self.to_export_all_save {
        self.out_log_c1s1.push_str(&format!(
          "{result_index}. 已勾选“导出所有存档”，导出包含未通关存档在内的所有存档的数据。\n"
        ));
      } else {
        self.out_log_c1s1.push_str(&format!(
          "{result_index}. 未勾选“导出所有存档”，仅导出通关存档的数据。\n"
        ));
      }

      result_index += 1;
      let password_grade_cn = self.pwd_grade_c1s1.unwrap().get_name();

      if let Some(password_version) = self.pwd_text_ver_c1s1 {
        let password_version_cn = password_version.get_name();
        if self.to_export_all_save {
          self.out_log_c1s1.push_str(&format!("{result_index}. 已生成存档记录所对应的{password_grade_cn}级别、{password_version_cn}版的“密码文本.txt”文件。\n"));
        } else {
          self.out_log_c1s1.push_str(&format!("{result_index}. 已生成通关的存档记录所对应的{password_grade_cn}级别、{password_version_cn}版的“密码文本.txt”文件。\n"));
        }
      }

      if self.to_export_pwd_bin_c1s1 {
        result_index += 1;
        if self.to_export_all_save {
          self.out_log_c1s1.push_str(&format!("{result_index}. 已生成存档记录所对应的{password_grade_cn}级别“密码内存转储.dmp”文件。\n"));
        } else {
          self.out_log_c1s1.push_str(&format!("{result_index}. 已生成通关的存档记录所对应的{password_grade_cn}级别“密码内存转储.dmp”文件。\n"));
        }
      }

      if let Some(cheat_version) = self.pwd_cheat_ver_c1s1 {
        result_index += 1;
        let cheat_version_cn = cheat_version.get_name();
        if self.to_export_all_save {
          self.out_log_c1s1.push_str(&format!("{result_index}. 已生成存档记录所对应的{password_grade_cn}级别的、适用于{cheat_version_cn}版《黄金太阳 失落的时代》的“密码金手指.txt”文件。\n"));
        } else {
          self.out_log_c1s1.push_str(&format!("{result_index}. 已生成通关的存档记录所对应的{password_grade_cn}级别的、适用于{cheat_version_cn}版《黄金太阳 失落的时代》的“密码金手指.txt”文件。\n"));
        }
      }

      if self.to_export_pwd_data_text_c1s1 {
        result_index += 1;
        if self.to_export_all_save {
          self.out_log_c1s1.push_str(&format!(
            "{result_index}. 已生成存档记录所对应的{password_grade_cn}级别“密码数据.txt”文件\n"
          ));
        } else {
          self.out_log_c1s1.push_str(&format!("{result_index}. 已生成通关的存档记录所对应的{password_grade_cn}级别“密码数据.txt”文件\n"));
        }
      }
    }
  }

  fn run_sub_two(&mut self) {
    let mut password_bits = BitArray::from(&self.txt_bits.as_ref().unwrap().get_bits());
    let password_version = self.for_run_c1s2.unwrap().1;

    // Generate save data, and set target password bytes
    let save_data =
      save::get_save_data_with_password_bits(&mut password_bits, self.for_run_c1s2.unwrap().0);
    let target_password_bytes = match self.pwd_grade_c1s2 {
      Some(password_grade) => save::get_password_bytes_with_save_data(&save_data, password_grade),
      None => self.txt_bytes.clone(),
    };

    // Create output directory.
    let output_dir_str = if self.out_dir_c1s2.is_empty() {
      match output::create_output_dir(Path::new(&self.input_file_path_c1s2), false) {
        Ok(o) => o,
        Err(e) => {
          self.err_msg = e.to_owned();
          self.modal_open = true;
          return;
        }
      }
    } else {
      match output::create_output_dir(Path::new(&self.out_dir_c1s2), true) {
        Ok(o) => o,
        Err(e) => {
          self.err_msg = e.to_owned();
          self.modal_open = true;
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
          output_dir_str.as_str(),
        ) {
          e.clone_into(&mut self.err_msg);
          self.modal_open = true;
          return;
        }
      } else if let Err(e) = output::write_converted_password_text_file(
        &convert::txt_to_another_version(&self.txt_file, password_version),
        password_version.rev(),
        &output_dir_str,
      ) {
        e.clone_into(&mut self.err_msg);
        self.modal_open = true;
        return;
      }
    }

    if self.to_export_pwd_bin_c1s2 {
      if let Err(e) = output::write_memory_dump_file(&target_password_bytes, &output_dir_str) {
        e.clone_into(&mut self.err_msg);
        self.modal_open = true;
        return;
      }
    }

    if let Some(cheat_version) = self.pwd_cheat_ver_c1s2 {
      if let Err(e) =
        output::write_cheat_file(&target_password_bytes, cheat_version, &output_dir_str)
      {
        e.clone_into(&mut self.err_msg);
        self.modal_open = true;
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
          self.modal_open = true;
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
        self.modal_open = true;
        return;
      }
    }

    // More detailed output logs, to improve user experience.
    // Sh*t code, but at least it works.
    self.out_log_c1s2.clear();
    self
      .out_log_c1s2
      .push_str(format!("输出目录：\n{output_dir_str}\n\n").as_str());
    self.out_log_c1s2.push_str("运行概况：\n");

    let mut result_index = 1;
    let source_password_grade_cn = self.for_run_c1s2.unwrap().0.get_name();
    let source_password_version_cn = password_version.get_name();

    self.out_log_c1s2.push_str(format!("{result_index}. 输入的源文件为{source_password_grade_cn}级别的{source_password_version_cn}版密码文本文件。\n").as_str());

    let convert_flag_str_option = if self.to_convert_pwd_text_ver {
      Some(password_version.rev().get_name())
    } else {
      None
    };

    let mut downgrade_str_option = None;
    if let Some(target_password_grade) = self.pwd_grade_c1s2 {
      result_index += 1;
      let target_password_grade_cn = target_password_grade.get_name();
      downgrade_str_option = Some(target_password_grade_cn);
    }

    match convert_flag_str_option {
      Some(convert_flag_str) => match downgrade_str_option {
        Some(downgrade_str) => {
          self.out_log_c1s2.push_str(format!("{result_index}. 已生成降级为{downgrade_str}级别的、转换为{convert_flag_str}版的“密码文本.txt”文件。\n").as_str());
        }
        None => {
          self.out_log_c1s2.push_str(format!("{result_index}. 已生成与源文件同级的{source_password_grade_cn}级别的、转换为{convert_flag_str}版的“密码文本.txt”文件。\n").as_str());
        }
      },
      None => match downgrade_str_option {
        Some(downgrade_str) => {
          self.out_log_c1s2.push_str(format!("{result_index}. 已生成降级为{downgrade_str}级别的、未经版本转换的{source_password_version_cn}版“密码文本.txt”文件。\n").as_str());
        }
        None => {
          self.out_log_c1s2.push_str(format!("{result_index}. 已生成与源文件同级的{source_password_grade_cn}级别的、未经版本转换的{source_password_version_cn}版“密码文本.txt”文件。\n").as_str());
        }
      },
    }

    if self.to_export_pwd_bin_c1s2 {
      result_index += 1;
      if let Some(target_password_grade_cn) = downgrade_str_option {
        self.out_log_c1s2.push_str(
          format!(
            "{result_index}. 已生成降级为{target_password_grade_cn}级别的“密码内存转储.dmp”文件。\n"
          )
          .as_str(),
        );
      } else {
        self.out_log_c1s2.push_str(format!("{result_index}. 已生成与源文件同级的{source_password_grade_cn}级别的“密码内存转储.dmp”文件。\n").as_str());
      }
    }

    if let Some(cheat_version) = self.pwd_cheat_ver_c1s2 {
      result_index += 1;
      let cheat_version_cn = cheat_version.get_name();
      if let Some(target_password_grade_cn) = downgrade_str_option {
        self.out_log_c1s2.push_str(format!("{result_index}. 已生成降级为{target_password_grade_cn}级别的、适用于{cheat_version_cn}版《黄金太阳 失落的时代》的“密码金手指.txt”文件。\n").as_str());
      } else {
        self.out_log_c1s2.push_str(format!("{result_index}. 已生成与源文件同级的{source_password_grade_cn}级别的、适用于{cheat_version_cn}版《黄金太阳 失落的时代》的“密码金手指.txt”文件。\n").as_str());
      }
    }

    if self.to_export_pwd_data_text_c1s2 {
      result_index += 1;
      if let Some(target_password_grade_cn) = downgrade_str_option {
        self.out_log_c1s2.push_str(
          format!(
            "{result_index}. 已生成降级为{target_password_grade_cn}级别的“密码数据.txt”文件\n"
          )
          .as_str(),
        );
      } else {
        self.out_log_c1s2.push_str(format!("{result_index}. 已生成与源文件同级的{source_password_grade_cn}级别的“密码数据.txt”文件\n").as_str());
      }
    }
  }

  fn run_sub_three(&mut self) {
    let mut password_bits = BitArray::from(&self.dmp_bits.as_ref().unwrap().get_bits());

    // Generate save data, and set target password bytes
    let save_data =
      save::get_save_data_with_password_bits(&mut password_bits, self.for_run_c1s3.unwrap());
    let target_password_bytes = match self.pwd_grade_c1s3 {
      Some(password_grade) => save::get_password_bytes_with_save_data(&save_data, password_grade),
      None => self.dmp_bytes.clone(),
    };

    // Create output directory.
    let output_dir_str = if self.out_dir_c1s3.is_empty() {
      match output::create_output_dir(Path::new(&self.input_file_path_c1s3), false) {
        Ok(o) => o,
        Err(e) => {
          self.err_msg = e.to_owned();
          self.modal_open = true;
          return;
        }
      }
    } else {
      match output::create_output_dir(Path::new(&self.out_dir_c1s3), true) {
        Ok(o) => o,
        Err(e) => {
          self.err_msg = e.to_owned();
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

    if self.to_export_data_text_3 {
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
    self
      .out_log_c1s3
      .push_str(format!("输出目录：\n{output_dir_str}\n\n").as_str());
    self.out_log_c1s3.push_str("运行概况：\n");

    let mut result_index = 1;
    let source_password_grade_cn = self.for_run_c1s3.unwrap().get_name();
    self.out_log_c1s3.push_str(
      format!("{result_index}. 输入的源文件为{source_password_grade_cn}级别的密码内存转储文件。\n")
        .as_str(),
    );

    let mut downgrade_str_option = None;
    if let Some(target_password_grade) = self.pwd_grade_c1s3 {
      result_index += 1;
      let target_password_grade_cn = target_password_grade.get_name();
      downgrade_str_option = Some(target_password_grade_cn);
      self.out_log_c1s3.push_str(
        format!(
          "{result_index}. 已生成降级为{target_password_grade_cn}级别的“密码内存转储.dmp”文件。\n"
        )
        .as_str(),
      );
    }

    if let Some(password_version) = self.pwd_text_ver_c1s3 {
      result_index += 1;
      let password_version_cn = password_version.get_name();
      if let Some(target_password_grade_cn) = downgrade_str_option {
        self.out_log_c1s3.push_str(format!("{result_index}. 已生成降级为{target_password_grade_cn}级别的{password_version_cn}版“密码文本.txt”文件。\n").as_str());
      } else {
        self.out_log_c1s3.push_str(format!("{result_index}. 已生成与源文件同级的{source_password_grade_cn}级别的{password_version_cn}版“密码文本.txt”文件。\n").as_str());
      }
    }

    if let Some(cheat_version) = self.pwd_cheat_ver_c1s3 {
      result_index += 1;
      let cheat_version_cn = cheat_version.get_name();
      if let Some(target_password_grade_cn) = downgrade_str_option {
        self.out_log_c1s3.push_str(format!("{result_index}. 已生成降级为{target_password_grade_cn}级别的、适用于{cheat_version_cn}版《黄金太阳 失落的时代》的“密码金手指.txt”文件。\n").as_str());
      } else {
        self.out_log_c1s3.push_str(format!("{result_index}. 已生成与源文件同级的{source_password_grade_cn}级别的、适用于{cheat_version_cn}版《黄金太阳 失落的时代》的“密码金手指.txt”文件。\n").as_str());
      }
    }

    if self.to_export_data_text_3 {
      result_index += 1;
      if let Some(target_password_grade_cn) = downgrade_str_option {
        self.out_log_c1s3.push_str(
          format!(
            "{result_index}. 已生成降级为{target_password_grade_cn}级别的“密码数据.txt”文件\n"
          )
          .as_str(),
        );
      } else {
        self.out_log_c1s3.push_str(format!("{result_index}. 已生成与源文件同级的{source_password_grade_cn}级别的“密码数据.txt”文件\n").as_str());
      }
    }
  }
}

fn validate_save_pwd(
  path_str: &str,
  to_export_all_save: bool,
) -> Result<Option<HashMap<u8, RawSaveData>>, &'static str> {
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
  if let Some(index) = save::get_loop_start_index_option(&raw_save_file) {
    loop_start_index = index;
  } else {
    return Err("输入的不是《黄金太阳 开启的封印》的存档文件！或者存档文件中没有数据！");
  }

  /* Get save data from save file with slot number.
  Also check if the save data is clear data. */
  let save_data_map =
    save::get_raw_save_data_map(to_export_all_save, &raw_save_file, loop_start_index);
  if save_data_map.is_empty() {
    if to_export_all_save {
      return Err("存档文件中没有数据！");
    }
    return Err("存档文件中没有通关数据！");
  }

  Ok(Some(save_data_map))
}

fn validate_txt(
  path_str: &str,
) -> Result<(String, Vec<u8>, BitArray, PasswordGrade, PasswordVersion), &'static str> {
  // Read password text file.
  let input_file_result = File::open(path_str);
  let mut input_file;
  if input_file_result.is_err() {
    return Err("打开密码文本文件时发生错误！");
  }
  input_file = input_file_result.unwrap();

  let mut password = String::new();
  if input_file.read_to_string(&mut password).is_err() {
    return Err("输入的不是有效的文本文件！");
  };
  // If the text file is empty...
  if password.is_empty() {
    return Err("文本文件内没有任何内容！");
  }
  // Remove all line break and whitespace.
  let password_without_whitespace = convert::remove_whitespace(&password);
  // Check password's length, and get its grade.
  let chars_cout = password_without_whitespace.chars().count();
  let source_password_grade = match chars_cout {
    260 => PasswordGrade::Gold,
    61 => PasswordGrade::Silver,
    16 => PasswordGrade::Bronze,
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
    password,
    password_bytes,
    password_bits,
    source_password_grade,
    password_version,
  ))
}

fn validate_dmp(path_str: &str) -> Result<(Vec<u8>, BitArray, PasswordGrade), &'static str> {
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
