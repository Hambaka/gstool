use std::time::Duration;

use crate::{
  App,
  app::{CommandEnum, SubCommandEnum},
  widgets::buttons::{copy_btn, run_btn_available, run_btn_unavailable},
};

const CUSTOM_ORDER_COMMANDS: [CommandEnum; CommandEnum::COMMANDS.len()] = [
  CommandEnum::First,
  CommandEnum::Fifth,
  CommandEnum::Third,
  CommandEnum::Second,
  CommandEnum::Fourth,
];

impl App {
  pub fn show_main_panel(&mut self, ui: &mut egui::Ui, svg_icon_size: f32, is_dark: bool) {
    egui::CentralPanel::default().show(ui, |ui| {
      if self.about_window_open || self.pwd_info_window_open {
        ui.disable();
      }

      egui::ScrollArea::vertical().show(ui, |ui| {
        // 功能选择“标签”页
        ui.columns_const(|columns: &mut [egui::Ui; CommandEnum::COMMANDS.len()]| {
          for i in 0..CUSTOM_ORDER_COMMANDS.len() {
            columns[i].selectable_value(
              &mut self.command,
              CUSTOM_ORDER_COMMANDS[i],
              CUSTOM_ORDER_COMMANDS[i].get_atoms(svg_icon_size, is_dark),
            );
          }
        });
        // 显示对应的功能页面
        match self.command {
          CommandEnum::First => self.show_c1(ui, svg_icon_size, is_dark),
          CommandEnum::Second => self.show_c2(ui, svg_icon_size),
          CommandEnum::Third => self.show_c3(ui),
          CommandEnum::Fourth => self.show_c4(ui, svg_icon_size),
          CommandEnum::Fifth => self.show_c5(ui, svg_icon_size, is_dark),
        }
        ui.separator();

        // 判断是否可以点击运行按钮
        let enable_run_button = match self.command {
          // 1. 密码导出功能部分
          CommandEnum::First => match self.subcommand {
            // 1.1 使用存档文件
            SubCommandEnum::First => {
              let bool1 = self.save_is_available.is_some();
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
          CommandEnum::Fifth => {
            !self.input_file_path_c5.is_empty()
              && (self.pwd_src[0].is_some()
                || self.pwd_src[1].is_some()
                || self.pwd_src[2].is_some())
          }
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
          CommandEnum::Fifth => self.out_log_c5.is_empty(),
        };

        ui.vertical_centered(|ui| {
          if enable_run_button {
            if ui.add(run_btn_available(svg_icon_size)).clicked()
              || ui.input_mut(|input| {
                input.consume_shortcut(&egui::KeyboardShortcut::new(
                  egui::Modifiers::COMMAND,
                  egui::Key::Enter,
                ))
              })
            {
              match self.command {
                CommandEnum::First => self.run_c1(),
                CommandEnum::Second => self.run_c2(),
                CommandEnum::Third => self.run_c3(),
                CommandEnum::Fourth => self.run_c4(),
                CommandEnum::Fifth => self.run_c5(),
              }
            }
          } else {
            ui.add_enabled(false, run_btn_unavailable(svg_icon_size));
          }
          // 输出按钮
          if ui
            .add_enabled(enable_copy_button, copy_btn(svg_icon_size))
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
              CommandEnum::Fifth => &self.out_log_c5,
            };
            ui.copy_text(log.to_string());
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
            CommandEnum::Fifth => self.show_c5_log(ui),
          }
        }
      });
    });
  }
}
