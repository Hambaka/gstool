use egui::Shadow;
use egui_notify::{Anchor, Toasts};

use crate::{
  core::{
    data::Language,
    enums::{GameType, GameVersion, NameType, PasswordGrade, PasswordVersion},
    save::{BitArray, RawSaveData},
  },
  ui::window::pwd_info::{PreviewData, PreviewModeEnum, PreviewPageEnum, SaveSlotEnum},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommandEnum {
  First,
  Second,
  Third,
  Fourth,
  Fifth,
}

impl CommandEnum {
  pub const COMMANDS: [CommandEnum; 5] = [
    CommandEnum::First,
    CommandEnum::Second,
    CommandEnum::Third,
    CommandEnum::Fourth,
    CommandEnum::Fifth,
  ];

  pub fn get_name(self) -> &'static str {
    match self {
      CommandEnum::First => "❶ 密码导出功能",
      CommandEnum::Second => "❷ 存档转换功能",
      CommandEnum::Third => "❸ 自定义角色名",
      CommandEnum::Fourth => "❹ 应用汉化补丁",
      CommandEnum::Fifth => "❺ 注入密码功能",
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SubCommandEnum {
  /// ❶ 密码导出功能 -> 🅰 游戏存档文件
  First,
  /// ❶ 密码导出功能 -> 🅱 密码文本文件
  Second,
  /// ❶ 密码导出功能 -> 🅲 内存转储文件
  Third,
}

impl SubCommandEnum {
  pub const SUB_COMMANDS: [SubCommandEnum; 3] = [
    SubCommandEnum::First,
    SubCommandEnum::Second,
    SubCommandEnum::Third,
  ];

  pub fn get_name(self) -> &'static str {
    match self {
      SubCommandEnum::First => "🅰 游戏存档文件",
      SubCommandEnum::Second => "🅱 密码文本文件",
      SubCommandEnum::Third => "🅲 内存转储文件",
    }
  }
}

pub struct App {
  pub preview_data: [Option<PreviewData>; 3],
  pub preview_current: Option<PreviewData>,
  pub preview_slot: SaveSlotEnum,
  pub preview_page: PreviewPageEnum,
  /// 全属性（金、银），仅等级（铜）
  pub char_page_mode: PreviewModeEnum,
  /// 基础属性，属性
  pub stats_mode: PreviewModeEnum,
  /// 全道具（金），仅精神力道具（银、铜）
  pub item_page_mode: PreviewModeEnum,
  pub item_preview_lang: Language,
  pub djinn_preview_lang: Language,

  pub about_window_open: bool,
  pub pwd_info_window_open: bool,
  pub toasts: Toasts,
  pub command: CommandEnum,
  pub subcommand: SubCommandEnum,
  pub err_modal_open: bool,
  pub err_msg: String,
  // 1. 密码导出功能部分
  // 1.1 使用存档文件
  pub to_export_all_save: bool,
  pub input_file_path_c1s1: String,
  pub save_is_available: Option<usize>,
  pub raw_save_data: [Option<RawSaveData>; 3],
  pub pwd_grade_c1s1: PasswordGrade,
  pub pwd_text_ver_c1s1: Option<PasswordVersion>,
  pub to_export_pwd_bin_c1s1: bool,
  pub pwd_cheat_ver_c1s1: Option<GameVersion>,
  pub to_export_pwd_data_text_c1s1: bool,
  pub out_dir_c1s1: String,
  pub out_log_c1s1: String,
  // 1.2 使用密码文本文件
  pub input_file_path_c1s2: String,
  pub txt_file: String,
  pub txt_bytes: Vec<u8>,
  pub txt_bits: Option<BitArray>,
  pub txt_src_grade_version: Option<(PasswordGrade, PasswordVersion)>,
  pub pwd_grade_c1s2: Option<PasswordGrade>,
  pub to_convert_pwd_text_ver: bool,
  pub to_export_pwd_bin_c1s2: bool,
  pub pwd_cheat_ver_c1s2: Option<GameVersion>,
  pub to_export_pwd_data_text_c1s2: bool,
  pub out_dir_c1s2: String,
  pub out_log_c1s2: String,
  // 1.3 使用内存转储文件
  pub input_file_path_c1s3: String,
  pub dmp_bytes: Vec<u8>,
  pub dmp_bits: Option<BitArray>,
  pub dmp_src_grade: Option<PasswordGrade>,
  pub pwd_grade_c1s3: Option<PasswordGrade>,
  pub pwd_text_ver_c1s3: Option<PasswordVersion>,
  pub pwd_cheat_ver_c1s3: Option<GameVersion>,
  pub to_export_pwd_data_text_c1s3: bool,
  pub out_dir_c1s3: String,
  pub out_log_c1s3: String,
  // 2. 存档转换功能部分
  pub input_file_path_c2: String,
  pub save_file: Vec<u8>,
  pub save_src_type_index: Option<(GameType, GameVersion, usize)>,
  pub default_name_ver: Option<NameType>,
  pub build_date_ver: Option<GameVersion>,
  pub out_dir_c2: String,
  pub out_log_c2: String,
  // 3. 自定义角色名部分
  /// 主角团7人的自定义名字
  pub new_pc_names: Vec<String>,
  pub out_log_c3: String,
  // 4. 打汉化补丁功能部分
  pub decompressed_bps: Vec<u8>,
  pub input_file_path_c4: String,
  pub input_file_crc32: Option<u32>,
  pub out_dir_c4: String,
  pub out_log_c4: String,
  // 5. 注入密码功能部分
  pub input_file_path_c5: String,
  pub pwd_src: [Option<SubCommandEnum>; 3],
  pub pwd_src_slot: [Option<SaveSlotEnum>; 3],
  pub out_dir_c5: String,
  pub out_log_c5: String,
}

impl Default for App {
  fn default() -> Self {
    Self {
      preview_data: [None, None, None],
      preview_current: None,
      preview_slot: SaveSlotEnum::First,
      preview_page: PreviewPageEnum::Character,
      char_page_mode: PreviewModeEnum::Main,
      stats_mode: PreviewModeEnum::Main,
      item_page_mode: PreviewModeEnum::Main,
      item_preview_lang: Language::ChineseTBS,
      djinn_preview_lang: Language::ChineseTBS,

      about_window_open: false,
      pwd_info_window_open: false,
      toasts: Toasts::default()
        .with_anchor(Anchor::BottomRight)
        .with_shadow(Shadow {
          offset: [1, 2],
          blur: 10,
          spread: 3,
          color: egui::Color32::from_black_alpha(70),
        }),
      command: CommandEnum::First,
      subcommand: SubCommandEnum::First,
      err_modal_open: false,
      err_msg: String::new(),
      // 1. 密码导出功能部分
      // 1.1 使用存档文件
      to_export_all_save: false,
      input_file_path_c1s1: String::new(),
      save_is_available: None,
      raw_save_data: [None, None, None],
      pwd_grade_c1s1: PasswordGrade::Gold,
      pwd_text_ver_c1s1: Some(PasswordVersion::English),
      to_export_pwd_bin_c1s1: false,
      pwd_cheat_ver_c1s1: None,
      to_export_pwd_data_text_c1s1: false,
      out_dir_c1s1: String::new(),
      out_log_c1s1: String::new(),
      // 1.2 使用密码文本文件
      input_file_path_c1s2: String::new(),
      txt_file: String::new(),
      txt_bytes: Vec::new(),
      txt_bits: None,
      txt_src_grade_version: None,
      pwd_grade_c1s2: None,
      to_convert_pwd_text_ver: false,
      to_export_pwd_bin_c1s2: false,
      pwd_cheat_ver_c1s2: None,
      to_export_pwd_data_text_c1s2: false,
      out_dir_c1s2: String::new(),
      out_log_c1s2: String::new(),
      // 1.3 使用内存转储文件
      input_file_path_c1s3: String::new(),
      dmp_bytes: Vec::new(),
      dmp_bits: None,
      dmp_src_grade: None,
      pwd_grade_c1s3: None,
      pwd_text_ver_c1s3: None,
      pwd_cheat_ver_c1s3: None,
      to_export_pwd_data_text_c1s3: false,
      out_dir_c1s3: String::new(),
      out_log_c1s3: String::new(),
      // 2. 存档转换功能部分
      input_file_path_c2: String::new(),
      save_file: Vec::new(),
      save_src_type_index: None,
      default_name_ver: None,
      build_date_ver: None,
      out_dir_c2: String::new(),
      out_log_c2: String::new(),
      // 3. 自定义角色名部分
      new_pc_names: vec![String::new(); 7],
      out_log_c3: String::new(),
      // 4. 打汉化补丁功能部分
      decompressed_bps: {
        zstd::decode_all(include_bytes!(concat!(env!("OUT_DIR"), "/all_bps.zst")).as_slice())
          .unwrap()
      },
      input_file_path_c4: String::new(),
      input_file_crc32: None,
      out_dir_c4: String::new(),
      out_log_c4: String::new(),
      // 5. 注入密码功能部分
      input_file_path_c5: String::new(),
      pwd_src: [None, None, None],
      pwd_src_slot: [None, None, None],
      out_dir_c5: String::new(),
      out_log_c5: String::new(),
    }
  }
}

impl App {
  pub fn name() -> String {
    // env!("CARGO_PKG_VERSION")
    // env!("CARGO_PKG_NAME")
    format!("GSTool - {}", env!("CARGO_PKG_VERSION"))
  }

  pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
    // 解压内嵌字体
    let decompressed_font =
      zstd::decode_all(include_bytes!(concat!(env!("OUT_DIR"), "/font.zst")).as_slice()).unwrap();
    // 替换默认字体以支持显示CJK等字符
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
      "simplified_chinese_font".to_owned(),
      std::sync::Arc::new(egui::FontData::from_owned(decompressed_font)),
    );
    fonts
      .families
      .entry(egui::FontFamily::Proportional)
      .or_default()
      .insert(0, "simplified_chinese_font".to_owned());
    cc.egui_ctx.set_fonts(fonts);

    Default::default()
  }
}

impl eframe::App for App {
  fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
    // “关于”窗口
    self.show_about_window(ui);
    // 密码信息窗口
    self.show_pwd_info_window(ui);
    // 错误信息显示模态窗口
    if self.err_modal_open {
      self.show_error_modal(ui);
    }
    // 顶部面板，主要用于菜单栏。
    self.show_top_panel(ui);
    // 主面板，核心功能区。
    self.show_main_panel(ui);
    self.toasts.show(ui);
  }
}
