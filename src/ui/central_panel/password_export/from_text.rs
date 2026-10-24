//! ② 密码文本子页视图：ui侧只负责画widget、登记下一帧logic侧要处理的request。

use egui::{Grid, Label, TextEdit, Ui, Widget};
use rfd::FileDialog;

use crate::{
  core::enums::{GameRegion, PasswordGrade},
  logic::{
    command::SubCommandEnum,
    password_export::from_text::{FromTextRequest, FromTextState},
    sources::PasswordSources,
  },
  ui::{
    page::{AppRequest, FrameCtx},
    widgets::{
      buttons::{clipboard_btn, reset_btn, select_file_btn},
      rows::{PathAction, checkbox_row, combo_row, dir_row, pwd_info_checkbox_row},
    },
  },
};

const DESC_C1S2: &str = r"“密码级别”，“转换密码版本”，“生成内存转储文件”，“生成密码金手指”，“导出密码数据至文本文件”。
以上五项虽然都是可以留空或者不勾选的可选选项，但是必须要有其中一项有选择值或者已勾选。";

/// 画【② 密码文本】子页；返回需要 `App` 处理的跨功能请求（本页自己的待办已登记进 `state`）
///
/// 任何的IO操作（选文件、读剪贴板）都作为请求登记在`FromTextRequest::LoadClipboard`和`FromTextRequest::LoadFile`，
/// 由下一帧的`logic()`实际处理；
/// 纯内存改动（勾选、重置本页、输出目录）在ui侧直接改好。
pub fn show(
  ui: &mut Ui,
  state: &mut FromTextState,
  sources: &mut PasswordSources,
  f_ctx: FrameCtx,
) -> Vec<AppRequest> {
  let mut requests = Vec::new();

  Label::new(DESC_C1S2).selectable(false).ui(ui);
  ui.add_space(16.0);

  Grid::new("ID_GRID_C1S2")
    .num_columns(2)
    .spacing([20.0, 5.0])
    .striped(true)
    .show(ui, |ui| {
      // 第1行（两个来源按钮+重置按钮+路径框；因为与常规`file_row`不同，所以单独写）
      Label::new("密码输入来源")
        .selectable(false)
        .ui(ui)
        .on_hover_text("《黄金太阳》的密码文本的输入来源。");
      ui.horizontal(|ui| {
        if ui.add(clipboard_btn(f_ctx.svg_icon_size)).clicked() {
          // 登记从剪贴板读取密码文本并解析的请求，由下一帧的`logic()`实际进行处理
          state.request(FromTextRequest::LoadClipboard);
        }

        if ui.add(select_file_btn(f_ctx.svg_icon_size)).clicked() {
          if let Some(file) = FileDialog::new().pick_file() {
            // 登记读取密码文本文件并解析的请求，由下一帧的`logic()`实际进行处理
            state.request(FromTextRequest::LoadFile(file));
          }
        }

        if sources.from_text.is_none() {
          ui.add_enabled(false, reset_btn(f_ctx.svg_icon_size));
        } else if ui.add(reset_btn(f_ctx.svg_icon_size)).clicked() {
          state.reset(sources);

          // `密码文本`的【重置】要联动`注入密码`的密码源，因此发送重置`注入密码`中密码源选择为`密码文本`的请求，由下一帧的`App::logic()`实际进行处理
          requests.push(AppRequest::ResetPasswordSources(SubCommandEnum::Second));
        }
        ui.add(
          TextEdit::singleline(&mut state.input_file_path)
            .interactive(false)
            .hint_text("密码文本文件路径")
            .desired_width(f32::INFINITY),
        );
      });
      ui.end_row();
      // 第2行
      combo_row(
        ui,
        "密码级别",
        "目标密码级别（仅用于降级），指定级别不可高于输入文件的级别。若两者级别相同则忽略；若可降级则生成降级后的密码文本文件（及相关文件）；若留空，则目标级别同输入文件。",
        sources.has_text(),
        "ID_CB_C1S2_密码级别",
        match &state.grade {
          Some(password_grade_enum) => password_grade_enum.short_name(),
          None => "无",
        },
        |ui| {
          ui.selectable_value(&mut state.grade, None, "无");
          if let Some(source) = &sources.from_text {
            for ele in PasswordGrade::ALL {
              ui.add_enabled_ui(source.grade.priority() < ele.priority(), |ui| {
                ui.selectable_value(&mut state.grade, Some(ele), ele.short_name());
              });
            }
          }
        },
      );
      // 第3行
      checkbox_row(
        ui,
        "转换密码版本",
        "若勾选，则自动检测输入的密码文本的版本，并转换输出另外一个版本的密码文本文件，即日转英，英转日。",
        sources.has_text(),
        &mut state.to_convert,
      );
      // 第4行
      checkbox_row(
        ui,
        "生成内存转储文件",
        "若勾选，则生成密码的内存转储文件",
        sources.has_text(),
        &mut state.export_dump,
      );
      // 第5行
      combo_row(
        ui,
        "生成密码金手指",
        "若选择了值，则生成指定版本的金手指文本文件",
        sources.has_text(),
        "ID_CB_C1S2_生成密码金手指",
        match &state.cheat_region {
          Some(version_enum) => version_enum.short_name(),
          None => "无",
        },
        |ui| {
          ui.selectable_value(&mut state.cheat_region, None, "无");
          for ele in GameRegion::ALL {
            ui.selectable_value(&mut state.cheat_region, Some(ele), ele.short_name());
          }
        },
      );
      // 第6行
      pwd_info_checkbox_row(
        ui,
        "导出密码数据至文本文件",
        "若勾选，则导出适用于 Dyrati 的“Golden Sun Password Generator”表格的密码数据文本文件",
        sources.has_text(),
        &mut state.export_data_text,
        SubCommandEnum::Second,
        f_ctx,
        &mut requests
      );
      // 第7行
      match dir_row(
        ui,
        "指定输出文件保存目录",
        "自定义输出文件夹（推荐留空），若留空则在输入文件的同目录下创建文件夹保存生成的所有文件。",
        "若留空，则输出目录与输入的密码文本文件所在目录相同",
        &mut state.out_dir,
        f_ctx.svg_icon_size,
      ) {
        PathAction::Picked(dir) => state.out_dir = dir.to_string_lossy().into_owned(),
        PathAction::Reset => state.out_dir.clear(),
        PathAction::None => {},
      };
    });

  requests
}
