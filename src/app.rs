use std::{collections::HashMap, time::Duration};

use egui::{Align2, Button, Id, Modal, RichText, Shadow, Sides, Widget};
use egui_notify::{Anchor, Toasts};

use crate::{
  core::{
    enums::{GameType, GameVersion, NameType, PasswordGrade, PasswordVersion},
    save::{BitArray, RawSaveData},
  },
  widgets::colors::{DANGER, LIGHT, SECONDARY, SUCCESS},
};

pub const GAME_VERSIONS: [GameVersion; 6] = [
  GameVersion::Japan,
  GameVersion::USA,
  GameVersion::Germany,
  GameVersion::Spain,
  GameVersion::France,
  GameVersion::Italy,
];

#[derive(Debug, PartialEq)]
pub enum CommandEnum {
  First,
  Second,
  Third,
  Fourth,
}

#[derive(Debug, PartialEq)]
pub enum SubCommandEnum {
  First,
  Second,
  Third,
}

pub struct App {
  about_window_open: bool,
  toasts: Toasts,
  command: CommandEnum,
  pub subcommand: SubCommandEnum,
  pub err_modal_open: bool,
  pub err_msg: String,
  // 1. 密码导出功能部分
  // 1.1 使用存档文件
  pub to_export_all_save: bool,
  pub input_file_path_c1s1: String,
  pub save_data_map: Option<HashMap<u8, RawSaveData>>,
  pub pwd_grade_c1s1: PasswordGrade,
  pub pwd_text_ver_c1s1: Option<PasswordVersion>,
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
  pub txt_src_grade_version: Option<(PasswordGrade, PasswordVersion)>,
  pub pwd_grade_c1s2: Option<PasswordGrade>,
  pub to_convert_pwd_text_ver: bool,
  pub to_export_pwd_bin_c1s2: bool,
  pub pwd_cheat_ver_c1s2: Option<GameVersion>,
  pub to_export_pwd_data_text_c1s2: bool,
  pub out_dir_c1s2: String,
  pub out_log_c1s2: String,
  // 1.3 使用内存转储文件
  pub input_file_path_c1s3: String,
  pub dmp_bytes: Vec<u8>,
  pub dmp_bits: Option<BitArray>,
  pub dmp_src_grade: Option<PasswordGrade>,
  pub pwd_grade_c1s3: Option<PasswordGrade>,
  pub pwd_text_ver_c1s3: Option<PasswordVersion>,
  pub pwd_cheat_ver_c1s3: Option<GameVersion>,
  pub to_export_pwd_data_text_c1s3: bool,
  pub out_dir_c1s3: String,
  pub out_log_c1s3: String,
  // 2. 存档转换功能部分
  pub input_file_path_c2: String,
  pub save_file: Vec<u8>,
  pub save_src_type_index: Option<(GameType, GameVersion, usize)>,
  pub default_name_ver: Option<NameType>,
  pub build_date_ver: Option<GameVersion>,
  pub out_dir_c2: String,
  pub out_log_c2: String,
  // 3. 自定义角色名部分
  /// 主角团7人的自定义名字
  pub new_pc_names: Vec<String>,
  pub out_log_c3: String,
  // 4. 打汉化补丁功能部分
  pub decompressed_bps: Vec<u8>,
  pub input_file_path_c4: String,
  pub input_file_crc32: Option<u32>,
  pub out_dir_c4: String,
  pub out_log_c4: String,
}

