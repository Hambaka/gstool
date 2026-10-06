use std::{
  fmt::Write as _,
  fs::{File, create_dir_all},
  io::{Read, Write},
  path::{Path, PathBuf},
};

use egui::{ComboBox, Label, RichText, Ui, Widget};
use rfd::FileDialog;

use crate::{
  App,
  app::{AtomsEnum, SubCommandEnum},
  core::{
    bits::BitArray,
    enums::{BuildDate, GameVersion, PasswordGrade, PasswordVersion},
    password::encode::{encode, from_raw_save},
    save::{self},
  },
  ui::window::pwd_info::SaveSlotEnum,
  widgets::buttons::{reset_btn, select_dir_btn, select_file_btn},
};

include!(concat!(env!("OUT_DIR"), "/codegen_bin.rs"));

const ROM_MEMORY_MAPPING: u32 = 0x08000000;
// len = 12
const SOFTWARE_TITLE_OFFSET: usize = 0xA0;
const SOFTWARE_TITLE_VALUE: [&str; 2] = ["OUGONTAIYO_B", "GOLDEN_SUN_B"];
// len = 12
const ROM_BUILD_DATE_OFFSET: usize = 0x682000;
// len = 4; le
const TARGET_ADDRESS_OFFSET: usize = 0x38304;

const CUSTOM_HEADER: [char; 4] = ['G', 'S', 'P', 'I'];

const PWD_SRC_LABEL: [&str; 3] = ["金密码来源", "银密码来源", "铜密码来源"];
const PWD_SRC_DESC: [&str; 3] = [
  "自动输入金密码所使用的密码数据来源。",
  "自动输入银密码所使用的密码数据来源。",
  "自动输入铜密码所使用的密码数据来源。",
];

const DESC_C5: &str = r"使用该功能可为《黄金太阳 失落的时代》注入能够自动输入密码的程序。
支持的版本：
1. 官方版本ROM（日/美/德/西/法/意）
2. 流传度最高的中文汉化版ROM（SOMA汉化组/Mobile+elffinal）

注：
1. 游戏版本的来源多种多样，无法一一进行测试。";

#[derive(Debug, Clone, Copy, PartialEq)]
enum InjectPasswordCombination {
  GoldSilverBronze,
  GoldSilver,
  GoldBronze,
  SilverBronze,
  Gold,
  Silver,
  Bronze,
}

impl InjectPasswordCombination {
  fn from(value: usize) -> Option<Self> {
    match value {
      0 => Some(InjectPasswordCombination::GoldSilverBronze),
      1 => Some(InjectPasswordCombination::GoldSilver),
      2 => Some(InjectPasswordCombination::GoldBronze),
      3 => Some(InjectPasswordCombination::SilverBronze),
      4 => Some(InjectPasswordCombination::Gold),
      5 => Some(InjectPasswordCombination::Silver),
      6 => Some(InjectPasswordCombination::Bronze),
      _ => None,
    }
  }

  fn get_name(&self) -> &'static str {
    match self {
      Self::GoldSilverBronze => "金银铜",
      Self::GoldSilver => "金银",
      Self::GoldBronze => "金铜",
      Self::SilverBronze => "银铜",
      Self::Gold => "金",
      Self::Silver => "银",
      Self::Bronze => "铜",
    }
  }
}

#[derive(Debug, Clone, Copy)]
struct BinInfo {
  diff_u8_gsb: [Option<usize>; 3],
  diff_u8_other: &'static [usize],
  diff_u32: &'static [(usize, u32)],
  pwd_buffer_start_pos: usize,
  pwd_buffer_end_pos: usize,
  exit_pos: usize,
  pwd_pos: [Option<usize>; 3],
}

struct FileInfo {
  is_matched: bool,
  already_injected_option: Option<InjectPasswordCombination>,
  /// Header
  inject_offset: usize,
  blank_byte: u8,
  base_game_version: GameVersion,
  rom_name: String,
}

