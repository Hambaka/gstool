use egui::{Button, ComboBox, Image, Label, RichText, Widget};

use crate::{
  core::{
    data::{ALL_DJINN_NAMES, ITEM_NAMES, Language, icon_id},
    enums::{Element, PasswordGrade},
    password::{
      items::{PSYNERGY_ITEMS, STACKABLE_ITEMS},
      stats::MAX_BASE_STATS,
    },
  },
  logic::{
    custom_name::PC_NAMES,
    preview::{PreviewModeEnum, PreviewPageEnum, PwdInfoState},
    save_slot::SaveSlotEnum,
  },
  ui::{
    page::FrameCtx,
    widgets::{
      buttons::save_slot_atoms,
      colors::{DANGER, SUCCESS},
      icons::{
        CHECK_MARK, NUMBER_1_SQUARE_FILL, NUMBER_2_SQUARE_FILL, NUMBER_3_SQUARE_FILL,
        NUMBER_4_SQUARE_FILL, X_MARK,
      },
    },
  },
};

const CHAR_STATS_STR: [&str; 6] = ["HP", "EP", "攻击", "防御", "敏捷", "幸运"];
const EVENT_STR: [&str; 6] = [
  "营救哈梅特",
  "竞技大赛获胜（最终决战获胜）",
  "帮忙救助乌鲁木齐",
  "打败塔罗斯（宝岛最终Boss）",
  "重返海迪亚（获悉罗宾母亲生病）",
  "重返库普阿普（获悉小偷越狱逃跑）",
];
const EVENT_GAMEPLAY_ORDER: [usize; 6] = [2, 5, 4, 1, 0, 3];
const PSYNERGY_ITEM_GAMEPLAY_ORDER: [usize; 8] = [7, 0, 1, 2, 3, 4, 5, 6];

