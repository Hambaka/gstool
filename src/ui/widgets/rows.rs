//! 各功能页面共用的“一行”控件。
//!
//! 只**绘制**并**报告做了什么**（返回动作 / 写传入的 `&mut`），
//! 不接触任何功能的状态结构体
//!
//! 同时实现`启用`和`禁用`两套：
//! 标签在禁用时自动变灰，下拉框禁用时另画一个显示「无」的替身

use std::path::PathBuf;

use egui::{Checkbox, ComboBox, Label, RichText, TextEdit, Ui, Widget};
use rfd::FileDialog;

use crate::{
  logic::command::SubCommandEnum,
  ui::{
    page::{AppRequest, FrameCtx},
    widgets::buttons::{pwd_info_btn, reset_btn, select_dir_btn, select_file_btn},
  },
};

/// 用户对`路径选择行`（文件、目录选择）做的动作
#[derive(Debug, Clone, PartialEq)]
pub enum PathAction {
  None,
  Picked(PathBuf),
  Reset,
}

/// 行首的文字标签：`enabled = false` 时标签自动变灰，提示文本不变
pub fn row_label(ui: &mut Ui, text: &str, hint: &str, enabled: bool) {
  let label = if enabled {
    Label::new(text)
  } else {
    Label::new(RichText::new(text).weak())
  };
  label.selectable(false).ui(ui).on_hover_text(hint);
}

/// 行末 -> 文件选择：`标签`+`【选择文件】`+`【重置】`+ `只读路径框`（自动 `end_row()`）。
///
/// 选中的文件只通过返回值 `PathAction::Picked` 报告，由调用方决定是否写入
pub fn file_row(
  ui: &mut Ui,
  label: &str,
  hint: &str,
  path_hint: &str,
  path: &mut String,
  svg_icon_size: f32,
) -> PathAction {
  row_label(ui, label, hint, true);
  let mut action = PathAction::None;

  ui.horizontal(|ui| {
    if ui.add(select_file_btn(svg_icon_size)).clicked()
      && let Some(file) = FileDialog::new().pick_file()
    {
      action = PathAction::Picked(file);
    }
    if path.is_empty() {
      ui.add_enabled(false, reset_btn(svg_icon_size));
    } else if ui.add(reset_btn(svg_icon_size)).clicked() {
      action = PathAction::Reset;
    }
    ui.add(
      TextEdit::singleline(path)
        .interactive(false)
        .hint_text(path_hint)
        .desired_width(f32::INFINITY),
    );
  });

  ui.end_row();
  action
}

/// 行末 -> 目录：`标签`+`【选择目录】`+`【重置】`+`只读路径框`（自动 `end_row()`）。
///
/// 选中的目录同样只通过返回值报告，由调用方写入。
pub fn dir_row(
  ui: &mut Ui,
  label: &str,
  hint: &str,
  path_hint: &str,
  path: &mut String,
  svg_icon_size: f32,
) -> PathAction {
  row_label(ui, label, hint, true);
  let mut action = PathAction::None;

  ui.horizontal(|ui| {
    if ui.add(select_dir_btn(svg_icon_size)).clicked()
      && let Some(dir) = FileDialog::new().pick_folder()
    {
      action = PathAction::Picked(dir);
    }
    if path.is_empty() {
      ui.add_enabled(false, reset_btn(svg_icon_size));
    } else if ui.add(reset_btn(svg_icon_size)).clicked() {
      action = PathAction::Reset;
    }
    ui.add(
      TextEdit::singleline(path)
        .interactive(false)
        .hint_text(path_hint)
        .desired_width(f32::INFINITY),
    );
  });

  ui.end_row();
  action
}

/// 行末 -> 复选框：`标签`+`复选框`（自动 `end_row()`）
pub fn checkbox_row(ui: &mut Ui, label: &str, hint: &str, enabled: bool, checked: &mut bool) {
  row_label(ui, label, hint, enabled);
  ui.add_enabled(enabled, Checkbox::without_text(checked));
  ui.end_row();
}

/// 复选框 + 密码信息预览按钮同行（用于密码导出的三个子页的"导出数据至文本"选项）。
///
/// 比`checkbox_row`多一个密码数据信息预览按钮：点击后将一个`OpenPasswordPreview`的请求push到`requests`中。
pub fn pwd_info_checkbox_row(
  ui: &mut Ui,
  label: &str,
  hint: &str,
  enabled: bool,
  checked: &mut bool,
  preview_cmd: SubCommandEnum,
  f_ctx: FrameCtx,
  requests: &mut Vec<AppRequest>,
) {
  row_label(ui, label, hint, enabled);
  ui.horizontal(|ui| {
    ui.add_enabled(enabled, Checkbox::without_text(checked));
    if ui
      .add_enabled(enabled, pwd_info_btn(f_ctx.svg_icon_size))
      .clicked()
    {
      requests.push(AppRequest::OpenPasswordPreview(preview_cmd, None));
    }
  });
  ui.end_row();
}

/// 行末 -> 下拉框：`标签`+`下拉框`（自动 `end_row()`）。
///
/// 禁用时标签变灰、下拉框显示“无”且不可交互。
/// `add_options` 里只写 `ui.selectable_value(...)`，不用管`启用`/`禁用`与id；
/// 选项本身需要再受限时（如级别不可高于源级别），在闭包里再包一层
/// `ui.add_enabled_ui(…)` 即可。
pub fn combo_row(
  ui: &mut Ui,
  label: &str,
  hint: &str,
  enabled: bool,
  combo_id: &str,
  selected_text: &str,
  add_options: impl FnOnce(&mut Ui),
) {
  row_label(ui, label, hint, enabled);

  if enabled {
    ComboBox::from_id_salt(combo_id)
      .width(100.0)
      .selected_text(selected_text)
      .show_ui(ui, add_options);
  } else {
    // 禁用时另画一个“无”的下拉框
    ui.add_enabled_ui(false, |ui| {
      ComboBox::from_id_salt(format!("{combo_id}（不可用）"))
        .width(100.0)
        .selected_text("无")
        .show_ui(ui, |_ui| {});
    });
  }

  ui.end_row();
}
