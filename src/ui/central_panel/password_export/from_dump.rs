//! ③ 内存转储子页视图：ui侧只负责画widget、登记下一帧logic侧要处理的request。

use egui::{Checkbox, ComboBox, Grid, Label, RichText, TextEdit, Ui, Widget};
use rfd::FileDialog;

use crate::{
  core::enums::{GameRegion, PasswordGrade, PasswordVersion},
  logic::command::SubCommandEnum,
  logic::password_export::from_dump::{FromDumpRequest, FromDumpState},
  logic::sources::PasswordSources,
  ui::page::{AppRequest, FrameCtx},
  widgets::buttons::{pwd_info_btn, reset_btn, select_dir_btn, select_file_btn},
};

const DESC_C1S3: &str = r"“密码级别”，“密码文本版本”，“生成密码金手指”，“导出密码数据至文本文件”。
以上四项虽然都是可以留空或者不勾选的可选选项，但是必须要有其中一项有选择值或者已勾选。";

/// 画【③ 内存转储】子页；返回需要 `App` 处理的跨功能请求（本页自己的待办已登记进 `state`）
///
/// IO操作作为请求登记在`FromDumpRequest::Load`，
/// 由下一帧的`logic()`实际处理；
/// 纯内存改动（勾选、重置本页、输出目录）在ui侧直接改好。
pub fn show(
  ui: &mut Ui,
  state: &mut FromDumpState,
  sources: &mut PasswordSources,
  ctx: FrameCtx,
) -> Vec<AppRequest> {
  let mut requests = Vec::new();
  let svg_icon_size = ctx.svg_icon_size;

  Label::new(DESC_C1S3).selectable(false).ui(ui);

  ui.add_space(16.0);

  Grid::new("ID_GRID_C1S3")
    .num_columns(2)
    .spacing([20.0, 5.0])
    .striped(true)
    .show(ui, |ui| {
      // 第1行
      Label::new("输入文件路径")
        .selectable(false)
        .ui(ui)
        .on_hover_text("黄金太阳的密码内存转储文件");
      ui.horizontal(|ui| {
        if ui.add(select_file_btn(svg_icon_size)).clicked() {
          if let Some(file) = FileDialog::new().pick_file() {
            // 登记读取密码内存转储文件并解析的请求，由下一帧的`logic()`实际进行处理
            state.request(FromDumpRequest::Load(file));
          }
        }
        if state.input_file_path.is_empty() {
          ui.add_enabled(false, reset_btn(svg_icon_size));
        } else if ui.add(reset_btn(svg_icon_size)).clicked() {
          state.reset(sources);

          // `内存转储`的【重置】要联动`注入密码`的密码源，因此发送重置`注入密码`中密码源选择为`内存转储`的请求，由下一帧的`App::logic()`实际进行处理
          requests.push(AppRequest::ResetPasswordSources(SubCommandEnum::Third));
        }
        ui.add(
          TextEdit::singleline(&mut state.input_file_path)
            .interactive(false)
            .hint_text("内存转储文件路径")
            .desired_width(f32::INFINITY),
        );
      });
      ui.end_row();
      // 第2行
      if let Some(source_grade) = sources.dmp_grade() {
        Label::new("密码级别").selectable(false).ui(ui).on_hover_text("目标密码级别（仅用于降级），指定级别不可高于输入文件的级别。若两者级别相同则忽略；若可降级则生成降级后的密码文本文件（及相关文件）；若留空，则目标级别同输入文件。");
        ComboBox::from_id_salt("ID_CB_C1S3_密码级别")
          .width(100.0)
          .selected_text(match &state.grade {
            Some(password_grade_enum) => password_grade_enum.short_name(),
            None => "无",
          })
          .show_ui(ui, |ui| {
            ui.selectable_value(&mut state.grade, None, "无");
            for ele in PasswordGrade::ALL {
              ui.add_enabled_ui(source_grade.priority() < ele.priority(), |ui| {
                ui.selectable_value(&mut state.grade, Some(ele), ele.short_name());
              });
            }
          });
      } else {
        Label::new(RichText::new("密码级别").weak()).selectable(false).ui(ui).on_hover_text("目标密码级别（仅用于降级），指定级别不可高于输入文件的级别。若两者级别相同则忽略；若可降级则生成降级后的密码文本文件（及相关文件）；若留空，则目标级别同输入文件。");
        ui.add_enabled_ui(false, |ui| {
          ComboBox::from_id_salt("ID_CB_C1S3_密码级别（不可用）")
            .width(100.0)
            .selected_text("无")
            .show_ui(ui, |_ui| {});
        });
      }
      ui.end_row();
      // 第3行
      if sources.has_dump() {
        Label::new("密码文本版本").selectable(false).ui(ui).on_hover_text("若选择了值，则生成指定版本的密码文本文件。");
        ComboBox::from_id_salt("ID_CB_C1S3_密码文本版本")
          .width(100.0)
          .selected_text(match &state.text_version {
            Some(password_version_enum) => password_version_enum.name(),
            None => "无",
          })
          .show_ui(ui, |ui| {
            ui.selectable_value(&mut state.text_version, None, "无");
            for ele in PasswordVersion::ALL {
              ui.selectable_value(&mut state.text_version, Some(ele), ele.name());
            }
          });
      } else {
        Label::new(RichText::new("密码文本版本").weak()).selectable(false).ui(ui).on_hover_text("若选择了值，则生成指定版本的密码文本文件。");
        ui.add_enabled_ui(false, |ui| {
          ComboBox::from_id_salt("ID_CB_C1S3_密码文本版本（不可用）")
            .width(100.0)
            .selected_text("无")
            .show_ui(ui, |_ui| {});
        });
      }
      ui.end_row();
      // 第4行
      if sources.has_dump() {
        Label::new("生成密码金手指").selectable(false).ui(ui).on_hover_text("若选择了值，则生成指定版本的金手指文本文件。");
        ComboBox::from_id_salt("ID_CB_C1S3_生成密码金手指")
          .width(100.0)
          .selected_text(match &state.cheat_region {
            Some(version_enum) => version_enum.short_name(),
            None => "无",
          })
          .show_ui(ui, |ui| {
            ui.selectable_value(&mut state.cheat_region, None, "无");
            for ele in GameRegion::ALL {
              ui.selectable_value(&mut state.cheat_region, Some(ele), ele.short_name());
            }
          });
      } else {
        Label::new(RichText::new("生成密码金手指").weak()).selectable(false).ui(ui).on_hover_text("若选择了值，则生成指定版本的金手指文本文件。");
        ui.add_enabled_ui(false, |ui| {
          ComboBox::from_id_salt("ID_CB_C1S3_生成密码金手指（不可用）")
            .width(100.0)
            .selected_text("无")
            .show_ui(ui, |_ui| {});
        });
      }
      ui.end_row();
      // 第5行
      if sources.has_dump() {
        Label::new("导出密码数据至文本文件").selectable(false).ui(ui).on_hover_text("若勾选，则导出适用于 Dyrati 的“Golden Sun Password Generator”表格的密码数据文本文件。");
        ui.horizontal(|ui| {
          Checkbox::without_text(&mut state.export_data_text).ui(ui);
          if ui.add(pwd_info_btn(svg_icon_size)).clicked() {
            // 发送打开密码信息预览窗口的请求（以`内存转储`为数据源）
            requests.push(AppRequest::OpenPasswordPreview(SubCommandEnum::Third));
          }
        });
      } else {
        Label::new(RichText::new("导出密码数据至文本文件").weak()).selectable(false).ui(ui).on_hover_text("若勾选，则导出适用于 Dyrati 的“Golden Sun Password Generator”表格的密码数据文本文件。");
        ui.horizontal(|ui| {
          ui.add_enabled(false, Checkbox::without_text(&mut state.export_data_text));
          ui.add_enabled(false, pwd_info_btn(svg_icon_size));
        });
      }
      ui.end_row();
      // 第6行
      Label::new("指定输出文件保存目录").selectable(false).ui(ui).on_hover_text("自定义输出文件夹（推荐留空），若留空则在输入文件的同目录下创建文件夹保存生成的所有文件。");
      ui.horizontal(|ui| {
        if ui.add(select_dir_btn(svg_icon_size)).clicked() {
          if let Some(dir) = FileDialog::new().pick_folder() {
            state.out_dir = dir.to_string_lossy().into_owned();
          }
        }
        if state.out_dir.is_empty() {
          ui.add_enabled(false, reset_btn(svg_icon_size));
        } else if ui.add(reset_btn(svg_icon_size)).clicked() {
          state.out_dir.clear();
        }
        ui.add(
          TextEdit::singleline(&mut state.out_dir)
            .interactive(false)
            .hint_text("若留空，则输出目录与输入的内存转储文件所在目录相同")
            .desired_width(f32::INFINITY),
        );
      });
      ui.end_row();
    });

  requests
}
