//! ① 游戏存档子页视图：ui侧只负责画widget、登记下一帧logic侧要处理的request。

use egui::{Checkbox, Grid, Label, Ui, Widget};

use crate::{
  core::enums::{GameRegion, PasswordGrade, PasswordVersion},
  logic::command::SubCommandEnum,
  logic::password_export::from_save::{FromSaveRequest, FromSaveState},
  logic::sources::PasswordSources,
  ui::page::{AppRequest, FrameCtx},
  widgets::{
    buttons::pwd_info_btn,
    rows::{PathAction, checkbox_row, combo_row, dir_row, file_row, row_label},
  },
};

const DESC_C1S1: &str = r"“生成内存转储文件”，“生成密码金手指”，“导出密码数据至文本文件”。
程序默认密码级别为“金”，密码文本版本为“英文”，因此以上三项都是可以留空或者不勾选的可选选项。";

/// 画【① 游戏存档】子页；返回需要 `App` 处理的跨功能请求（本页自己的待办已登记进 `state`）
///
/// IO操作作为请求登记在`FromSaveRequest::Load`，
/// 由下一帧的`logic()`实际处理；
/// 纯内存改动（勾选、重置本页、输出目录）在ui侧直接改好。
pub fn show(
  ui: &mut Ui,
  state: &mut FromSaveState,
  sources: &mut PasswordSources,
  ctx: FrameCtx,
) -> Vec<AppRequest> {
  let mut requests = Vec::new();
  let svg_icon_size = ctx.svg_icon_size;

  Label::new(DESC_C1S1).selectable(false).ui(ui);

  ui.add_space(16.0);

  Grid::new("ID_GRID_C1S1")
    .num_columns(2)
    .spacing([20.0, 5.0])
    .striped(true)
    .show(ui, |ui| {
      // 第1行
      row_label(
        ui,
        "存档导出选项",
        "根据需求进行勾选。",
        state.input_file_path.is_empty(),
      );
      ui.add_enabled_ui(state.input_file_path.is_empty(), |ui| {
        ui.radio_value(&mut state.export_all_saves, false, "仅导出通关存档");
        ui.radio_value(
          &mut state.export_all_saves,
          true,
          "导出全部存档（无论是否通关）",
        );
      });
      ui.end_row();
      // 第2行
      match file_row(
        ui,
        "输入文件路径",
        "《黄金太阳 开启的封印》的存档文件。",
        "存档文件路径",
        &mut state.input_file_path,
        svg_icon_size,
      ) {
        // 登记读取存档文件并解析的请求，由下一帧的`logic()`实际进行处理
        PathAction::Picked(file) => state.request(FromSaveRequest::Load(file)),
        PathAction::Reset => {
          state.reset(sources);
          // `存档文件`的【重置】要联动`注入密码`的密码源，因此发送重置`注入密码`中密码源选择为`存档文件`的请求，由下一帧的`App::logic()`实际进行处理
          requests.push(AppRequest::ResetPasswordSources(SubCommandEnum::First));
        }
        PathAction::None => {}
      }

      // 第3行
      combo_row(
        ui,
        "密码级别",
        "目标密码级别，默认值为“金”",
        sources.has_save(),
        "ID_CB_C1S1_密码级别",
        state.grade.short_name(),
        |ui| {
          for ele in PasswordGrade::ALL {
            ui.selectable_value(&mut state.grade, ele, ele.short_name());
          }
        },
      );

      // 第4行
      combo_row(
        ui,
        "密码文本版本",
        "生成指定版本的密码文本文件，若留空则自动指定默认值“英文”。",
        sources.has_save(),
        "ID_CB_C1S1_密码文本版本",
        match &state.text_version {
          Some(password_version_enum) => password_version_enum.name(),
          None => "无",
        },
        |ui| {
          ui.selectable_value(&mut state.text_version, None, "无");
          for ele in PasswordVersion::ALL {
            ui.selectable_value(&mut state.text_version, Some(ele), ele.name());
          }
        },
      );

      // 第5行
      checkbox_row(
        ui,
        "生成内存转储文件",
        "若勾选，则生成密码的内存转储文件。",
        sources.has_save(),
        &mut state.export_dump,
      );

      // 第6行
      combo_row(
        ui,
        "生成密码金手指",
        "若选择了值，则生成指定版本的金手指文本文件。",
        sources.has_save(),
        "ID_CB_C1S1_生成密码金手指",
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

      // 第7行（复选框与预览按钮同处一行，未用 checkbox_row）
      row_label(
        ui,
        "导出密码数据至文本文件",
        "若勾选，则导出适用于 Dyrati 的“Golden Sun Password Generator”表格的密码数据文本文件。",
        sources.has_save(),
      );
      ui.horizontal(|ui| {
        ui.add_enabled(
          sources.has_save(),
          Checkbox::without_text(&mut state.export_data_text),
        );
        if ui
          .add_enabled(sources.has_save(), pwd_info_btn(svg_icon_size))
          .clicked()
        {
          // 登记打开密码信息预览窗口的请求（以`存档文件`为数据源）
          requests.push(AppRequest::OpenPasswordPreview(SubCommandEnum::First, None));
        }
      });
      ui.end_row();

      // 第8行
      match dir_row(
        ui,
        "指定输出文件保存目录",
        "自定义输出文件夹（推荐留空），若留空则在输入文件的同目录下创建文件夹保存生成的所有文件。",
        "若留空，则输出目录与输入的存档文件所在目录相同",
        &mut state.out_dir,
        svg_icon_size,
      ) {
        PathAction::Picked(dir) => {
          state.out_dir = dir.to_string_lossy().into_owned();
        }
        PathAction::Reset => {
          state.out_dir.clear();
        }
        PathAction::None => {}
      }
    });

  requests
}