impl Default for App {
  fn default() -> Self {
    Self {
      about_window_open: false,
      toasts: Toasts::default()
        .with_anchor(Anchor::BottomRight)
        .with_shadow(Shadow {
          offset: [1, 2],
          blur: 10,
          spread: 3,
          color: egui::Color32::from_black_alpha(70),
        }),
      command: CommandEnum::First,
      subcommand: SubCommandEnum::First,
      err_modal_open: false,
      err_msg: String::new(),
      // 1. 密码导出功能部分
      // 1.1 使用存档文件
      to_export_all_save: false,
      input_file_path_c1s1: String::new(),
      save_data_map: None,
      pwd_grade_c1s1: PasswordGrade::Gold,
      pwd_text_ver_c1s1: Some(PasswordVersion::English),
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
      txt_src_grade_version: None,
      pwd_grade_c1s2: None,
      to_convert_pwd_text_ver: false,
      to_export_pwd_bin_c1s2: false,
      pwd_cheat_ver_c1s2: None,
      to_export_pwd_data_text_c1s2: false,
      out_dir_c1s2: String::new(),
      out_log_c1s2: String::new(),
      // 1.3 使用内存转储文件
      input_file_path_c1s3: String::new(),
      dmp_bytes: Vec::new(),
      dmp_bits: None,
      dmp_src_grade: None,
      pwd_grade_c1s3: None,
      pwd_text_ver_c1s3: None,
      pwd_cheat_ver_c1s3: None,
      to_export_pwd_data_text_c1s3: false,
      out_dir_c1s3: String::new(),
      out_log_c1s3: String::new(),
      // 2. 存档转换功能部分
      input_file_path_c2: String::new(),
      save_file: Vec::new(),
      save_src_type_index: None,
      default_name_ver: None,
      build_date_ver: None,
      out_dir_c2: String::new(),
      out_log_c2: String::new(),
      // 3. 自定义角色名部分
      new_pc_names: vec![String::new(); 7],
      out_log_c3: String::new(),
      // 4. 打汉化补丁功能部分
      decompressed_bps: {
        zstd::decode_all(include_bytes!(concat!(env!("OUT_DIR"), "/all_bps.zst")).as_slice())
          .unwrap()
      },
      input_file_path_c4: String::new(),
      input_file_crc32: None,
      out_dir_c4: String::new(),
      out_log_c4: String::new(),
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
    // 解压内嵌字体
    let decompressed_font =
      zstd::decode_all(include_bytes!(concat!(env!("OUT_DIR"), "/font.zst")).as_slice()).unwrap();
    // 替换默认字体以支持显示CJK等字符
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
      "simplified_chinese_font".to_owned(),
      std::sync::Arc::new(egui::FontData::from_owned(decompressed_font)),
    );
    fonts
      .families
      .entry(egui::FontFamily::Proportional)
      .or_default()
      .insert(0, "simplified_chinese_font".to_owned());
    cc.egui_ctx.set_fonts(fonts);

    Default::default()
  }
}