impl App {
  pub fn show_c5(&mut self, ui: &mut Ui, svg_icon_size: f32, is_dark: bool) {
    // 正式界面
    Label::new(DESC_C5).selectable(false).ui(ui);

    ui.add_space(16.0);

    egui::Grid::new("ID_GRID_C5")
      .num_columns(2)
      .spacing([40.0, 4.0])
      .striped(true)
      .show(ui, |ui| {
        // 第1行 -> 输入文件路径 【选择文件】【重置】
        Label::new("输入文件路径")
          .selectable(false)
          .ui(ui)
          .on_hover_text("《黄金太阳 失落的时代》的ROM文件");
        ui.horizontal(|ui| {
          if ui.add(select_file_btn(svg_icon_size)).clicked() {
            if let Some(file) = FileDialog::new().pick_file() {
              let new_file_path = file.to_string_lossy().into_owned();
              // 验证ROM文件
              let result = validate_input_file(&new_file_path, false);
              match result {
                Ok(_) => {
                  self.input_file_path_c5 = new_file_path;
                }
                Err(e) => {
                  e.clone_into(&mut self.err_msg);
                  self.err_modal_open = true;
                }
              }
            }
          }
          if self.input_file_path_c5.is_empty() {
            ui.add_enabled(false, reset_btn(svg_icon_size));
          } else if ui.add(reset_btn(svg_icon_size)).clicked() {
            self.input_file_path_c5.clear();
          }
          ui.add(
            egui::TextEdit::singleline(&mut self.input_file_path_c5)
              .interactive(false)
              .hint_text("ROM文件路径")
              .desired_width(f32::INFINITY),
          );
        });
        ui.end_row();

        // 第2行 -> 金密码来源
        // 第3行 -> 银密码来源
        // 第4行 -> 铜密码来源

        for i in 0..3 {
          let data_is_available = self.save_is_available.is_some() || self.txt_bits.is_some() || self.dmp_bits.is_some();

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
                  match &self.pwd_src[i] {
                    Some(sub_command) => AtomsEnum::ImageAndText(sub_command.get_atoms(svg_icon_size, is_dark)) ,
                    None => AtomsEnum::Text("无"),
                  }
                })
                .show_ui(ui, |ui| {
                  if ui.selectable_value(&mut self.pwd_src[i], None, "无").changed() {
                    self.pwd_src_slot[i] = None;
                  };

                  ui.add_enabled_ui(self.save_is_available.is_some(), |ui| {
                    if ui
                      .selectable_value(&mut self.pwd_src[i], Some(SubCommandEnum::First), SubCommandEnum::First.get_atoms(svg_icon_size,is_dark))
                      .changed()
                    {
                      self.pwd_src_slot[i] = Some(SaveSlotEnum::from(self.save_is_available.unwrap()));
                    };
                  });

                  let bool_txt = match i {
                    0 => {
                      if let Some((src_pwd_grade, _)) = self.txt_src_grade_version {
                        match src_pwd_grade {
                          PasswordGrade::Gold => true,
                          PasswordGrade::Silver | PasswordGrade::Bronze => false,
                        }
                      } else {
                        false
                      }
                    }
                    1 => {
                      if let Some((src_pwd_grade, _)) = self.txt_src_grade_version {
                        match src_pwd_grade {
                          PasswordGrade::Gold | PasswordGrade::Silver => true,
                          PasswordGrade::Bronze => false,
                        }
                      } else {
                        false
                      }
                    }
                    _ => true,
                  };

                  ui.add_enabled_ui(self.txt_bits.is_some() && bool_txt, |ui| {
                    if ui
                      .selectable_value(&mut self.pwd_src[i], Some(SubCommandEnum::Second), SubCommandEnum::Second.get_atoms(svg_icon_size,is_dark))
                      .changed()
                    {
                      self.pwd_src_slot[i] = Some(SaveSlotEnum::First);
                    };
                  });

                  let bool_dmp = match i {
                    0 => {
                      if let Some(src_pwd_grade) = self.dmp_src_grade {
                        match src_pwd_grade {
                          PasswordGrade::Gold => true,
                          PasswordGrade::Silver | PasswordGrade::Bronze => false,
                        }
                      } else {
                        false
                      }
                    }
                    1 => {
                      if let Some(src_pwd_grade) = self.dmp_src_grade {
                        match src_pwd_grade {
                          PasswordGrade::Gold | PasswordGrade::Silver => true,
                          PasswordGrade::Bronze => false,
                        }
                      } else {
                        false
                      }
                    }
                    _ => true,
                  };

                  ui.add_enabled_ui(self.dmp_bits.is_some() && bool_dmp, |ui| {
                    if ui
                      .selectable_value(&mut self.pwd_src[i], Some(SubCommandEnum::Third), SubCommandEnum::Third.get_atoms(svg_icon_size,is_dark))
                      .changed()
                    {
                      self.pwd_src_slot[i] = Some(SaveSlotEnum::First);
                    };
                  });
                });

              if self.pwd_src[i] == Some(SubCommandEnum::First) {
                ComboBox::from_id_salt(format!("ID_CB_C5_{}_2", PWD_SRC_LABEL[i]))
                  .width(100.0)
                  .selected_text(match &self.pwd_src_slot[i] {
                    Some(save_slot) => AtomsEnum::ImageAndText(save_slot.get_atoms(svg_icon_size, is_dark, false)),
                    None => AtomsEnum::Text("无"),
                  })
                  .show_ui(ui, |ui| {
                    for j in 0..3 {
                      ui.add_enabled_ui(self.raw_save_data[j].is_some(), |ui| {
                        ui.selectable_value(
                          &mut self.pwd_src_slot[i],
                          Some(SaveSlotEnum::SAVE_SLOTS[j]),
                          SaveSlotEnum::SAVE_SLOTS[j].get_atoms(svg_icon_size, is_dark, false),
                        );
                      });
                    }
                  });
              }
            });
          }

          ui.end_row();
        }

        // 第5行 -> 指定输出文件保存目录  【选择目录】【重置】
        Label::new("指定输出文件保存目录")
          .selectable(false)
          .ui(ui)
          .on_hover_text("自定义输出的汉化版ROM文件路径（推荐留空），若留空则将转换后的ROM文件保存到输入文件的同目录下。");
        ui.horizontal(|ui| {
          if ui.add(select_dir_btn(svg_icon_size)).clicked() {
            if let Some(file) = FileDialog::new().pick_folder() {
              self.out_dir_c5 = file.to_string_lossy().into_owned();
            }
          }
          if self.out_dir_c5.is_empty() {
            ui.add_enabled(false, reset_btn(svg_icon_size));
          } else if ui.add(reset_btn(svg_icon_size)).clicked() {
            self.out_dir_c5.clear();
          }
          ui.add(
            egui::TextEdit::singleline(&mut self.out_dir_c5)
              .interactive(false)
              .hint_text("最新版汉化ROM输出路径")
              .desired_width(f32::INFINITY),
          );
        });
        ui.end_row();
      });
  }

  pub fn run_c5(&mut self) {
    let inject_type = match &self.pwd_src {
      [Some(_), Some(_), Some(_)] => InjectPasswordCombination::GoldSilverBronze,
      [Some(_), Some(_), None] => InjectPasswordCombination::GoldSilver,
      [Some(_), None, Some(_)] => InjectPasswordCombination::GoldBronze,
      [None, Some(_), Some(_)] => InjectPasswordCombination::SilverBronze,
      [Some(_), None, None] => InjectPasswordCombination::Gold,
      [None, Some(_), None] => InjectPasswordCombination::Silver,
      [None, None, Some(_)] => InjectPasswordCombination::Bronze,
      [None, None, None] => unreachable!(),
    };

    let gold_pwd_bytes = if let Some(sub_command) = &self.pwd_src[0] {
      match sub_command {
        SubCommandEnum::First => {
          let save_slot = &self.pwd_src_slot[0].unwrap();
          let raw_save_data = self.raw_save_data[*save_slot as usize]
            .as_ref()
            .unwrap()
            .get_data();
          let password_bytes = from_raw_save(raw_save_data, PasswordGrade::Gold);
          Some(password_bytes)
        }
        SubCommandEnum::Second => Some(self.txt_bytes.clone()),
        SubCommandEnum::Third => Some(self.dmp_bytes.clone()),
      }
    } else {
      None
    };

    let silver_pwd_bytes = if let Some(sub_command) = &self.pwd_src[1] {
      match sub_command {
        SubCommandEnum::First => {
          let save_slot = &self.pwd_src_slot[1].unwrap();
          let raw_save_data = self.raw_save_data[*save_slot as usize]
            .as_ref()
            .unwrap()
            .get_data();
          let password_bytes = from_raw_save(raw_save_data, PasswordGrade::Silver);
          Some(password_bytes)
        }
        SubCommandEnum::Second => match self.txt_src_grade_version {
          Some((src_grade, _)) => {
            if src_grade == PasswordGrade::Silver {
              Some(self.txt_bytes.clone())
            } else {
              let mut password_bits = BitArray::from(&self.txt_bits.as_ref().unwrap().get_bits());

              let save_data = save::get_save_data_with_password_bits(
                &mut password_bits,
                self.txt_src_grade_version.unwrap().0,
              );

              let target_password_bytes = encode(&save_data, PasswordGrade::Silver);
              Some(target_password_bytes)
            }
          }
          None => unreachable!(),
        },
        SubCommandEnum::Third => match self.dmp_src_grade {
          Some(src_grade) => {
            if src_grade == PasswordGrade::Silver {
              Some(self.dmp_bytes.clone())
            } else {
              let mut password_bits = BitArray::from(&self.dmp_bits.as_ref().unwrap().get_bits());
              let save_data = save::get_save_data_with_password_bits(
                &mut password_bits,
                self.dmp_src_grade.unwrap(),
              );

              let target_password_bytes = encode(&save_data, PasswordGrade::Silver);
              Some(target_password_bytes)
            }
          }
          None => unreachable!(),
        },
      }
    } else {
      None
    };

    let bronze_pwd_bytes = if let Some(sub_command) = &self.pwd_src[2] {
      match sub_command {
        SubCommandEnum::First => {
          let save_slot = &self.pwd_src_slot[2].unwrap();
          let raw_save_data = self.raw_save_data[*save_slot as usize]
            .as_ref()
            .unwrap()
            .get_data();
          let password_bytes = from_raw_save(raw_save_data, PasswordGrade::Bronze);
          Some(password_bytes)
        }
        SubCommandEnum::Second => match self.txt_src_grade_version {
          Some((src_grade, _)) => {
            if src_grade == PasswordGrade::Bronze {
              Some(self.txt_bytes.clone())
            } else {
              let mut password_bits = BitArray::from(&self.txt_bits.as_ref().unwrap().get_bits());

              let save_data = save::get_save_data_with_password_bits(
                &mut password_bits,
                self.txt_src_grade_version.unwrap().0,
              );

              let target_password_bytes = encode(&save_data, PasswordGrade::Bronze);
              Some(target_password_bytes)
            }
          }
          None => unreachable!(),
        },
        SubCommandEnum::Third => match self.dmp_src_grade {
          Some(src_grade) => {
            if src_grade == PasswordGrade::Bronze {
              Some(self.dmp_bytes.clone())
            } else {
              let mut password_bits = BitArray::from(&self.dmp_bits.as_ref().unwrap().get_bits());
              let save_data = save::get_save_data_with_password_bits(
                &mut password_bits,
                self.dmp_src_grade.unwrap(),
              );

              let target_password_bytes = encode(&save_data, PasswordGrade::Bronze);
              Some(target_password_bytes)
            }
          }
          None => unreachable!(),
        },
      }
    } else {
      None
    };

    let result = inject(
      &self.input_file_path_c5,
      inject_type,
      [gold_pwd_bytes, silver_pwd_bytes, bronze_pwd_bytes],
    );
    if let Err(e) = result {
      e.clone_into(&mut self.err_msg);
      self.err_modal_open = true;
      self.input_file_path_c5.clear();
      return;
    }

    let (new_rom, file_info) = result.unwrap();

    // Start to create and write output ROM file.
    let output_path;

    let input_path = Path::new(&self.input_file_path_c5);
    let file_stem = input_path.file_stem().unwrap().to_str().unwrap();
    let file_extension = if input_path.extension().is_none() {
      ""
    } else {
      input_path.extension().unwrap().to_str().unwrap()
    };
    let mut file_name_str = String::new();
    file_name_str.push_str(file_stem);
    file_name_str.push_str(&format!("_{}", inject_type.get_name()));

    if !file_extension.is_empty() {
      file_name_str.push('.');
      file_name_str.push_str(file_extension);
    }

    if self.out_dir_c5.is_empty() {
      let parent = input_path.parent().unwrap().to_str().unwrap();
      output_path = PathBuf::from(parent).join(file_name_str);
    } else {
      if create_dir_all(self.out_dir_c5.clone()).is_err() {
        "创建目录失败！".clone_into(&mut self.err_msg);
        self.err_modal_open = true;
        return;
      }
      output_path = PathBuf::from(self.out_dir_c5.clone()).join(file_name_str);
    }

    let output_file = File::create(output_path.clone());
    if let Ok(mut file) = output_file {
      if file.write_all(&new_rom).is_err() {
        self.err_msg = format!("创建 “{}” 失败！", output_path.to_str().unwrap());
        self.err_modal_open = true;
        return;
      }
    } else {
      self.err_msg = format!("创建 “{}” 失败！", output_path.to_str().unwrap());
      self.err_modal_open = true;
      return;
    }

    // Log
    self.out_log_c5.clear();
    let _ = write!(
      self.out_log_c5,
      "输出文件：\n{}\n\n",
      output_path.to_str().unwrap()
    );

    self.out_log_c5.push_str("运行概况：\n");

    let mut result_index = 1;

    if file_info.is_matched {
      let _ = writeln!(
        self.out_log_c5,
        "{result_index}. 输入的源文件版本信息：“{}”（无任何修改）",
        file_info.rom_name
      );
    } else {
      match file_info.already_injected_option {
        Some(combo) => {
          let _ = writeln!(
            self.out_log_c5,
            "{result_index}. 输入的源文件（本质）版本信息：“{}”，（已注入密码，密码类型：{}）",
            file_info.rom_name,
            combo.get_name()
          );
        }
        None => {
          let _ = writeln!(
            self.out_log_c5,
            "{result_index}. 输入的源文件（本质）版本信息：“{}”，（未注入密码）",
            file_info.rom_name
          );
        }
      }
    }

    result_index += 1;
    let _ = writeln!(
      self.out_log_c5,
      "{result_index}. 本次注入的密码类型为：“{}”",
      inject_type.get_name()
    );
  }

  pub fn show_c5_log(&mut self, ui: &mut Ui) {
    Label::new(&self.out_log_c5).selectable(false).ui(ui);
  }
}

