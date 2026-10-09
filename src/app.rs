use std::io::Read;

use egui::{Image, Shadow, TextStyle, TextureHandle, TextureOptions};
use egui_notify::{Anchor, Toasts};
use image::GenericImageView;

use crate::{
  core::{
    bits::BitVec,
    data::Language,
    enums::{GameRegion, NameType, PasswordGrade, PasswordVersion},
    save::{GameInfo, SaveSlot},
  },
  ui::window::pwd_info::{PreviewData, PreviewModeEnum, PreviewPageEnum, SaveSlotEnum},
  widgets::{
    colors::get_default_svg_color,
    icons::{
      ARROW_DOWN, ARROW_LEFT_RIGHT, ARROW_UP, NUMBER_1_SQUARE_FILL, NUMBER_2_SQUARE_FILL,
      NUMBER_3_SQUARE_FILL, PENCIL, TOOLS,
    },
  },
};

#[derive(Debug, Clone)]
pub enum AtomsEnum {
  Text(&'static str),
  ImageAndText((Image<'static>, &'static str)),
}

impl<'a> egui::IntoAtoms<'a> for AtomsEnum {
  fn collect(self, atoms: &mut egui::Atoms<'a>) {
    match self {
      AtomsEnum::Text(text) => text.collect(atoms),
      AtomsEnum::ImageAndText(image_and_text) => image_and_text.collect(atoms),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommandEnum {
  First = 0,
  Second = 1,
  Third = 2,
  Fourth = 3,
  Fifth = 4,
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
      // ⏏、⮉、⤴、⬆
      CommandEnum::First => "密码导出功能",
      // ⇆
      CommandEnum::Second => "存档转换功能",
      // 📝、🖊
      CommandEnum::Third => " 自定义角色名",
      // 🌐、🔨、🛠
      CommandEnum::Fourth => "应用汉化补丁",
      // ⤵、⤓、⮋、⬇
      CommandEnum::Fifth => "注入密码功能",
    }
  }

  pub fn get_atoms(self, svg_icon_size: f32, is_dark: bool) -> (Image<'static>, &'static str) {
    let color = get_default_svg_color(is_dark);

    (
      Image::new(match self {
        CommandEnum::First => ARROW_UP,
        CommandEnum::Second => ARROW_LEFT_RIGHT,
        CommandEnum::Third => PENCIL,
        CommandEnum::Fourth => TOOLS,
        CommandEnum::Fifth => ARROW_DOWN,
      })
      .tint(color)
      .max_height(svg_icon_size),
      self.get_name(),
    )
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SubCommandEnum {
  /// ❶ 密码导出功能 -> 🅰 游戏存档
  First = 0,
  /// ❶ 密码导出功能 -> 🅱 密码文本
  Second = 1,
  /// ❶ 密码导出功能 -> 🅲 内存转储
  Third = 2,
}

impl SubCommandEnum {
  pub const SUB_COMMANDS: [SubCommandEnum; 3] = [
    SubCommandEnum::First,
    SubCommandEnum::Second,
    SubCommandEnum::Third,
  ];

  pub fn get_name(&self) -> &'static str {
    match self {
      SubCommandEnum::First => "游戏存档",
      SubCommandEnum::Second => "密码文本",
      SubCommandEnum::Third => "内存转储",
    }
  }

  pub fn get_atoms(&self, svg_icon_size: f32, is_dark: bool) -> (Image<'static>, &'static str) {
    let color = get_default_svg_color(is_dark);

    (
      Image::new(match self {
        SubCommandEnum::First => NUMBER_1_SQUARE_FILL,
        SubCommandEnum::Second => NUMBER_2_SQUARE_FILL,
        SubCommandEnum::Third => NUMBER_3_SQUARE_FILL,
      })
      .tint(color)
      .max_height(svg_icon_size),
      self.get_name(),
    )
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
  pub item_icon_img: Vec<TextureHandle>,

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
  pub save_is_available: bool,
  pub first_available_slot: Option<SaveSlotEnum>,
  pub raw_save_data: [Option<SaveSlot>; 3],
  pub pwd_grade_c1s1: PasswordGrade,
  pub pwd_text_ver_c1s1: Option<PasswordVersion>,
  pub to_export_pwd_bin_c1s1: bool,
  pub pwd_cheat_ver_c1s1: Option<GameRegion>,
  pub to_export_pwd_data_text_c1s1: bool,
  pub out_dir_c1s1: String,
  pub out_log_c1s1: String,
  // 1.2 使用密码文本文件
  pub input_file_path_c1s2: String,
  pub txt_file: String,
  pub txt_bytes: Vec<u8>,
  pub txt_bits: Option<BitVec>,
  pub txt_src_grade_version: Option<(PasswordGrade, PasswordVersion)>,
  pub pwd_grade_c1s2: Option<PasswordGrade>,
  pub to_convert_pwd_text_ver: bool,
  pub to_export_pwd_bin_c1s2: bool,
  pub pwd_cheat_ver_c1s2: Option<GameRegion>,
  pub to_export_pwd_data_text_c1s2: bool,
  pub out_dir_c1s2: String,
  pub out_log_c1s2: String,
  // 1.3 使用内存转储文件
  pub input_file_path_c1s3: String,
  pub dmp_bytes: Vec<u8>,
  pub dmp_bits: Option<BitVec>,
  pub dmp_src_grade: Option<PasswordGrade>,
  pub pwd_grade_c1s3: Option<PasswordGrade>,
  pub pwd_text_ver_c1s3: Option<PasswordVersion>,
  pub pwd_cheat_ver_c1s3: Option<GameRegion>,
  pub to_export_pwd_data_text_c1s3: bool,
  pub out_dir_c1s3: String,
  pub out_log_c1s3: String,
  // 2. 存档转换功能部分
  pub input_file_path_c2: String,
  pub save_file: Vec<u8>,
  pub save_src_type_index: Option<GameInfo>,
  pub default_name_ver: Option<NameType>,
  pub build_date_ver: Option<GameRegion>,
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

impl App {
  pub fn name() -> String {
    // env!("CARGO_PKG_VERSION")
    // env!("CARGO_PKG_NAME")
    format!("GSTool - {}", env!("CARGO_PKG_VERSION"))
  }

  pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
    // 1. 设置道具图标图片
    // 加载道具图标图片
    let image_bytes = include_bytes!("../assets/item_icon_img/all_items.png");
    let image = image::load_from_memory_with_format(image_bytes, image::ImageFormat::Png).unwrap();
    let mut handles = Vec::new();

    let icon_size = 16;
    let columns = 10;

    for id in 0..362 {
      // 当前图标所在列
      let col = id % columns;
      // 当前图标所在行
      let row = id / columns;

      // 起始像素坐标
      let x = (col * icon_size) as u32;
      let y = (row * icon_size) as u32;

      // 从完整大图中获取该图标的 16x16 的图像
      let sub_image = image.view(x, y, icon_size as u32, icon_size as u32);
      // 开始转换为egui的TextureHandle
      let rgba_image = sub_image.to_image();
      let pixels = rgba_image.into_raw();
      let color_image = egui::ColorImage::from_rgba_unmultiplied([icon_size, icon_size], &pixels);

      let handle = cc.egui_ctx.load_texture(
        format!("item_icon_{}", id),
        color_image,
        TextureOptions::NEAREST,
      );

      handles.push(handle);
    }

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
      item_icon_img: handles,

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
      save_is_available: false,
      first_available_slot: None,
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
        let mut compressed_data =
          include_bytes!(concat!(env!("OUT_DIR"), "/all_bps.zst")).as_slice();
        let mut decoder = ruzstd::decoding::StreamingDecoder::new(&mut compressed_data).unwrap();
        let mut decompreessed_data = Vec::new();
        decoder.read_to_end(&mut decompreessed_data).unwrap();
        decompreessed_data
        /*
        zstd::decode_all(include_bytes!(concat!(env!("OUT_DIR"), "/all_bps.zst")).as_slice())
          .unwrap()
        */
        /*
        let mut compressed_data =
          include_bytes!(concat!(env!("OUT_DIR"), "/all_bps.zst")).as_slice();
        let mut decoder =
          structured_zstd::decoding::StreamingDecoder::new(&mut compressed_data).unwrap();
        let mut decompreessed_data = Vec::new();
        decoder.read_to_end(&mut decompreessed_data).unwrap();
        decompreessed_data
        */
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

impl eframe::App for App {
  fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
    let svg_icon_size = ui.text_style_height(&TextStyle::Body);
    let is_dark = ui.visuals().dark_mode;

    // ui.set_visuals(egui::Visuals::dark());

    // “关于”窗口
    self.show_about_window(ui, svg_icon_size, is_dark);
    // 密码信息窗口
    self.show_pwd_info_window(ui, svg_icon_size, is_dark);
    // 错误信息显示模态窗口
    if self.err_modal_open {
      self.show_error_modal(ui);
    }
    // 顶部面板，主要用于菜单栏。
    self.show_top_panel(ui, svg_icon_size, is_dark);
    // 主面板，核心功能区。
    self.show_main_panel(ui, svg_icon_size, is_dark);
    self.toasts.show(ui);
  }
}
