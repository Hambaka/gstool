//! `存档转换`功能的视图：说明+视图+日志区。
//!
//! 只画widget、登记自己状态的request，不做任何实际的IO操作。

use egui::{Label, Ui, Widget};

use crate::{
  core::enums::{GameRegion, GameTitle, NameType},
  logic::save_convert::{SaveConvertRequest, SaveConvertState},
  ui::{
    page::{AppRequest, FrameCtx},
    widgets::rows::{PathAction, combo_row, dir_row, file_row},
  },
};

const DESC_C2: &str = r"使用该功能可转换《黄金太阳 开启的封印》或《黄金太阳 失落的时代》存档的版本：
1. 修改主角团所有成员的名字为指定语言版本的默认名
2. 通过修改存档中的游戏构建日期来转换存档版本

注：
1. 如果存档中的构建日期与游戏 ROM 中的构建日期不匹配，游戏将强制玩家从神殿开始游戏。
2. Mobile&elffinal/SOMA 的《黄金太阳 失落的时代》汉化版基于美版制作，所以 ROM 中的构建日期同美版。
3. 《黄金太阳 开启的封印》新汉化版基于日版制作，所以 ROM 中的构建日期同日版。";

/// 画`存档转换`功能页（说明+视图）；
///
/// 因为这一页只有自己状态的请求而没有跨功能的请求，
/// 所以返回值是空的Vec。
pub fn show(ui: &mut Ui, state: &mut SaveConvertState, f_ctx: FrameCtx) -> Vec<AppRequest> {
  // 说明文本
  Label::new(DESC_C2).selectable(false).ui(ui);
  ui.add_space(16.0);

  egui::Grid::new("ID_GRID_C2")
    .num_columns(2)
    .spacing([40.0, 4.0])
    .striped(true)
    .show(ui, |ui| {
      let enabled = state.source.is_some();

      // 第1行 -> 输入文件路径
      match file_row(
        ui,
        "输入文件路径",
        "《黄金太阳 开启的封印》或《黄金太阳 失落的时代》的存档文件",
        "存档文件路径",
        &mut state.input_file_path,
        f_ctx.svg_icon_size,
      ) {
        PathAction::Picked(file) => state.request(SaveConvertRequest::Load(file)),
        // 因为是纯内存改动，直接ui侧动就行
        PathAction::Reset => state.reset(),
        PathAction::None => {},
      }

      // 第2行 -> 名字版本
      combo_row(
        ui,
        "名字版本",
        "若选择了值，则将存档中的主角团所有成员的名字转换至指定版本的默认名；若留空则不修改。",
        enabled,
        "ID_CB_C2_名字版本",
        match &state.default_name_ver {
          Some(name_type) => name_type.name(),
          None => "无",
        },
        |ui| {
          ui.selectable_value(&mut state.default_name_ver, None, "无");

          for ele in NameType::ALL {
            match ele {
              NameType::ChineseFanTranslation2023Team => {
                if let Some(game_info) = &state.source {
                  ui.add_enabled_ui(matches!(game_info.title, GameTitle::TheBrokenSeal), |ui| {
                    ui.selectable_value(&mut state.default_name_ver, Some(ele), ele.name());
                  });
                }
              },
              NameType::ChineseFanTranslationMobileTeam => {
                if let Some(game_info) = &state.source {
                  ui.add_enabled_ui(matches!(game_info.title, GameTitle::TheLostAge), |ui| {
                    ui.selectable_value(&mut state.default_name_ver, Some(ele), ele.name());
                  });
                }
              },
              _ => {
                ui.selectable_value(&mut state.default_name_ver, Some(ele), ele.name());
              },
            }
          }
        },
      );

      // 第3行 -> 构建日期版本
      combo_row(
        ui,
        "构建日期版本",
        "若选择了值，则将存档中的构建日期转换至指定版本的构建日期；若留空则不修改。",
        enabled,
        "ID_CB_C2_构建日期版本",
        match &state.build_date_ver {
          Some(version_enum) => version_enum.short_name(),
          None => "无",
        },
        |ui| {
          ui.selectable_value(&mut state.build_date_ver, None, "无");
          if let Some(game_info) = &state.source {
            for ele in GameRegion::ALL {
              ui.add_enabled_ui(ele != game_info.region, |ui| {
                ui.selectable_value(&mut state.build_date_ver, Some(ele), ele.short_name());
              });
            }
          }
        },
      );

      // 第4行 -> 指定输出文件保存目录
      match dir_row(
        ui,
        "指定输出文件保存目录",
        "自定义输出的存档文件路径（推荐留空），若留空则将转换后的存档文件保存到输入文件的同目录下。",
        "存档文件输出路径",
        &mut state.out_dir,
        f_ctx.svg_icon_size,
      ) {
        PathAction::Picked(dir) => state.out_dir = dir.to_string_lossy().into_owned(),
        PathAction::Reset => state.out_dir.clear(),
        PathAction::None => {},
      };
    });

  // 因为无跨功能请求，所以是空的Vec
  Vec::new()
}

/// 显示输出日志（概要）
pub fn show_log(ui: &mut Ui, state: &SaveConvertState) {
  Label::new(state.log()).selectable(false).ui(ui);
}
