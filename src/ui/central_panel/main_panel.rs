use std::time::Duration;

use crate::{
  App,
  logic::command::CommandEnum,
  ui::{
    central_panel::{custom_names, password_export, patch_rom, save_convert},
    page::{AppRequest, FrameCtx},
  },
  widgets::atoms::command_atoms,
  widgets::buttons::{copy_btn, run_btn_available, run_btn_unavailable},
};

const CUSTOM_ORDER_COMMANDS: [CommandEnum; CommandEnum::ALL.len()] = [
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
        ui.columns_const(|columns: &mut [egui::Ui; CommandEnum::ALL.len()]| {
          for i in 0..CUSTOM_ORDER_COMMANDS.len() {
            columns[i].selectable_value(
              &mut self.command,
              CUSTOM_ORDER_COMMANDS[i],
              command_atoms(CUSTOM_ORDER_COMMANDS[i], svg_icon_size, is_dark),
            );
          }
        });
        // 显示对应的功能页面
        let ctx = FrameCtx {
          svg_icon_size,
          is_dark,
        };
        match self.command {
          CommandEnum::First => {
            let requests = password_export::show(ui, &mut self.c1, &mut self.sources, ctx);
            self.requests.extend(requests);
          }
          CommandEnum::Second => {
            let requests = save_convert::show(ui, &mut self.c2, ctx);
            self.requests.extend(requests);
          }
          CommandEnum::Third => {
            let requests = custom_names::show(ui, &mut self.c3, ctx);
            self.requests.extend(requests);
          }
          CommandEnum::Fourth => {
            let requests = patch_rom::show(ui, &mut self.c4, ctx);
            self.requests.extend(requests);
          }
          CommandEnum::Fifth => self.show_c5(ui, svg_icon_size, is_dark),
        }
        ui.separator();

        // 判断是否可以点击运行按钮
        let enable_run_button = self.can_run();

        // 判断是否可以点击输出按钮
        let enable_copy_button = !match self.command {
          CommandEnum::First => !self.c1.has_output(),
          CommandEnum::Second => !self.c2.has_output(),
          CommandEnum::Third => !self.c3.has_output(),
          CommandEnum::Fourth => !self.c4.has_output(),
          CommandEnum::Fifth => self.out_log_c5.is_empty(),
        };

        ui.vertical_centered(|ui| {
          if enable_run_button {
            if ui.add(run_btn_available(svg_icon_size)).clicked() {
              // 现在运行逻辑交由`App::logic()`处理，因此这里改为发送请求；
              self.requests.push(AppRequest::Run(self.command));
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
              CommandEnum::First => self.c1.log(),
              CommandEnum::Second => self.c2.log(),
              CommandEnum::Third => self.c3.log(),
              CommandEnum::Fourth => self.c4.log(),
              CommandEnum::Fifth => self.out_log_c5.as_str(),
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
            CommandEnum::First => password_export::show_log(ui, &self.c1),
            CommandEnum::Second => save_convert::show_log(ui, &self.c2),
            CommandEnum::Third => custom_names::show_log(ui, &self.c3),
            CommandEnum::Fourth => patch_rom::show_log(ui, &self.c4),
            CommandEnum::Fifth => self.show_c5_log(ui),
          }
        }
      });
    });
  }
}
