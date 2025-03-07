use std::collections::HashMap;

use egui::{Align2, Button, Id, Modal, Sides, Widget};

use crate::core::{
  enums::{GameType, GameVersion, NameType, PasswordGrade, PasswordVersion},
  save::{BitArray, RawSaveData},
};

#[derive(Debug, PartialEq)]
pub enum CommandEnum {
  First,
  Second,
  Third,
}

pub struct App {
  window_open: bool,
  command: CommandEnum,
  pub subcommand: CommandEnum,
  pub modal_open: bool,
  pub err_msg: String,
  // 1. 密码导出功能部分
  // 1.1 使用存档文件
  pub input_file_path_c1s1: String,
  pub save_data_map: Option<HashMap<u8, RawSaveData>>,
  pub pwd_grade_c1s1: Option<PasswordGrade>,
  pub pwd_text_ver_c1s1: Option<PasswordVersion>,
  pub to_export_all_save: bool,
  pub to_export_pwd_bin_c1s1: bool,
  pub pwd_cheat_ver_c1s1: Option<GameVersion>,
  pub to_export_pwd_data_text_c1s1: bool,
  pub out_dir_c1s1: String,
  pub out_log_c1s1: String,
  // 1.2 使用密码文本文件
  pub input_file_path_c1s2: String,
  pub txt_file: String,
  pub txt_bytes: Vec<u8>,
  pub txt_bits: Option<BitArray>,
  pub pwd_grade_c1s2: Option<PasswordGrade>,
  pub to_convert_pwd_text_ver: bool,
  pub to_export_pwd_bin_c1s2: bool,
  pub pwd_cheat_ver_c1s2: Option<GameVersion>,
  pub to_export_pwd_data_text_c1s2: bool,
  pub out_dir_c1s2: String,
  pub for_run_c1s2: Option<(PasswordGrade, PasswordVersion)>,
  pub out_log_c1s2: String,
  // 1.3 使用内存转储文件
  pub input_file_path_c1s3: String,
  pub dmp_bytes: Vec<u8>,
  pub dmp_bits: Option<BitArray>,
  pub pwd_grade_c1s3: Option<PasswordGrade>,
  pub pwd_text_ver_c1s3: Option<PasswordVersion>,
  pub pwd_cheat_ver_c1s3: Option<GameVersion>,
  pub to_export_data_text_3: bool,
  pub out_dir_c1s3: String,
  pub for_run_c1s3: Option<PasswordGrade>,
  pub out_log_c1s3: String,
  // 2. 存档转换功能部分
  pub input_file_path_c2: String,
  pub save_file: Vec<u8>,
  pub default_name_ver: Option<NameType>,
  pub build_date_ver: Option<GameVersion>,
  pub out_dir_c2: String,
  pub for_run_c2: Option<(GameType, usize)>,
  pub out_log_c2: String,
  // 3. 自定义角色名部分
  /// 主角团7人的自定义名字
  pub new_pc_names: Vec<String>,
  pub out_log_c3: String,
}

impl Default for App {
  fn default() -> Self {
    Self {
      window_open: false,
      command: CommandEnum::First,
      subcommand: CommandEnum::First,
      modal_open: false,
      err_msg: String::new(),
      // 1. 密码导出功能部分
      // 1.1 使用存档文件
      input_file_path_c1s1: String::new(),
      save_data_map: None,
      pwd_grade_c1s1: Some(PasswordGrade::Gold),
      pwd_text_ver_c1s1: Some(PasswordVersion::English),
      to_export_all_save: false,
      to_export_pwd_bin_c1s1: false,
      pwd_cheat_ver_c1s1: None,
      to_export_pwd_data_text_c1s1: false,
      out_dir_c1s1: String::new(),
      out_log_c1s1: String::new(),
      // 1.2 使用密码文本文件
      input_file_path_c1s2: String::new(),
      txt_file: String::new(),
      txt_bytes: Vec::new(),
      txt_bits: None,
      pwd_grade_c1s2: None,
      to_convert_pwd_text_ver: false,
      to_export_pwd_bin_c1s2: false,
      pwd_cheat_ver_c1s2: None,
      to_export_pwd_data_text_c1s2: false,
      out_dir_c1s2: String::new(),
      for_run_c1s2: None,
      out_log_c1s2: String::new(),
      // 1.3 使用内存转储文件
      input_file_path_c1s3: String::new(),
      dmp_bytes: Vec::new(),
      dmp_bits: None,
      pwd_grade_c1s3: None,
      pwd_text_ver_c1s3: None,
      pwd_cheat_ver_c1s3: None,
      to_export_data_text_3: false,
      out_dir_c1s3: String::new(),
      for_run_c1s3: None,
      out_log_c1s3: String::new(),
      // 2. 存档转换功能部分
      input_file_path_c2: String::new(),
      save_file: Vec::new(),
      default_name_ver: None,
      build_date_ver: None,
      out_dir_c2: String::new(),
      for_run_c2: None,
      out_log_c2: String::new(),
      // 3. 自定义角色名部分
      new_pc_names: vec![String::new(); 7],
      out_log_c3: String::new(),
    }
  }
}

