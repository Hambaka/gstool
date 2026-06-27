use egui::{Button, ComboBox, Label, RichText, Widget};

use crate::{
  App,
  app::SubCommandEnum,
  core::{
    data::{ALL_DJINN_NAMES, ITEM_NAMES, Language},
    enums::PasswordGrade,
    save::{
      self, BitArray, MAX_VALID_BASE_STATS_VALUES_PWD, PSYNERGY_ITEMS, PasswordInfo,
      STACKABLE_ITEMS, gen_password_info_with_raw_save_bytes, gen_password_info_with_save_data,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SaveSlotEnum {
  First,
  Second,
  Third,
}

impl SaveSlotEnum {
  pub const SAVE_SLOTS: [SaveSlotEnum; 3] = [
    SaveSlotEnum::First,
    SaveSlotEnum::Second,
    SaveSlotEnum::Third,
  ];

  pub fn get_name(&self) -> &str {
    match self {
      Self::First => "存档 ❶",
      Self::Second => "存档 ❷",
      Self::Third => "存档 ❸",
    }
  }

  pub fn from(value: usize) -> Self {
    match value {
      0 => Self::First,
      1 => Self::Second,
      _ => Self::Third,
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PreviewPageEnum {
  Character,
  Item,
  Djinn,
  Misc,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PreviewModeEnum {
  Main,
  Alt,
}

#[derive(Debug, Clone)]
pub struct PreviewData {
  grade: PasswordGrade,
  info: PasswordInfo,
}

impl PreviewData {
  pub fn new(grade: PasswordGrade, info: PasswordInfo) -> Self {
    PreviewData { grade, info }
  }

  pub fn get_grade(&self) -> PasswordGrade {
    self.grade
  }

  pub fn get_levels(&self, i: usize) -> u8 {
    self.info.get_levels()[i]
  }

  pub fn get_djinn(&self, i: usize, j: usize) -> bool {
    self.info.get_djinn()[j][i]
  }

  pub fn get_events(&self, i: usize) -> bool {
    self.info.get_events()[i]
  }

  pub fn get_base_stats(&self, i: usize, j: usize) -> u16 {
    self.info.get_base_stats()[j][i]
  }

  pub fn get_stats(&self, i: usize, j: usize) -> u16 {
    self.info.get_stats()[j][i]
  }

  pub fn get_psy_items(&self, i: usize) -> bool {
    self.info.get_psy_items()[i]
  }

  pub fn get_items(&self, i: usize, j: usize) -> (u16, u8) {
    self.info.get_items()[j][i]
  }

  pub fn get_coins(&self) -> u32 {
    self.info.get_coins()
  }
}

impl App {
  pub fn show_pwd_info_window(&mut self, ui: &mut egui::Ui) {
    egui::Window::new("密码数据预览")
      .open(&mut self.pwd_info_window_open)
      .resizable(false)
      .default_width(450.0)
      .show(ui, |ui| {
        ui.columns_const(|[col_1, col_2, col_3]| {
          if self.preview_data[0].is_some() {
            col_1.selectable_value(&mut self.preview_slot, SaveSlotEnum::First, "存档 ❶");
          } else {
            col_1.add_enabled(
              false,
              Button::selectable(self.preview_slot == SaveSlotEnum::First, "存档 ❶"),
            );
          }

          if self.preview_data[1].is_some() {
            col_2.selectable_value(&mut self.preview_slot, SaveSlotEnum::Second, "存档 ❷");
          } else {
            col_2.add_enabled(
              false,
              Button::selectable(self.preview_slot == SaveSlotEnum::Second, "存档 ❷"),
            );
          }

          if self.preview_data[2].is_some() {
            col_3.selectable_value(&mut self.preview_slot, SaveSlotEnum::Third, "存档 ❸");
          } else {
            col_3.add_enabled(
              false,
              Button::selectable(self.preview_slot == SaveSlotEnum::Third, "存档 ❸"),
            );
          }
        });

        ui.separator();

        if self.preview_current.is_some() {
          let current_data = self.preview_current.as_ref().unwrap();

          ui.columns_const(|[col_1, col_2, col_3, col_4]| {
            col_1.selectable_value(&mut self.preview_page, PreviewPageEnum::Character, "🅰 角色");
            col_2.selectable_value(&mut self.preview_page, PreviewPageEnum::Item, "🅱 物品");
            col_3.selectable_value(&mut self.preview_page, PreviewPageEnum::Djinn, "🅲 精灵");
            col_4.selectable_value(&mut self.preview_page, PreviewPageEnum::Misc, "🅳 其他");
          });

          match self.preview_page {
            PreviewPageEnum::Character => {
              ui.horizontal(|ui| {
                ui.radio_value(
                  &mut self.stats_mode,
                  PreviewModeEnum::Main,
                  "基础能力状态属性",
                );
                ui.radio_value(&mut self.stats_mode, PreviewModeEnum::Alt, "能力状态属性");
              });

              ui.columns_const(|[col_1, col_2, col_3, col_4, col_5]| {
                Label::new("").selectable(false).ui(col_1);
                Label::new(RichText::new("罗宾").strong())
                  .selectable(false)
                  .ui(col_2);
                Label::new(RichText::new("杰拉德").strong())
                  .selectable(false)
                  .ui(col_3);
                Label::new(RichText::new("伊万").strong())
                  .selectable(false)
                  .ui(col_4);
                Label::new(RichText::new("梅雅莉").strong())
                  .selectable(false)
                  .ui(col_5);
              });

              egui::Grid::new("preview_stats")
                .striped(true)
                .num_columns(5)
                .min_col_width(
                  (ui.available_width() - ui.spacing().item_spacing.x * (5 as f32 - 1.0)) / 5.0,
                )
                .show(ui, |ui| {
                  Label::new(RichText::new("等级").strong())
                    .selectable(false)
                    .ui(ui);

                  for i in 0..4 {
                    Label::new(format!("{}", current_data.get_levels(i)))
                      .selectable(false)
                      .ui(ui);
                  }
                  ui.end_row();

                  // 金、银密码
                  if self.char_page_mode == PreviewModeEnum::Main {
                    for i in 0..6 {
                      Label::new(RichText::new(CHAR_STATS_STR[i]).strong())
                        .selectable(false)
                        .ui(ui);

                      for j in 0..4 {
                        let stats_string = match self.stats_mode {
                          PreviewModeEnum::Main => {
                            let value = current_data.get_base_stats(i, j);
                            if value > MAX_VALID_BASE_STATS_VALUES_PWD[i] {
                              let real_value = match i {
                                // Attack, Defense, Agility
                                2 | 3 | 4 => value & MAX_VALID_BASE_STATS_VALUES_PWD[i],
                                _ => MAX_VALID_BASE_STATS_VALUES_PWD[i],
                              };

                              format!("{value} ({real_value})")
                            } else {
                              format!("{value}")
                            }
                          }
                          PreviewModeEnum::Alt => {
                            let value = current_data.get_stats(i, j);
                            format!("{value}")
                          }
                        };

                        Label::new(stats_string).selectable(false).ui(ui);
                      }
                      ui.end_row();
                    }
                  }
                });
            }
            PreviewPageEnum::Item => {
              ui.horizontal(|ui| {
                Label::new("物品名称")
                  .selectable(false)
                  .ui(ui)
                  .on_hover_text("默认值为“1代汉化”");
                ComboBox::from_id_salt("物品名语言版本")
                  .width(100.0)
                  .selected_text(self.item_preview_lang.get_name())
                  .show_ui(ui, |ui| {
                    for ele in Language::LANGUAGES {
                      ui.selectable_value(&mut self.item_preview_lang, ele, ele.get_name());
                    }
                  });
              });

              match self.item_page_mode {
                PreviewModeEnum::Main => {
                  ui.columns_const(|[col_1, col_2, col_3, col_4]| {
                    Label::new("罗宾").selectable(false).ui(col_1);
                    Label::new("杰拉德").selectable(false).ui(col_2);
                    Label::new("伊万").selectable(false).ui(col_3);
                    Label::new("梅雅莉").selectable(false).ui(col_4);
                  });

                  egui::Grid::new("preview_items")
                    .striped(true)
                    .num_columns(4)
                    .min_col_width(
                      (ui.available_width() - ui.spacing().item_spacing.x * (4 as f32 - 1.0)) / 4.0,
                    )
                    .show(ui, |ui| {
                      for i in 0..15 {
                        for j in 0..4 {
                          let (id, amount) = current_data.get_items(i, j);
                          let value = if id == 0 {
                            ""
                          } else {
                            if STACKABLE_ITEMS.contains(&id) {
                              &format!(
                                "{}×{amount}",
                                ITEM_NAMES[id as usize - 1].get_name(self.item_preview_lang)
                              )
                            } else {
                              &format!(
                                "{}",
                                ITEM_NAMES[id as usize - 1].get_name(self.item_preview_lang)
                              )
                            }
                          };
                          Label::new(value).selectable(false).ui(ui);
                        }
                        ui.end_row();
                      }
                    });
                }
                PreviewModeEnum::Alt => {
                  for i in PSYNERGY_ITEM_GAMEPLAY_ORDER {
                    if current_data.get_psy_items(i) {
                      Label::new(format!(
                        "{}\t√",
                        ITEM_NAMES[i + PSYNERGY_ITEMS[0] as usize - 1]
                          .get_name(self.item_preview_lang)
                      ))
                      .selectable(false)
                      .ui(ui);
                    } else {
                      Label::new(
                        RichText::new(format!(
                          "{}\t×",
                          ITEM_NAMES[i + PSYNERGY_ITEMS[0] as usize - 1]
                            .get_name(self.item_preview_lang)
                        ))
                        .weak()
                        .strikethrough(),
                      )
                      .selectable(false)
                      .ui(ui);
                    }
                  }
                }
              }
            }
            PreviewPageEnum::Djinn => {
              ui.horizontal(|ui| {
                Label::new("精灵名称")
                  .selectable(false)
                  .ui(ui)
                  .on_hover_text("默认值为“1代汉化”");
                ComboBox::from_id_salt("精灵名语言版本")
                  .width(100.0)
                  .selected_text(self.djinn_preview_lang.get_name())
                  .show_ui(ui, |ui| {
                    for ele in Language::LANGUAGES {
                      ui.selectable_value(&mut self.djinn_preview_lang, ele, ele.get_name());
                    }
                  });
              });

              ui.columns_const(|[col_1, col_2, col_3, col_4]| {
                Label::new("维纳斯").selectable(false).ui(col_1);
                Label::new("玛尔斯").selectable(false).ui(col_2);
                Label::new("朱庇特").selectable(false).ui(col_3);
                Label::new("墨丘利").selectable(false).ui(col_4);
              });

              egui::Grid::new("preview_djinns")
                .striped(true)
                .num_columns(4)
                .min_col_width(
                  (ui.available_width() - ui.spacing().item_spacing.x * (4 as f32 - 1.0)) / 4.0,
                )
                .show(ui, |ui| {
                  for i in 0..7 {
                    for j in 0..4 {
                      if self.preview_current.as_ref().unwrap().get_djinn(i, j) {
                        Label::new(ALL_DJINN_NAMES[j][i].get_name(self.djinn_preview_lang))
                          .selectable(false)
                          .ui(ui);
                      } else {
                        Label::new(
                          RichText::new(ALL_DJINN_NAMES[j][i].get_name(self.djinn_preview_lang))
                            .weak()
                            .strikethrough(),
                        )
                        .selectable(false)
                        .ui(ui);
                      }
                    }

                    ui.end_row();
                  }
                });
            }
            PreviewPageEnum::Misc => {
              if current_data.get_grade() == PasswordGrade::Gold {
                ui.horizontal(|ui| {
                  Label::new("金币数").selectable(false).ui(ui);
                  Label::new(format!("{}", current_data.get_coins()))
                    .selectable(false)
                    .ui(ui);
                });
              }

              ui.separator();

              for i in EVENT_GAMEPLAY_ORDER {
                if current_data.get_events(i) {
                  Label::new(format!("{}\t√", EVENT_STR[i]))
                    .selectable(false)
                    .ui(ui);
                } else {
                  Label::new(
                    RichText::new(format!("{}\t×", EVENT_STR[i]))
                      .weak()
                      .strikethrough(),
                  )
                  .selectable(false)
                  .ui(ui);
                }
              }
            }
          }
        }
      });
  }

  pub fn read_from(&mut self, sub_command: SubCommandEnum) {
    match sub_command {
      SubCommandEnum::First => {
        self.preview_data = [None, None, None];
        let mut is_first_element = true;
        let mut min_slot = 0;
        for (index, ele) in self.raw_save_data.iter().enumerate() {
          if let Some(raw_save) = ele {
            if is_first_element {
              min_slot = index;
              is_first_element = false;
            }

            let pwd_info = gen_password_info_with_raw_save_bytes(raw_save.get_data());
            self.preview_data[index] = Some(PreviewData::new(PasswordGrade::Gold, pwd_info));
          }
        }
        self.preview_slot = SaveSlotEnum::from(min_slot);
        self.preview_current = self.preview_data[min_slot].clone();

        self.char_page_mode = PreviewModeEnum::Main;
        self.item_page_mode = PreviewModeEnum::Main;
        self.pwd_info_window_open = true;
      }
      SubCommandEnum::Second => {
        self.preview_data = [None, None, None];
        let mut password_bits = BitArray::from(&self.txt_bits.as_ref().unwrap().get_bits());
        let src_grade = self.txt_src_grade_version.unwrap().0;
        let save_data = save::get_save_data_with_password_bits(&mut password_bits, src_grade);
        let pwd_info = gen_password_info_with_save_data(&save_data);
        self.preview_data[0] = Some(PreviewData::new(src_grade, pwd_info));
        self.preview_slot = SaveSlotEnum::First;
        self.preview_current = self.preview_data[0].clone();

        match src_grade {
          PasswordGrade::Gold => {
            self.char_page_mode = PreviewModeEnum::Main;
            self.item_page_mode = PreviewModeEnum::Main;
          }
          PasswordGrade::Silver => {
            self.char_page_mode = PreviewModeEnum::Main;
            self.item_page_mode = PreviewModeEnum::Alt;
          }
          PasswordGrade::Bronze => {
            self.char_page_mode = PreviewModeEnum::Alt;
            self.item_page_mode = PreviewModeEnum::Alt;
          }
        }

        self.pwd_info_window_open = true;
      }
      SubCommandEnum::Third => {
        self.preview_data = [None, None, None];
        let mut password_bits = BitArray::from(&self.dmp_bits.as_ref().unwrap().get_bits());
        let src_grade = self.dmp_src_grade.unwrap();
        let save_data = save::get_save_data_with_password_bits(&mut password_bits, src_grade);
        let pwd_info = gen_password_info_with_save_data(&save_data);
        self.preview_data[0] = Some(PreviewData::new(src_grade, pwd_info));
        self.preview_slot = SaveSlotEnum::First;
        self.preview_current = self.preview_data[0].clone();

        match src_grade {
          PasswordGrade::Gold => {
            self.char_page_mode = PreviewModeEnum::Main;
            self.item_page_mode = PreviewModeEnum::Main;
          }
          PasswordGrade::Silver => {
            self.char_page_mode = PreviewModeEnum::Main;
            self.item_page_mode = PreviewModeEnum::Alt;
          }
          PasswordGrade::Bronze => {
            self.char_page_mode = PreviewModeEnum::Alt;
            self.item_page_mode = PreviewModeEnum::Alt;
          }
        }

        self.pwd_info_window_open = true;
      }
    }
  }
}
