use std::{
  fmt::Write as _,
  fs::{File, create_dir_all},
  io::{Read, Write},
  path::{Path, PathBuf},
};

use egui::{ComboBox, Label, RichText, Ui, Widget};
use rfd::FileDialog;

use crate::{
  App,
  app::{AtomsEnum, SubCommandEnum},
  core::{
    enums::PasswordGrade,
    password::{
      decode::read_from_bits,
      encode::{encode, from_raw_save},
    },
    rom::inject::{PasswordCombination, TlaRomAnalysis, analyze, inject},
  },
  io::suffixed_file_name,
  ui::window::pwd_info::SaveSlotEnum,
  widgets::buttons::{reset_btn, select_dir_btn, select_file_btn},
};

const PWD_SRC_LABEL: [&str; 3] = ["金密码来源", "银密码来源", "铜密码来源"];
const PWD_SRC_DESC: [&str; 3] = [
  "自动输入金密码所使用的密码数据来源。",
  "自动输入银密码所使用的密码数据来源。",
  "自动输入铜密码所使用的密码数据来源。",
];

const DESC_C5: &str = r"使用该功能可为《黄金太阳 失落的时代》注入能够自动输入密码的程序。
支持的版本：
1. 官方版本ROM（日/美/德/西/法/意）
2. 流传度最高的中文汉化版ROM（SOMA汉化组/Mobile+elffinal）

注：
1. 游戏版本的来源多种多样，无法一一进行测试。";