fn validate_input_file(
  path_str: &str,
  get_bytes: bool,
) -> Result<(FileInfo, Option<Vec<u8>>), &'static str> {
  // Read rom file.
  let input_file_result = File::open(path_str);
  let mut rom_file;
  if input_file_result.is_err() {
    return Err("打开 ROM 文件时发生错误！");
  }
  rom_file = input_file_result.unwrap();

  let file_size = rom_file.metadata().unwrap().len() as usize;
  if file_size > 0x200_0000 {
    return Err("ROM 大小不应超过 32 MB！");
  }

  let mut rom_bytes = Vec::new();
  rom_file.read_to_end(&mut rom_bytes).unwrap();

  let crc32 = crc32fast::hash(&rom_bytes);

  // 输入的ROM与记录的各类未修改“原版”ROM匹配时，直接返回提前记录好的信息
  if let Some((rom_name_str, base_game_version, inject_offset, blank_byte)) =
    KNOWN_INPUT_ROM_INFO_MAP.get(&crc32)
  {
    Ok((
      FileInfo {
        is_matched: true,
        already_injected_option: None,
        inject_offset: *inject_offset,
        blank_byte: *blank_byte,
        base_game_version: *base_game_version,
        rom_name: String::from(*rom_name_str),
      },
      if get_bytes {
        Some(rom_bytes.clone())
      } else {
        None
      },
    ))
  } else {
    // 未匹配到任何“原版”ROM，需要进一步检查该ROM的各类信息
    let is_matched = false;

    let already_injected_option;
    let inject_offset;
    let blank_byte;
    let base_game_version;
    let rom_name;

    // 获取游戏ROM的标题文本
    let software_title =
      String::from_utf8_lossy(&rom_bytes[SOFTWARE_TITLE_OFFSET..SOFTWARE_TITLE_OFFSET + 12])
        .into_owned();
    if software_title == SOFTWARE_TITLE_VALUE[PasswordVersion::Japanese as usize] {
      // 仅有日版或者基于日版修改的版本的标题文本才会是“OUGONTAIYO_B”
      base_game_version = GameVersion::Japan;
      rom_name = String::from(BASE_ROM_NAME[GameVersion::Japan as usize]);
    } else if software_title == SOFTWARE_TITLE_VALUE[PasswordVersion::English as usize] {
      // 其余的美、德、西、法、意版本的标题文本都是“GOLDEN_SUN_B”
      // 直接根据游戏ROM内的构建日期信息判断游戏ROM的版本
      let rom_build_date =
        String::from_utf8_lossy(&rom_bytes[ROM_BUILD_DATE_OFFSET..ROM_BUILD_DATE_OFFSET + 12])
          .into_owned();
      let game_version_option = GameVersion::from_build_date(BuildDate::String(rom_build_date));
      if game_version_option.is_none() {
        return Err("无法确定该《黄金太阳 失落的时代》ROM的版本！");
      } else {
        base_game_version = game_version_option.unwrap();
        rom_name = String::from(BASE_ROM_NAME[base_game_version as usize]);
      }
    } else {
      return Err("输入的不是《黄金太阳 失落的时代》的ROM文件！");
    }

    // 获取注入自动密码输入程序时要修改的程序入口地址值位置的原始值
    let target_address = u32::from_le_bytes(
      (&rom_bytes[TARGET_ADDRESS_OFFSET..TARGET_ADDRESS_OFFSET + 4])
        .try_into()
        .unwrap(),
    );

    // 日、美、德、西、法、意版的ROM在此处的值都各不相同
    if GameVersion::from_target_address(target_address).is_some() {
      // 若能匹配，则证明游戏ROM没有被注入自动输入密码的程序
      let last_byte = rom_bytes[file_size - 1];

      let remain = file_size % 0x10;
      let padding_size = if remain > 0 { 0x10 - remain } else { 0 };

      // 如果文件最后一个字节不是空白字节（0xFF或0x00），那么基本上新程序数据的添加点就是下一个十六字节的起始位置
      if last_byte != 0xFF && last_byte != 0x00 {
        blank_byte = 0x00;

        // 不做情况区分了，直接默认使用最大的金、银、铜的程序的对齐十六字节的大小0x370 + 0x10
        if 0x200_0000 - file_size - padding_size >= 0x380 {
          let new_len = file_size + padding_size + 0x380;
          rom_bytes.resize(new_len, 0x00);
          inject_offset = file_size + padding_size;
          already_injected_option = None;
        } else {
          return Err("没有足够的空间用于注入自动输入密码的程序！");
        }
      } else {
        // 若最后一个字节是空白字节（0xFF或0x00），那么需要从文件末尾往前查找空白字节的位置
        blank_byte = last_byte;
        let mut new_pos = 0;
        let mut counter = 0;
        let mut inject_offset_found = false;

        // 如果文件不是16字节对齐，那么优先把未对齐的部分判断一下
        let mut found_pos_in_remain = false;
        for i in file_size - remain..file_size {
          if rom_bytes[i] != blank_byte {
            found_pos_in_remain = true;
            break;
          }
        }

        // 如果未对齐的部分里已经有字节不是空白字节，那么插入点就是下一个十六字节的起始位置
        if found_pos_in_remain {
          inject_offset_found = true;
          new_pos = file_size + padding_size;
        } else {
          // 如果未对齐部分都是空白字节，则起始下标跳过未对齐的部分，以16字节为步长往前查找插入的位置
          let start_index = file_size - remain;

          // 先把起始下标赋值（即下一个十六字节的起始位置）（应对第一遍循环就找到的情况）
          let mut new_inject_offset = start_index;
          for i in (0..start_index).step_by(0x10).rev() {
            for j in 0..0x10 {
              if rom_bytes[i - j] != blank_byte {
                inject_offset_found = true;
                break;
              }
            }

            if inject_offset_found {
              new_pos = new_inject_offset;
              break;
            } else {
              new_inject_offset -= 0x10;
              counter += 1;
            }
          }
        }

        if !inject_offset_found {
          return Err("未找到该《黄金太阳 失落的时代》ROM的自动输入密码的程序注入位置！");
        } else {
          if 0x200_0000 - file_size - (counter * 0x10) >= 0x380 {
            inject_offset = new_pos;
            already_injected_option = None;

            if counter < 38 {
              let new_len = file_size + 0x380 - counter * 0x10 + padding_size;
              rom_bytes.resize(new_len, blank_byte);
            }
          } else {
            return Err("没有足够的空间用于注入自动输入密码的程序！");
          }
        }
      }
    } else {
      // 若不能匹配，则证明游戏ROM已经被注入自动输入密码的程序，直接使用ROM已经记录的信息，无需二次计算
      inject_offset = (target_address - 1 - 0x10 - ROM_MEMORY_MAPPING) as usize;

      let header =
        String::from_utf8_lossy(&rom_bytes[inject_offset..inject_offset + 4]).into_owned();

      if header != CUSTOM_HEADER.iter().collect::<String>() {
        return Err("无法确定该《黄金太阳 失落的时代》ROM是否已经注入了自动输入密码的程序！");
      }

      // 虽然不太可能发生，但是以防万一，还是检查一下剩余的空间够不够
      if inject_offset + 0x380 > file_size {
        return Err("没有足够的空间用于注入自动输入密码的程序！");
      }

      let inject_type_num = u32::from_le_bytes(
        (&rom_bytes[inject_offset + 4..inject_offset + 8])
          .try_into()
          .unwrap(),
      );

      let inject_option = InjectPasswordCombination::from(inject_type_num as usize);

      if inject_option.is_none() {
        return Err("无法确定该《黄金太阳 失落的时代》ROM是否已经注入了自动输入密码的程序！");
      }

      already_injected_option = inject_option;
      blank_byte = rom_bytes[inject_offset + 8];
    };

    Ok((
      FileInfo {
        is_matched,
        already_injected_option,
        inject_offset,
        blank_byte,
        base_game_version,
        rom_name,
      },
      if get_bytes {
        Some(rom_bytes.clone())
      } else {
        None
      },
    ))
  }
}