/// 显示“密码数据预览”窗口。
pub fn show_pwd_info_window(
  ui: &mut egui::Ui,
  preview: &mut PwdInfoState,
  item_icon_img: &[egui::TextureHandle],
  f_ctx: FrameCtx,
) {
  egui::Window::new("密码数据预览")
    .open(&mut preview.window_open)
    .resizable(false)
    .default_width(450.0)
    .show(ui, |ui| {
      // 槽位选择
      ui.columns_const(|columns: &mut [egui::Ui; 3]| {
        for (col, slot) in columns.iter_mut().zip(SaveSlotEnum::ALL) {
          if preview.data[slot.index()].is_some() {
            col.selectable_value(&mut preview.slot, slot, save_slot_atoms(slot, f_ctx, true));
          } else {
            col.add_enabled(
              false,
              Button::selectable(preview.slot == slot, save_slot_atoms(slot, f_ctx, true)),
            );
          }
        }
      });

      ui.separator();

      // 页签选择
      ui.columns_const(|columns: &mut [egui::Ui; 4]| {
        for (i, (col, img)) in columns
          .iter_mut()
          .zip([
            NUMBER_1_SQUARE_FILL,
            NUMBER_2_SQUARE_FILL,
            NUMBER_3_SQUARE_FILL,
            NUMBER_4_SQUARE_FILL,
          ])
          .enumerate()
        {
          col.selectable_value(
            &mut preview.page,
            PreviewPageEnum::ALL[i],
            (
              Image::new(img)
                .tint(f_ctx.svg_color())
                .max_height(f_ctx.svg_icon_size),
              PreviewPageEnum::ALL[i].name(),
            ),
          );
        }
      });

      // 按页签渲染
      if let Some(data) = preview.data[preview.slot.index()].as_ref() {
        match preview.page {
          // 角色
          PreviewPageEnum::Character => {
            ui.horizontal(|ui| {
              ui.radio_value(
                &mut preview.stats_mode,
                PreviewModeEnum::Main,
                "基础能力状态属性",
              );
              ui.radio_value(
                &mut preview.stats_mode,
                PreviewModeEnum::Alt,
                "能力状态属性",
              );
            });
            ui.add_space(8.0);
            ui.columns_const(|columns: &mut [egui::Ui; 5]| {
              for (col, name) in
                columns
                  .iter_mut()
                  .zip(["", PC_NAMES[0], PC_NAMES[1], PC_NAMES[2], PC_NAMES[3]])
              {
                Label::new(name).selectable(false).ui(col);
              }
            });
            ui.add_space(4.0);
            egui::Grid::new("preview_stats")
              .striped(true)
              .num_columns(5)
              .min_col_width((ui.available_width() - ui.spacing().item_spacing.x * 4.0) / 5.0)
              .show(ui, |ui| {
                Label::new(RichText::new("等级").strong())
                  .selectable(false)
                  .ui(ui);

                for i in 0..4 {
                  Label::new(format!("{}", data.levels(i)))
                    .selectable(false)
                    .ui(ui);
                }
                ui.end_row();
                if preview.char_mode == PreviewModeEnum::Main {
                  for i in 0..6 {
                    Label::new(RichText::new(CHAR_STATS_STR[i]).strong())
                      .selectable(false)
                      .ui(ui);
                    for j in 0..4 {
                      let stats_string = match preview.stats_mode {
                        PreviewModeEnum::Main => {
                          let value = data.base_stats(j, i);
                          if value > MAX_BASE_STATS[i] {
                            let real_value = match i {
                              2..=4 => value & MAX_BASE_STATS[i],
                              _ => MAX_BASE_STATS[i],
                            };
                            format!("{value} ({real_value})")
                          } else {
                            format!("{value}")
                          }
                        }
                        PreviewModeEnum::Alt => format!("{}", data.stats(j, i)),
                      };
                      Label::new(stats_string).selectable(false).ui(ui);
                    }
                    ui.end_row();
                  }
                }
              });
          }
          // 物品
          PreviewPageEnum::Item => {
            ui.horizontal(|ui| {
              Label::new("物品名称")
                .selectable(false)
                .ui(ui)
                .on_hover_text("默认值为“1代汉化”");
              ComboBox::from_id_salt("物品名语言版本")
                .width(100.0)
                .selected_text(preview.item_lang.name())
                .show_ui(ui, |ui| {
                  for lang in Language::LANGUAGES {
                    ui.selectable_value(&mut preview.item_lang, lang, lang.name());
                  }
                });
            });
            ui.add_space(8.0);
            match preview.item_mode {
              // 物品 → 全物品
              PreviewModeEnum::Main => {
                ui.columns_const(|columns: &mut [egui::Ui; 4]| {
                  for (col, name) in
                    columns
                      .iter_mut()
                      .zip([PC_NAMES[0], PC_NAMES[1], PC_NAMES[2], PC_NAMES[3]])
                  {
                    Label::new(RichText::new(name).strong())
                      .selectable(false)
                      .ui(col);
                  }
                });
                ui.add_space(4.0);
                egui::Grid::new("preview_items")
                  .striped(true)
                  .num_columns(4)
                  .min_col_width((ui.available_width() - ui.spacing().item_spacing.x * 3.0) / 4.0)
                  .show(ui, |ui| {
                    for i in 0..15 {
                      for j in 0..4 {
                        let (id, amount) = data.items(j, i);
                        if id == 0 {
                          Label::new("").selectable(false).ui(ui);
                        } else {
                          ui.horizontal(|ui| {
                            ui.add(
                              egui::Image::new(&item_icon_img[icon_id(id - 1)])
                                .fit_to_exact_size(egui::vec2(16.0, 16.0)),
                            );
                            let item_str = if STACKABLE_ITEMS.contains(&id) {
                              format!(
                                "{}×{amount}",
                                ITEM_NAMES[id as usize - 1].name(preview.item_lang)
                              )
                            } else {
                              ITEM_NAMES[id as usize - 1]
                                .name(preview.item_lang)
                                .to_string()
                            };
                            Label::new(item_str).selectable(false).ui(ui);
                          });
                        }
                      }
                      ui.end_row();
                    }
                  });
              }
              // 物品 → 精神力物品
              PreviewModeEnum::Alt => {
                for i in PSYNERGY_ITEM_GAMEPLAY_ORDER {
                  ui.horizontal(|ui| {
                    if data.psy_items(i) {
                      ui.horizontal(|ui| {
                        ui.add(
                          Image::new(CHECK_MARK)
                            .tint(SUCCESS)
                            .max_height(f_ctx.svg_icon_size),
                        );
                        ui.add(
                          egui::Image::new(
                            &item_icon_img[icon_id(i as u16 + PSYNERGY_ITEMS[0] - 1)],
                          )
                          .fit_to_exact_size(egui::vec2(16.0, 16.0)),
                        );
                        Label::new(
                          ITEM_NAMES[i + PSYNERGY_ITEMS[0] as usize - 1].name(preview.item_lang),
                        )
                        .selectable(false)
                        .ui(ui);
                      });
                    } else {
                      ui.horizontal(|ui| {
                        ui.add(
                          Image::new(X_MARK)
                            .tint(DANGER)
                            .max_height(f_ctx.svg_icon_size),
                        );
                        ui.add(
                          egui::Image::new(
                            &item_icon_img[icon_id(i as u16 + PSYNERGY_ITEMS[0] - 1)],
                          )
                          .fit_to_exact_size(egui::vec2(16.0, 16.0)),
                        );
                        Label::new(
                          RichText::new(
                            ITEM_NAMES[i + PSYNERGY_ITEMS[0] as usize - 1].name(preview.item_lang),
                          )
                          .weak()
                          .strikethrough(),
                        )
                        .selectable(false)
                        .ui(ui);
                      });
                    }
                  });
                }
              }
            }
          }
          // 精灵
          PreviewPageEnum::Djinn => {
            ui.horizontal(|ui| {
              Label::new("精灵名称")
                .selectable(false)
                .ui(ui)
                .on_hover_text("默认值为“1代汉化”");
              ComboBox::from_id_salt("精灵名语言版本")
                .width(100.0)
                .selected_text(preview.djinn_lang.name())
                .show_ui(ui, |ui| {
                  for ele in Language::LANGUAGES {
                    ui.selectable_value(&mut preview.djinn_lang, ele, ele.name());
                  }
                });
            });
            ui.add_space(8.0);
            ui.columns_const(|columns: &mut [egui::Ui; 4]| {
              for (col, element) in columns.iter_mut().zip(Element::GAMEPLAY_ORDER) {
                Label::new(RichText::new(element.name()).strong())
                  .selectable(false)
                  .ui(col);
              }
            });
            ui.add_space(4.0);
            egui::Grid::new("preview_djinns")
              .striped(true)
              .num_columns(4)
              .min_col_width((ui.available_width() - ui.spacing().item_spacing.x * 3.0) / 4.0)
              .show(ui, |ui| {
                for j in 0..7 {
                  for i in 0..4 {
                    if data.djinn(i, j) {
                      ui.horizontal(|ui| {
                        ui.add(
                          Image::new(CHECK_MARK)
                            .tint(SUCCESS)
                            .max_height(f_ctx.svg_icon_size),
                        );
                        Label::new(ALL_DJINN_NAMES[i][j].name(preview.djinn_lang))
                          .selectable(false)
                          .ui(ui);
                      });
                    } else {
                      ui.horizontal(|ui| {
                        ui.add(
                          Image::new(X_MARK)
                            .tint(DANGER)
                            .max_height(f_ctx.svg_icon_size),
                        );
                        Label::new(
                          RichText::new(ALL_DJINN_NAMES[i][j].name(preview.djinn_lang))
                            .weak()
                            .strikethrough(),
                        )
                        .selectable(false)
                        .ui(ui);
                      });
                    }
                  }
                  ui.end_row();
                }
              });
          }
          // 其他（金币、事件）
          PreviewPageEnum::Misc => {
            // 金级别 → 金币
            if data.grade() == PasswordGrade::Gold {
              ui.horizontal(|ui| {
                Label::new("金币数").selectable(false).ui(ui);
                Label::new(format!("{}", data.coins()))
                  .selectable(false)
                  .ui(ui);
              });

              ui.separator();
            }

            // 事件（游戏顺序）
            for i in EVENT_GAMEPLAY_ORDER {
              if data.events(i) {
                ui.horizontal(|ui| {
                  ui.add(
                    Image::new(CHECK_MARK)
                      .tint(SUCCESS)
                      .max_height(f_ctx.svg_icon_size),
                  );
                  Label::new(EVENT_STR[i]).selectable(false).ui(ui);
                });
              } else {
                ui.horizontal(|ui| {
                  ui.add(
                    Image::new(X_MARK)
                      .tint(DANGER)
                      .max_height(f_ctx.svg_icon_size),
                  );
                  Label::new(RichText::new(EVENT_STR[i]).weak().strikethrough())
                    .selectable(false)
                    .ui(ui);
                });
              }
            }
          }
        }
      }
    });
}