impl App {
  pub fn name() -> String {
    // env!("CARGO_PKG_VERSION")
    // env!("CARGO_PKG_NAME")
    format!("GSTool - {}", env!("CARGO_PKG_VERSION"))
  }

  pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
    // 替换默认字体以支持显示CJK等字符
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
      "unifont".to_owned(),
      std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
        "font/unifont-16.0.02.otf"
      ))),
    );
    fonts
      .families
      .entry(egui::FontFamily::Proportional)
      .or_default()
      .insert(0, "unifont".to_owned());
    fonts
      .families
      .entry(egui::FontFamily::Monospace)
      .or_default()
      .push("unifont".to_owned());
    cc.egui_ctx.set_fonts(fonts);

    Default::default()
  }
}

impl eframe::App for App {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    // “关于”窗口
    if self.window_open {
      egui::Window::new("关于")
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .open(&mut self.window_open)
        .show(ctx, |ui| {
          ui.vertical_centered(|ui| {
            ui.heading(format!("GSTool - {}", env!("CARGO_PKG_VERSION")));
            ui.label("为 GBA 的《黄金太阳》系列游戏准备的小工具！");
            ui.separator();
            ui.label("鸣谢");

            ui.horizontal(|ui| {
              ui.label("密码算法：");
              ui.hyperlink_to(
                "Golden-Sun-Password-Transfer",
                "https://github.com/Dyrati/Golden-Sun-Password-Transfer",
              );
            });

            ui.horizontal(|ui| {
              ui.label("界面字体：");
              ui.hyperlink_to("unifont", "https://unifoundry.com/unifont/index.html");
            });

            ui.separator();
            ui.label("© 2025 ♥ Hambaka");
            ui.hyperlink_to("Powered by egui", "https://www.egui.rs/");
          })
        });
    }

    // 错误信息显示窗口
    if self.modal_open {
      let modal = Modal::new(Id::new("Modal_Error_Msg")).show(ctx, |ui| {
        ui.set_width(250.0);

        ui.heading("错误！");
        ui.label(self.err_msg.clone());

        ui.separator();

        Sides::new().show(
          ui,
          |_ui| {},
          |ui| {
            if ui.button("关闭").clicked() {
              // 新版本中应使用
              // ui.close()
              self.modal_open = false;
            }
          },
        );
      });

      if modal.should_close() {
        self.modal_open = false;
      }
    }