impl App {
  pub fn show_c5(&mut self, ui: &mut Ui, svg_icon_size: f32, is_dark: bool) {
    // 正式界面
    Label::new(DESC_C5).selectable(false).ui(ui);

    ui.add_space(16.0);

    egui::Grid::new("ID_GRID_C5")
      .num_columns(2)
      .spacing([40.0, 4.0])
      .striped(true)
      .show(ui, |ui| {
        // 第1行 -> 输入文件路径 【选择文件】【重置】
        Label::new("输入文件路径")
          .selectable(false)
          .ui(ui)
          .on_hover_text("《黄金太阳 失落的时代》的ROM文件");
        ui.horizontal(|ui| {
          if ui.add(select_file_btn(svg_icon_size)).clicked() {
            if let Some(file) = FileDialog::new().pick_file() {
              let new_file_path = file.to_string_lossy().into_owned();
              // 验证ROM文件
              let result = validate_input_file(&new_file_path, false);
              match result {
                Ok(_) => {
                  self.input_file_path_c5 = new_file_path;
                }
                Err(e) => {
                  self.fail(e);
                }
              }
            }
          }
          if self.input_file_path_c5.is_empty() {
            ui.add_enabled(false, reset_btn(svg_icon_size));
          } else if ui.add(reset_btn(svg_icon_size)).clicked() {
            self.input_file_path_c5.clear();
          }
          ui.add(
            egui::TextEdit::singleline(&mut self.input_file_path_c5)
              .interactive(false)
              .hint_text("ROM文件路径")
              .desired_width(f32::INFINITY),
          );
        });
        ui.end_row();

        // 第2行 -> 金密码来源
        // 第3行 -> 银密码来源
        // 第4行 -> 铜密码来源

        for i in 0..3 {
          let data_is_available = self.save_is_available || self.txt_bits.is_some() || self.dmp_bits.is_some();

          if !data_is_available {
            Label::new(RichText::new(PWD_SRC_LABEL[i]).weak())
              .selectable(false)
              .ui(ui)
              .on_hover_text(PWD_SRC_DESC[i]);
            ui.add_enabled_ui(false, |ui| {
              ComboBox::from_id_salt(format!("ID_CB_C5_{}_1（不可用）", PWD_SRC_LABEL[i]))
                .width(100.0)
                .selected_text("无")
                .show_ui(ui, |_ui| {});
            });
          } else {
            Label::new(PWD_SRC_LABEL[i]).selectable(false).ui(ui).on_hover_text(PWD_SRC_DESC[i]);
            ui.horizontal(|ui| {
              ComboBox::from_id_salt(format!("ID_CB_C5_{}_1", PWD_SRC_LABEL[i]))
                .width(100.0)
                .selected_text({
                  match &self.pwd_src[i] {
                    Some(sub_command) => AtomsEnum::ImageAndText(sub_command.get_atoms(svg_icon_size, is_dark)) ,
                    None => AtomsEnum::Text("无"),
                  }
                })
                .show_ui(ui, |ui| {
                  if ui.selectable_value(&mut self.pwd_src[i], None, "无").changed() {
                    self.pwd_src_slot[i] = None;
                  };

                  ui.add_enabled_ui(self.save_is_available, |ui| {
                    if ui
                      .selectable_value(&mut self.pwd_src[i], Some(SubCommandEnum::First), SubCommandEnum::First.get_atoms(svg_icon_size,is_dark))
                      .changed()
                    {
                      self.pwd_src_slot[i] = self.first_available_slot;
                    };
                  });

                  let bool_txt = match i {
                    0 => {
                      if let Some((src_pwd_grade, _)) = self.txt_src_grade_version {
                        match src_pwd_grade {
                          PasswordGrade::Gold => true,
                          PasswordGrade::Silver | PasswordGrade::Bronze => false,
                        }
                      } else {
                        false
                      }
                    }
                    1 => {
                      if let Some((src_pwd_grade, _)) = self.txt_src_grade_version {
                        match src_pwd_grade {
                          PasswordGrade::Gold | PasswordGrade::Silver => true,
                          PasswordGrade::Bronze => false,
                        }
                      } else {
                        false
                      }
                    }
                    _ => true,
                  };

                  ui.add_enabled_ui(self.txt_bits.is_some() && bool_txt, |ui| {
                    if ui
                      .selectable_value(&mut self.pwd_src[i], Some(SubCommandEnum::Second), SubCommandEnum::Second.get_atoms(svg_icon_size,is_dark))
                      .changed()
                    {
                      self.pwd_src_slot[i] = Some(SaveSlotEnum::First);
                    };
                  });

                  let bool_dmp = match i {
                    0 => {
                      if let Some(src_pwd_grade) = self.dmp_src_grade {
                        match src_pwd_grade {
                          PasswordGrade::Gold => true,
                          PasswordGrade::Silver | PasswordGrade::Bronze => false,
                        }
                      } else {
                        false
                      }
                    }
                    1 => {
                      if let Some(src_pwd_grade) = self.dmp_src_grade {
                        match src_pwd_grade {
                          PasswordGrade::Gold | PasswordGrade::Silver => true,
                          PasswordGrade::Bronze => false,
                        }
                      } else {
                        false
                      }
                    }
                    _ => true,
                  };

                  ui.add_enabled_ui(self.dmp_bits.is_some() && bool_dmp, |ui| {
                    if ui
                      .selectable_value(&mut self.pwd_src[i], Some(SubCommandEnum::Third), SubCommandEnum::Third.get_atoms(svg_icon_size,is_dark))
                      .changed()
                    {
                      self.pwd_src_slot[i] = Some(SaveSlotEnum::First);
                    };
                  });
                });

              if self.pwd_src[i] == Some(SubCommandEnum::First) {
                ComboBox::from_id_salt(format!("ID_CB_C5_{}_2", PWD_SRC_LABEL[i]))
                  .width(100.0)
                  .selected_text(match &self.pwd_src_slot[i] {
                    Some(save_slot) => AtomsEnum::ImageAndText(save_slot.get_atoms(svg_icon_size, is_dark, false)),
                    None => AtomsEnum::Text("无"),
                  })
                  .show_ui(ui, |ui| {
                    for j in 0..3 {
                      ui.add_enabled_ui(self.raw_save_data[j].is_some(), |ui| {
                        ui.selectable_value(
                          &mut self.pwd_src_slot[i],
                          Some(SaveSlotEnum::SAVE_SLOTS[j]),
                          SaveSlotEnum::SAVE_SLOTS[j].get_atoms(svg_icon_size, is_dark, false),
                        );
                      });
                    }
                  });
              }
            });
          }

          ui.end_row();
        }

        // 第5行 -> 指定输出文件保存目录  【选择目录】【重置】
        Label::new("指定输出文件保存目录")
          .selectable(false)
          .ui(ui)
          .on_hover_text("自定义输出的汉化版ROM文件路径（推荐留空），若留空则将转换后的ROM文件保存到输入文件的同目录下。");
        ui.horizontal(|ui| {
          if ui.add(select_dir_btn(svg_icon_size)).clicked() {
            if let Some(file) = FileDialog::new().pick_folder() {
              self.out_dir_c5 = file.to_string_lossy().into_owned();
            }
          }
          if self.out_dir_c5.is_empty() {
            ui.add_enabled(false, reset_btn(svg_icon_size));
          } else if ui.add(reset_btn(svg_icon_size)).clicked() {
            self.out_dir_c5.clear();
          }
          ui.add(
            egui::TextEdit::singleline(&mut self.out_dir_c5)
              .interactive(false)
              .hint_text("最新版汉化ROM输出路径")
              .desired_width(f32::INFINITY),
          );
        });
        ui.end_row();
      });
  }