impl eframe::App for App {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    // “关于”窗口
    if self.about_window_open {
      egui::Window::new("关于")
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .open(&mut self.about_window_open)
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
              ui.hyperlink_to("更纱黑体", "https://github.com/be5invis/Sarasa-Gothic");
            });

            ui.separator();
            ui.label("© 2025 ♥ Hambaka");
            ui.hyperlink_to("Powered by egui", "https://www.egui.rs/");
          })
        });
    }

    // 错误信息显示窗口
    if self.err_modal_open {
      let modal = Modal::new(Id::new("ID_MODAL_ERROR_MSG")).show(ctx, |ui| {
        ui.set_width(250.0);
        ui.heading("错误！");
        ui.add_space(16.0);
        ui.label(self.err_msg.clone());
        ui.separator();
        Sides::new().show(
          ui,
          |_ui| {},
          |ui| {
            if ui.button("关闭").clicked() {
              // 新版本中应使用
              // ui.close()
              self.err_modal_open = false;
            }
          },
        );
      });

      if modal.should_close() {
        self.err_modal_open = false;
      }
    }

    // 顶部面板，主要用于实现菜单栏
    egui::TopBottomPanel::top("ID_MENU_BAR").show(ctx, |ui| {
      // 定义快捷键
      if ui.ctx().input_mut(|input| {
        input.consume_shortcut(&egui::KeyboardShortcut::new(
          egui::Modifiers::COMMAND,
          egui::Key::Q,
        ))
      }) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
      }

      // 菜单栏
      egui::MenuBar::new().ui(ui, |ui| {
        ui.menu_button("🗀 文件", |ui| {
          if ui.button("❓ 关于").clicked() {
            self.about_window_open = !self.about_window_open;
            ui.close();
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
      });
    });

    // 功能区
    egui::CentralPanel::default().show(ctx, |ui| {
      egui::ScrollArea::vertical().show(ui, |ui| {
        // 功能选择“标签”页
        ui.columns(4, |columns| {
          columns[0].selectable_value(&mut self.command, CommandEnum::First, "❶ 密码导出功能");
          columns[1].selectable_value(&mut self.command, CommandEnum::Second, "❷ 存档转换功能");
          columns[2].selectable_value(&mut self.command, CommandEnum::Third, "❸ 自定义角色名");
          columns[3].selectable_value(&mut self.command, CommandEnum::Fourth, "❹ 应用汉化补丁");
        });
        // 显示对应的功能页面
        match self.command {
          CommandEnum::First => self.show_c1(ui),
          CommandEnum::Second => self.show_c2(ui),
          CommandEnum::Third => self.show_c3(ui),
          CommandEnum::Fourth => self.show_c4(ui),
        }
        ui.separator();

        // 判断是否可以点击运行按钮
        let enable_run_button = match self.command {
          // 1. 密码导出功能部分
          CommandEnum::First => match self.subcommand {
            // 1.1 使用存档文件
            SubCommandEnum::First => {
              let bool1 = self.save_data_map.is_some();
              let bool2 = self.pwd_text_ver_c1s1.is_some()
                || self.to_export_pwd_bin_c1s1
                || self.pwd_cheat_ver_c1s1.is_some()
                || self.to_export_pwd_data_text_c1s1;

              bool1 && bool2
            }
            // 1.2 使用密码文本文件
            SubCommandEnum::Second => {
              let bool1 = self.txt_bits.is_some();
              let bool2 = self.pwd_grade_c1s2.is_some()
                || self.to_convert_pwd_text_ver
                || self.to_export_pwd_bin_c1s2
                || self.pwd_cheat_ver_c1s2.is_some()
                || self.to_export_pwd_data_text_c1s2;

              bool1 && bool2
            }
            // 1.3 使用内存转储文件
            SubCommandEnum::Third => {
              let bool1 = self.dmp_bits.is_some();
              let bool2 = self.pwd_grade_c1s3.is_some()
                || self.pwd_text_ver_c1s3.is_some()
                || self.pwd_cheat_ver_c1s3.is_some()
                || self.to_export_pwd_data_text_c1s3;

              bool1 && bool2
            }
          },
          // 2. 存档转换功能部分
          CommandEnum::Second => {
            let bool1 = self.save_src_type_index.is_some();
            let bool2 = self.default_name_ver.is_some() || self.build_date_ver.is_some();

            bool1 && bool2
          }
          // 3. 自定义角色名部分
          CommandEnum::Third => {
            let mut bool1 = false;
            for ele in self.new_pc_names.clone() {
              if !ele.is_empty() {
                bool1 = true;
              }
            }
            bool1
          }
          CommandEnum::Fourth => !self.input_file_path_c4.is_empty(),
        };

        // 判断是否可以点击输出按钮
        let enable_copy_button = !match self.command {
          CommandEnum::First => match self.subcommand {
            SubCommandEnum::First => self.out_log_c1s1.is_empty(),
            SubCommandEnum::Second => self.out_log_c1s2.is_empty(),
            SubCommandEnum::Third => self.out_log_c1s3.is_empty(),
          },
          CommandEnum::Second => self.out_log_c2.is_empty(),
          CommandEnum::Third => self.out_log_c3.is_empty(),
          CommandEnum::Fourth => self.out_log_c4.is_empty(),
        };

        ui.vertical_centered(|ui| {
          if enable_run_button {
            if ui
              .add(Button::new(RichText::new("⏵ 运行").color(LIGHT)).fill(SUCCESS))
              .clicked()
              || ui.input(|i| i.key_pressed(egui::Key::Enter))
            {
              match self.command {
                CommandEnum::First => self.run_c1(),
                CommandEnum::Second => self.run_c2(),
                CommandEnum::Third => self.run_c3(),
                CommandEnum::Fourth => self.run_c4(),
              }
            }
          } else {
            ui.add_enabled(
              false,
              Button::new(RichText::new("❌ 运行").color(LIGHT)).fill(DANGER),
            );
          }
          // 输出按钮
          if ui
            .add_enabled(
              enable_copy_button,
              Button::new(RichText::new("🗐 复制").color(LIGHT)).fill(SECONDARY),
            )
            .clicked()
          {
            let log = match self.command {
              CommandEnum::First => match self.subcommand {
                SubCommandEnum::First => &self.out_log_c1s1,
                SubCommandEnum::Second => &self.out_log_c1s2,
                SubCommandEnum::Third => &self.out_log_c1s3,
              },
              CommandEnum::Second => &self.out_log_c2,
              CommandEnum::Third => &self.out_log_c3,
              CommandEnum::Fourth => &self.out_log_c4,
            };
            ctx.copy_text(log.to_string());
            self
              .toasts
              .success("已复制输出内容至剪贴板！")
              .duration(Some(Duration::from_secs(5)));
          }
        });

        // 显示输出日志（概要）
        if enable_copy_button {
          match self.command {
            CommandEnum::First => self.show_log_c1(ui),
            CommandEnum::Second => self.show_log_c2(ui),
            CommandEnum::Third => self.show_log_c3(ui),
            CommandEnum::Fourth => self.show_log_c4(ui),
          }
        }
      });
    });

    self.toasts.show(ctx);
  }
}
