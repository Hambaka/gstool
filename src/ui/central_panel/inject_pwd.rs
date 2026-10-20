use egui::{ComboBox, Label, RichText, Ui, Widget};

use crate::{
  app::AtomsEnum,
  core::enums::PasswordGrade,
  logic::{
    command::SubCommandEnum,
    inject_pwd::{InjectPwdRequest, InjectPwdState, SourceChoice},
    save_slot::SaveSlotEnum,
    sources::PasswordSources,
  },
  ui::page::{AppRequest, FrameCtx},
  widgets::{
    atoms::sub_command_atoms,
    buttons::save_slot_atoms,
    rows::{PathAction, dir_row, file_row},
  },
};

const DESC_C5: &str = r"使用该功能可为《黄金太阳 失落的时代》注入能够自动输入密码的程序。
支持的版本：
1. 官方版本ROM（日/美/德/西/法/意）
2. 流传度最高的中文汉化版ROM（SOMA汉化组/Mobile+elffinal）

注：
1. 游戏版本的来源多种多样，无法一一进行测试。";

const PWD_SRC_LABEL: [&str; 3] = ["金密码来源", "银密码来源", "铜密码来源"];
const PWD_SRC_DESC: [&str; 3] = [
  "自动输入金密码所使用的密码数据来源。",
  "自动输入银密码所使用的密码数据来源。",
  "自动输入铜密码所使用的密码数据来源。",
];

/// 画“注入密码”功能页（说明+视图）；
///
/// 因为这一页只有自己状态的请求而没有跨功能的请求，
/// 所以返回值是空的Vec。
pub fn show(
  ui: &mut Ui,
  state: &mut InjectPwdState,
  sources: &PasswordSources,
  ctx: FrameCtx,
) -> Vec<AppRequest> {
  let svg_icon_size = ctx.svg_icon_size;
  let is_dark = ctx.is_dark;

  Label::new(DESC_C5).selectable(false).ui(ui);
  ui.add_space(16.0);

  egui::Grid::new("ID_GRID_C5")
    .num_columns(2)
    .spacing([40.0, 4.0])
    .striped(true)
    .show(ui, |ui| {
      // 第1行 -> 输入文件路径
      match file_row(
        ui,
        "输入文件路径",
        "《黄金太阳 失落的时代》的ROM文件",
        "ROM文件路径",
        &mut state.input_file_path,
        svg_icon_size,
      ) {
        PathAction::Picked(file) => state.request(InjectPwdRequest::Load(file)),
        PathAction::Reset => state.reset(),
        PathAction::None => {},
      }

      // 第2~4行 -> 金/银/铜密码来源
      for i in 0..3 {
        let data_is_available = sources.has_save() || sources.has_text() || sources.has_dump();

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
                match &state.source[i] {
                  Some(choice) => AtomsEnum::ImageAndText(sub_command_atoms(choice.kind, svg_icon_size, is_dark)),
                  None => AtomsEnum::Text("无"),
                }
              })
              .show_ui(ui, |ui| {
                // “无”选项
                ui.selectable_value(&mut state.source[i], None, "无");

                // “游戏存档”选项
                ui.add_enabled_ui(sources.has_save(), |ui| {
                  ui.selectable_value(
                    &mut state.source[i],
                    Some(SourceChoice { kind: SubCommandEnum::First, slot: sources.first_available_slot() }),
                    sub_command_atoms(SubCommandEnum::First, svg_icon_size, is_dark)
                    );
                });

                // “密码文本”选项(+ 数据源级别检测)
                let bool_txt = match i {
                  0 => sources.txt_grade() == Some(PasswordGrade::Gold),
                  1 => sources.txt_grade() != Some(PasswordGrade::Bronze),
                  _ => true,
                };
                ui.add_enabled_ui(sources.has_text() && bool_txt, |ui| {
                  ui.selectable_value(
                    &mut state.source[i],
                    Some(SourceChoice { kind: SubCommandEnum::Second, slot: None }),
                    sub_command_atoms(SubCommandEnum::Second, svg_icon_size, is_dark)
                    );
                });

                // “内存转储”选项(+ 数据源级别检测)
                let bool_dmp = match i {
                  0 => sources.dmp_grade() == Some(PasswordGrade::Gold),
                  1 => sources.dmp_grade() != Some(PasswordGrade::Bronze),
                  _ => true,
                };
                ui.add_enabled_ui(sources.has_dump() && bool_dmp, |ui| {
                  ui.selectable_value(
                    &mut state.source[i],
                    Some(SourceChoice { kind: SubCommandEnum::Third, slot: None }),
                    sub_command_atoms(SubCommandEnum::Third, svg_icon_size, is_dark)
                  );
                });
              });

            // 密码数据源为存档时，显示存档槽位下拉框（即 kind == First 时显示）
            if state.source[i].map_or(false, |c| c.kind == SubCommandEnum::First) {
              ComboBox::from_id_salt(format!("ID_CB_C5_{}_2", PWD_SRC_LABEL[i]))
                .width(100.0)
                .selected_text(match &state.source[i] {
                  Some(choice) => match choice.slot {
                    Some(save_slot) => AtomsEnum::ImageAndText(save_slot_atoms(save_slot, svg_icon_size, is_dark, false)),
                    None => AtomsEnum::Text("无"),
                  },
                  None => AtomsEnum::Text("无"),
                })
                .show_ui(ui, |ui| {
                  for j in 0..3 {
                    ui.add_enabled_ui(sources.from_save.slots[j].is_some(), |ui| {
                      ui.selectable_value(
                        &mut state.source[i].as_mut().unwrap().slot,
                        Some(SaveSlotEnum::SAVE_SLOTS[j]),
                        save_slot_atoms(SaveSlotEnum::SAVE_SLOTS[j], svg_icon_size, is_dark, false),
                      );
                    });
                  }
                });
            }
          });
        }

        ui.end_row();
      }

      // 第5行 -> 指定输出文件保存目录
      match dir_row(
        ui,
        "指定输出文件保存目录",
        "自定义输出的注入密码版ROM文件路径（推荐留空），若留空则将转换后的ROM文件保存到输入文件的同目录下。",
        "最新版注入密码ROM输出路径",
        &mut state.out_dir,
        svg_icon_size,
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
pub fn show_log(ui: &mut Ui, state: &InjectPwdState) {
  Label::new(state.log()).selectable(false).ui(ui);
}