  pub fn run_c5(&mut self) {
    let sources = [
      self.pwd_src[0].is_some(),
      self.pwd_src[1].is_some(),
      self.pwd_src[2].is_some(),
    ];
    let inject_type = PasswordCombination::from_sources(sources).unwrap();

    let gold_pwd_bytes = if let Some(sub_command) = &self.pwd_src[0] {
      match sub_command {
        SubCommandEnum::First => {
          let save_slot = &self.pwd_src_slot[0].unwrap();
          let raw_save_data = &self.raw_save_data[*save_slot as usize]
            .as_ref()
            .unwrap()
            .data;
          let password_bytes = from_raw_save(&raw_save_data, PasswordGrade::Gold);
          Some(password_bytes)
        }
        SubCommandEnum::Second => Some(self.txt_bytes.clone()),
        SubCommandEnum::Third => Some(self.dmp_bytes.clone()),
      }
    } else {
      None
    };

    let silver_pwd_bytes = if let Some(sub_command) = &self.pwd_src[1] {
      match sub_command {
        SubCommandEnum::First => {
          let save_slot = &self.pwd_src_slot[1].unwrap();
          let raw_save_data = &self.raw_save_data[*save_slot as usize]
            .as_ref()
            .unwrap()
            .data;
          let password_bytes = from_raw_save(&raw_save_data, PasswordGrade::Silver);
          Some(password_bytes)
        }
        SubCommandEnum::Second => match self.txt_src_grade_version {
          Some((src_grade, _)) => {
            if src_grade == PasswordGrade::Silver {
              Some(self.txt_bytes.clone())
            } else {
              let password_bits = self.txt_bits.as_ref().unwrap();
              let password_data =
                read_from_bits(&password_bits, self.txt_src_grade_version.unwrap().0);
              let target_password_bytes = encode(&password_data, PasswordGrade::Silver);

              Some(target_password_bytes)
            }
          }
          None => unreachable!(),
        },
        SubCommandEnum::Third => match self.dmp_src_grade {
          Some(src_grade) => {
            if src_grade == PasswordGrade::Silver {
              Some(self.dmp_bytes.clone())
            } else {
              let password_bits = self.dmp_bits.as_ref().unwrap();
              let password_data = read_from_bits(&password_bits, self.dmp_src_grade.unwrap());
              let target_password_bytes = encode(&password_data, PasswordGrade::Silver);

              Some(target_password_bytes)
            }
          }
          None => unreachable!(),
        },
      }
    } else {
      None
    };

    let bronze_pwd_bytes = if let Some(sub_command) = &self.pwd_src[2] {
      match sub_command {
        SubCommandEnum::First => {
          let save_slot = &self.pwd_src_slot[2].unwrap();
          let raw_save_data = &self.raw_save_data[*save_slot as usize]
            .as_ref()
            .unwrap()
            .data;
          let password_bytes = from_raw_save(&raw_save_data, PasswordGrade::Bronze);
          Some(password_bytes)
        }
        SubCommandEnum::Second => match self.txt_src_grade_version {
          Some((src_grade, _)) => {
            if src_grade == PasswordGrade::Bronze {
              Some(self.txt_bytes.clone())
            } else {
              let password_bits = self.txt_bits.as_ref().unwrap();
              let password_data =
                read_from_bits(&password_bits, self.txt_src_grade_version.unwrap().0);
              let target_password_bytes = encode(&password_data, PasswordGrade::Bronze);

              Some(target_password_bytes)
            }
          }
          None => unreachable!(),
        },
        SubCommandEnum::Third => match self.dmp_src_grade {
          Some(src_grade) => {
            if src_grade == PasswordGrade::Bronze {
              Some(self.dmp_bytes.clone())
            } else {
              let password_bits = self.dmp_bits.as_ref().unwrap();
              let password_data = read_from_bits(&password_bits, self.dmp_src_grade.unwrap());
              let target_password_bytes = encode(&password_data, PasswordGrade::Bronze);

              Some(target_password_bytes)
            }
          }
          None => unreachable!(),
        },
      }
    } else {
      None
    };

    let (analysis, bytes) = validate_input_file(&self.input_file_path_c5, true).unwrap();

    let result = inject(
      bytes.unwrap(),
      &analysis,
      inject_type,
      [gold_pwd_bytes, silver_pwd_bytes, bronze_pwd_bytes],
    );

    if let Err(e) = result {
      self.fail(e.to_string());
      self.input_file_path_c5.clear();
      return;
    }

    let new_rom = result.unwrap();

    // Start to create and write output ROM file.
    let output_path;

    let input_path = Path::new(&self.input_file_path_c5);
    let file_name = suffixed_file_name(input_path, inject_type.get_name());

    if self.out_dir_c5.is_empty() {
      let parent = input_path.parent().unwrap().to_str().unwrap();
      output_path = PathBuf::from(parent).join(file_name);
    } else {
      if create_dir_all(self.out_dir_c5.clone()).is_err() {
        "创建目录失败！".clone_into(&mut self.err_msg);
        self.err_modal_open = true;
        return;
      }
      output_path = PathBuf::from(self.out_dir_c5.clone()).join(file_name);
    }

    let output_file = File::create(output_path.clone());
    if let Ok(mut file) = output_file {
      if file.write_all(&new_rom).is_err() {
        self.err_msg = format!("创建 “{}” 失败！", output_path.to_str().unwrap());
        self.err_modal_open = true;
        return;
      }
    } else {
      self.err_msg = format!("创建 “{}” 失败！", output_path.to_str().unwrap());
      self.err_modal_open = true;
      return;
    }

    // Log
    self.out_log_c5.clear();
    let _ = write!(
      self.out_log_c5,
      "输出文件：\n{}\n\n",
      output_path.to_str().unwrap()
    );

    self.out_log_c5.push_str("运行概况：\n");

    let mut result_index = 1;

    if analysis.is_known_rom {
      let _ = writeln!(
        self.out_log_c5,
        "{result_index}. 输入的源文件版本信息：“{}”（无任何修改）",
        analysis.rom_name
      );
    } else {
      match analysis.already_injected {
        Some(combo) => {
          let _ = writeln!(
            self.out_log_c5,
            "{result_index}. 输入的源文件（本质）版本信息：“{}”，（已注入密码，密码类型：{}）",
            analysis.rom_name,
            combo.get_name()
          );
        }
        None => {
          let _ = writeln!(
            self.out_log_c5,
            "{result_index}. 输入的源文件（本质）版本信息：“{}”，（未注入密码）",
            analysis.rom_name
          );
        }
      }
    }

    result_index += 1;
    let _ = writeln!(
      self.out_log_c5,
      "{result_index}. 本次注入的密码类型为：“{}”",
      inject_type.get_name()
    );
  }

  pub fn show_c5_log(&mut self, ui: &mut Ui) {
    Label::new(&self.out_log_c5).selectable(false).ui(ui);
  }
}

fn validate_input_file(
  path_str: &str,
  get_bytes: bool,
) -> Result<(TlaRomAnalysis, Option<Vec<u8>>), String> {
  // Read rom file.
  let input_file_result = File::open(path_str);
  let mut rom_file;
  if input_file_result.is_err() {
    return Err("打开 ROM 文件时发生错误！".to_owned());
  }
  rom_file = input_file_result.unwrap();

  let file_size = rom_file.metadata().unwrap().len() as usize;
  if file_size > 0x200_0000 {
    return Err("ROM 大小不应超过 32 MB！".to_owned());
  }

  let mut rom_bytes = Vec::new();
  rom_file.read_to_end(&mut rom_bytes).unwrap();

  analyze(&rom_bytes).map_or_else(
    |e| Err(e.to_string()),
    |v| {
      Ok(if get_bytes {
        (v, Some(rom_bytes))
      } else {
        (v, None)
      })
    },
  )
}