    // 顶部面板，主要用于实现菜单栏
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
      // 定义快捷键
      if ui.ctx().input_mut(|input| {
        input.consume_shortcut(&egui::KeyboardShortcut::new(
          egui::Modifiers::COMMAND,
          egui::Key::Q,
        ))
      }) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
      };

      // 菜单栏
      egui::menu::bar(ui, |ui| {
        ui.menu_button("🗁 文件", |ui| {
          if ui.button("❓ 关于").clicked() {
            self.window_open = !self.window_open;
            ui.close_menu();
          }

          ui.separator();

          if egui::Button::new("🚪 退出")
            .shortcut_text(ui.ctx().format_shortcut(&egui::KeyboardShortcut::new(
              egui::Modifiers::COMMAND,
              egui::Key::Q,
            )))
            .ui(ui)
            .clicked()
          {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
          }
        });
        ui.add_space(16.0);
        egui::widgets::global_theme_preference_buttons(ui);
      });
    });

    // 功能区
    egui::CentralPanel::default().show(ctx, |ui| {
      egui::ScrollArea::vertical().show(ui, |ui| {
        ui.columns(3, |columns| {
          columns[0].selectable_value(&mut self.command, CommandEnum::First, "（1）密码导出功能");
          columns[1].selectable_value(&mut self.command, CommandEnum::Second, "（2）存档转换功能");
          columns[2].selectable_value(&mut self.command, CommandEnum::Third, "（3）自定义角色名");
        });

        match self.command {
          CommandEnum::First => self.show_pwd(ui),
          CommandEnum::Second => self.show_sav(ui),
          CommandEnum::Third => self.show_name(ui),
        }

        ui.separator();
        let enable_run_button = match self.command {
          CommandEnum::First => match self.subcommand {
            CommandEnum::First => {
              if self.input_file_path_c1s1.is_empty() {
                false
              } else {
                true
              }
            }
            CommandEnum::Second => {
              let temp1 = !self.input_file_path_c1s2.is_empty();

              let temp2 = if self.pwd_grade_c1s2.is_none()
                && !self.to_convert_pwd_text_ver
                && !self.to_export_pwd_bin_c1s2
                && self.pwd_cheat_ver_c1s2.is_none()
                && !self.to_export_pwd_data_text_c1s2
              {
                false
              } else {
                true
              };

              temp1 && temp2
            }
            CommandEnum::Third => {
              let temp1 = !self.input_file_path_c1s3.is_empty();

              let temp2 = if self.pwd_grade_c1s3.is_none()
                && self.pwd_text_ver_c1s3.is_none()
                && self.pwd_cheat_ver_c1s3.is_none()
                && !self.to_export_data_text_3
              {
                false
              } else {
                true
              };

              temp1 && temp2
            }
          },
          CommandEnum::Second => {
            let temp1 = !self.input_file_path_c2.is_empty();
            let temp2 = if self.default_name_ver.is_none() && self.build_date_ver.is_none() {
              false
            } else {
              true
            };

            temp1 && temp2
          }
          CommandEnum::Third => {
            let mut temp = false;
            for ele in self.new_pc_names.clone() {
              if !ele.is_empty() {
                temp = true;
              }
            }
            temp
          }
        };

        if enable_run_button {
          if ui.add_enabled(true, Button::new("运行")).clicked() {
            match self.command {
              CommandEnum::First => self.run_pwd(),
              CommandEnum::Second => self.run_save(),
              CommandEnum::Third => self.run_name(),
            }
          };
        } else {
          ui.add_enabled(false, Button::new("运行"));
        }

        match self.command {
          CommandEnum::First => match self.subcommand {
            CommandEnum::First => {
              if !self.out_log_c1s1.is_empty() {
                if ui.button("复制输出内容").clicked() {
                  ctx.copy_text(self.out_log_c1s1.clone());
                }
                ui.label(self.out_log_c1s1.clone());
              }
            }
            CommandEnum::Second => {
              if !self.out_log_c1s2.is_empty() {
                if ui.button("复制输出内容").clicked() {
                  ctx.copy_text(self.out_log_c1s2.clone());
                }
                ui.label(self.out_log_c1s2.clone());
              }
            }
            CommandEnum::Third => {
              if !self.out_log_c1s3.is_empty() {
                if ui.button("复制输出内容").clicked() {
                  ctx.copy_text(self.out_log_c1s3.clone());
                }
                ui.label(self.out_log_c1s3.clone());
              }
            }
          },
          CommandEnum::Second => {
            if !self.out_log_c2.is_empty() {
              if ui.button("复制输出内容").clicked() {
                ctx.copy_text(self.out_log_c2.clone());
              }
              ui.label(self.out_log_c2.clone());
            }
          }
          CommandEnum::Third => {
            if !self.out_log_c3.is_empty() {
              if ui.button("复制输出内容").clicked() {
                ctx.copy_text(self.out_log_c3.clone());
              }
              ui.label(self.out_log_c3.clone());
            }
          }
        }
      });
    });
  }
}