fn inject(
  path_str: &str,
  inject_type: InjectPasswordCombination,
  pwd_bytes_options: [Option<Vec<u8>>; 3],
) -> Result<(Vec<u8>, FileInfo), &'static str> {
  match validate_input_file(path_str, true) {
    Ok((file_info, rom_bytes)) => {
      let bin_info = BIN_INFO[inject_type as usize];
      let bin = BIN_BYTES[inject_type as usize];

      let mut injected_rom = rom_bytes.unwrap();
      let mut start_index = file_info.inject_offset;

      for i in 0..4 {
        injected_rom[start_index + i] = CUSTOM_HEADER[i] as u8;
      }

      let inject_type_num = u32::to_le_bytes(inject_type as u32);
      for i in 4..8 {
        injected_rom[start_index + i] = inject_type_num[i - 4] as u8;
      }

      for i in 8..16 {
        injected_rom[start_index + i] = file_info.blank_byte;
      }

      start_index += 16;

      match file_info.already_injected_option {
        Some(inject_pwd_type) => {
          for i in 0..BIN_BYTES[inject_pwd_type as usize].len() {
            injected_rom[start_index + i] = file_info.blank_byte;
          }
        }
        None => {
          let target_address_value = u32::to_le_bytes(start_index as u32 + ROM_MEMORY_MAPPING + 1);
          for i in 0..4 {
            injected_rom[TARGET_ADDRESS_OFFSET + i] = target_address_value[i] as u8;
          }
        }
      }

      for i in 0..bin.len() {
        injected_rom[start_index + i] = bin[i];
      }

      let password_version = PasswordVersion::from(file_info.base_game_version);

      for (index, pos_option) in bin_info.diff_u8_gsb.iter().enumerate() {
        if let Some(pos) = pos_option {
          let u8_value = match index {
            0 => password_version.get_diff_u8_gold(),
            1 => password_version.get_diff_u8_silver(),
            _ => password_version.get_diff_u8_bronze(),
          };
          injected_rom[start_index + pos] = u8_value;
        }
      }

      for index in bin_info.diff_u8_other {
        injected_rom[start_index + index] = password_version.get_diff_u8_other();
      }

      for (index, value) in bin_info.diff_u32 {
        let final_value = u32::to_le_bytes(value + start_index as u32 + ROM_MEMORY_MAPPING);
        for i in *index..*index + 4 {
          injected_rom[start_index + i] = final_value[i - *index];
        }
      }

      let pwd_buffer_address_value =
        u32::to_le_bytes(file_info.base_game_version.get_tla_pwd_buffer_address());
      for i in 0..4 {
        injected_rom[start_index + bin_info.pwd_buffer_start_pos + i] = pwd_buffer_address_value[i];
      }

      let pwd_buffer_address_end_value =
        u32::to_le_bytes(file_info.base_game_version.get_tla_pwd_address_end());
      for i in 0..4 {
        injected_rom[start_index + bin_info.pwd_buffer_end_pos + i] =
          pwd_buffer_address_end_value[i];
      }

      let exit_address_value = u32::to_le_bytes(file_info.base_game_version.get_tla_exit_address());
      for i in 0..4 {
        injected_rom[start_index + bin_info.exit_pos + i] = exit_address_value[i];
      }

      for i in 0..3 {
        if let (Some(pwd_bytes), Some(pwd_start_offset)) =
          (&pwd_bytes_options[i], bin_info.pwd_pos[i])
        {
          for i in 0..pwd_bytes.len() {
            injected_rom[start_index + pwd_start_offset + i] = pwd_bytes[i];
          }
        }
      }

      Ok((injected_rom, file_info))
    }
    Err(e) => Err(e),
  }
}
