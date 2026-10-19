use egui::{Image, Shadow, TextStyle, TextureHandle, TextureOptions};
use egui_notify::{Anchor, Toasts};
use image::GenericImageView;

use crate::{
  core::data::Language,
  logic::command::{CommandEnum, SubCommandEnum},
  logic::password_export::PasswordExportState,
  logic::save_convert::SaveConvertState,
  logic::save_slot::SaveSlotEnum,
  logic::sources::PasswordSources,
  ui::{
    page::AppRequest,
    window::pwd_info::{PreviewData, PreviewModeEnum, PreviewPageEnum},
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
  /// 跨功能的请求队列：先在ui侧发送请求进行登记，再到下一帧由`App::logic()`消费（约定见`ui/page.rs`）
  pub requests: Vec<AppRequest>,
  pub err_modal_open: bool,
  pub err_msg: String,
  // 1. 密码导出功能部分
  /// `密码导出`功能的状态（三个子页：游戏存档 / 密码文本 / 内存转储）
  pub c1: PasswordExportState,
  /// 三个子页共享的已载入密码来源：存档、文本，内存转储各写自己的槽位，
  /// 注入密码功能与密码信息预览窗口是读方
  pub sources: PasswordSources,
  // 2. 存档转换功能部分
  /// `存档转换`功能的状态
  pub c2: SaveConvertState,
  // 3. 自定义角色名部分
  /// 主角团7人的自定义名字
  pub new_pc_names: Vec<String>,
  pub out_log_c3: String,
  // 4. 打汉化补丁功能部分
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
      requests: Vec::new(),
      err_modal_open: false,
      err_msg: String::new(),
      // 1. 密码导出功能部分
      c1: PasswordExportState::default(),
      sources: PasswordSources::default(),
      // 2. 存档转换功能部分
      c2: SaveConvertState::default(),
      // 3. 自定义角色名部分
      new_pc_names: vec![String::new(); 7],
      out_log_c3: String::new(),
      // 4. 打汉化补丁功能部分
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
  /// 当前功能是否可以运行（【运行】按钮与Ctrl+Enter快捷键共用）
  pub fn can_run(&self) -> bool {
    match self.command {
      // 1. 密码导出功能部分
      CommandEnum::First => self.c1.can_run(&self.sources),
      // 2. 存档转换功能部分
      CommandEnum::Second => self.c2.can_run(),
      // 3. 自定义角色名部分
      CommandEnum::Third => {
        let mut bool1 = false;
        for ele in self.new_pc_names.clone() {
          if !ele.is_empty() {
            bool1 = true;
          }
        }
        bool1
      }
      CommandEnum::Fourth => !self.input_file_path_c4.is_empty(),
      CommandEnum::Fifth => {
        !self.input_file_path_c5.is_empty()
          && (self.pwd_src[0].is_some() || self.pwd_src[1].is_some() || self.pwd_src[2].is_some())
      }
    }
  }

  /// 运行`command`对应的功能（【运行】按钮请求与Ctrl+Enter快捷键的共用）
  pub fn run_command(&mut self, command: CommandEnum) {
    match command {
      CommandEnum::First => {
        let result = self.c1.run(&mut self.sources);
        if let Err(msg) = result {
          self.fail(msg);
        }
      }
      CommandEnum::Second => {
        let result = self.c2.run();
        if let Err(msg) = result {
          self.fail(msg);
        }
      }
      CommandEnum::Third => self.run_c3(),
      CommandEnum::Fourth => self.run_c4(),
      CommandEnum::Fifth => self.run_c5(),
    }
  }
}

impl eframe::App for App {
  /// 帧级的工作流程：在**本帧**的`ui()`之前执行，目前负责两类事：
  /// 1. 消费**上一帧**在`ui()`侧发送登记的跨功能请求（`self.requests`）：运行当前功能/打开密码信息预览/重置联动……
  /// 2. 键盘的快捷键。
  ///
  /// 即`logic()`是响应请求和输入事件的唯一入口；
  /// 各功能之后会有自己对应的状态（State），并各自有自己的`logic()`；
  /// 一帧内 `logic()` 先于 `ui()`，所以本帧登记的请求一定由**下一帧**的`logic()`消费。
  fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    // Ctrl+Enter 快捷键
    if self.can_run()
      && ctx.input_mut(|input| {
        input.consume_shortcut(&egui::KeyboardShortcut::new(
          egui::Modifiers::COMMAND,
          egui::Key::Enter,
        ))
      })
    {
      self.run_command(self.command);
    }

    // 消费上一帧在ui()登记的跨功能请求
    // 使用std::mem::take取出来“消费”
    let requests = std::mem::take(&mut self.requests);
    for request in requests {
      match request {
        AppRequest::Run(command) => self.run_command(command),
        AppRequest::OpenPasswordPreview(sub) => self.read_from(sub),
        AppRequest::ResetPasswordSources(sub) => {
          for i in 0..3 {
            if self.pwd_src[i] == Some(sub) {
              self.pwd_src[i] = None;
              self.pwd_src_slot[i] = None;
            }
          }
        }
      }
    }

    // 各功能自己待办的逻辑处理（例如各功能的选文件后的读取/解析）；
    if let Err(msg) = self.c1.logic(&mut self.sources) {
      self.fail(msg);
    }
    if let Err(msg) = self.c2.logic() {
      self.fail(msg);
    }
  }

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
