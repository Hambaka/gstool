use std::collections::HashMap;

use egui::{ColorImage, Image, Shadow, TextStyle, TextureHandle, TextureOptions};
use egui_notify::{Anchor, Toasts};
use image::GenericImageView;

use crate::{
  logic::{
    command::CommandEnum, custom_name::CustomNameState, inject_pwd::InjectPwdState,
    password_export::PasswordExportState, patch_rom::PatchRomState, preview::PwdInfoState,
    save_convert::SaveConvertState, sources::PasswordSources,
  },
  ui::{
    error_modal::show_error_modal,
    page::{AppRequest, FrameCtx},
    top_panel::show_top_panel,
    window::{
      about::{AboutState, show_about_window},
      pwd_info::show_pwd_info_window,
    },
  },
};

const ITEM_ICON_COUNT: usize = 362;
const ITEM_ICON_SIZE: u32 = 16;
const ITEM_ICON_COLS: usize = 10;

#[derive(Default)]
pub struct KeyboardCache(pub HashMap<u8, (TextureHandle, [f32; 2])>);

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
  pub preview: PwdInfoState,
  pub item_icon_img: Vec<TextureHandle>,

  pub about: AboutState,
  pub toasts: Toasts,
  pub command: CommandEnum,
  /// 跨功能的请求队列：先在ui侧发送请求进行登记，再到下一帧由`App::logic()`消费（约定见`ui/page.rs`）
  pub requests: Vec<AppRequest>,
  pub err_modal_open: bool,
  pub err_msg: String,
  /// 1. `密码导出`功能的状态（三个子页：游戏存档 / 密码文本 / 内存转储）
  pub c1: PasswordExportState,
  /// 三个子页共享的已载入密码来源：存档、文本，内存转储各写自己的槽位，
  /// 注入密码功能与密码信息预览窗口是读方
  pub sources: PasswordSources,
  /// 2. `存档转换`功能的状态
  pub c2: SaveConvertState,
  /// 3. `自定义角色名`功能的状态
  pub c3: CustomNameState,
  /// 自定义角色名功能页的键盘按键图缓存
  pub kb_cache: KeyboardCache,
  /// 4. `应用汉化补丁`功能的状态
  pub c4: PatchRomState,
  /// 5. `注入密码`功能的状态
  pub c5: InjectPwdState,
}

impl App {
  #[must_use]
  pub fn name() -> String {
    // env!("CARGO_PKG_VERSION")
    // env!("CARGO_PKG_NAME")
    format!("GSTool - {}", env!("CARGO_PKG_VERSION"))
  }

  #[must_use]
  pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
    // 1. 设置道具图标图片
    // 加载道具图标图片
    let image_bytes = include_bytes!("../assets/item_icon_img/all_items.png");
    let handles = {
      let mut handle_vec = Vec::new();

      let image =
        image::load_from_memory_with_format(image_bytes, image::ImageFormat::Png).unwrap();

      let icon_size = ITEM_ICON_SIZE;
      let columns = ITEM_ICON_COLS;

      for id in 0..ITEM_ICON_COUNT {
        // 当前图标所在列
        let col = id % columns;
        // 当前图标所在行
        let row = id / columns;

        // 起始像素坐标
        let x = (col as u32) * icon_size;
        let y = (row as u32) * icon_size;

        // 从完整大图中获取该图标的 16x16 的图像
        let sub_image = image.view(x, y, icon_size, icon_size);
        // 开始转换为egui的TextureHandle
        let rgba_image = sub_image.to_image();
        let pixels = rgba_image.into_raw();
        let color_image = {
          let icon_size = ITEM_ICON_SIZE as usize;
          ColorImage::from_rgba_unmultiplied([icon_size, icon_size], &pixels)
        };

        let handle = cc.egui_ctx.load_texture(
          format!("item_icon_{id}"),
          color_image,
          TextureOptions::NEAREST,
        );

        handle_vec.push(handle);
      }

      handle_vec
    };

    Self {
      preview: PwdInfoState::default(),
      item_icon_img: handles,

      about: AboutState { window_open: false },
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
      c3: CustomNameState::default(),
      kb_cache: KeyboardCache::default(),
      // 4. 打汉化补丁功能部分
      c4: PatchRomState::default(),
      // 5. 注入密码功能部分
      c5: InjectPwdState::default(),
    }
  }
}

impl App {
  /// 当前功能是否可以运行（【运行】按钮与Ctrl+Enter快捷键共用）
  #[must_use]
  pub fn can_run(&self) -> bool {
    match self.command {
      // 1. 密码导出功能部分
      CommandEnum::First => self.c1.can_run(&self.sources),
      // 2. 存档转换功能部分
      CommandEnum::Second => self.c2.can_run(),
      CommandEnum::Third => self.c3.can_run(),
      CommandEnum::Fourth => self.c4.can_run(),
      CommandEnum::Fifth => self.c5.can_run(),
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
      CommandEnum::Third => {
        let result = self.c3.run();
        if let Err(msg) = result {
          self.fail(msg);
        }
      }
      CommandEnum::Fourth => {
        let result = self.c4.run();
        if let Err(msg) = result {
          self.fail(msg);
        }
      }
      CommandEnum::Fifth => {
        let result = self.c5.run(&self.sources);
        if let Err(msg) = result {
          self.fail(msg);
        }
      }
    }
  }

  /// 设置错误信息并打开错误Modal窗口。
  pub fn fail(&mut self, err_msg: String) {
    self.err_msg = err_msg;
    self.err_modal_open = true;
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
      && !self.err_modal_open
      && !self.about.window_open
      && !self.preview.window_open
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
        AppRequest::OpenPasswordPreview(sub, slot) => {
          self.preview.load_from(&self.sources, sub, slot);
        }
        AppRequest::ResetPasswordSources(sub) => self.c5.clear_source(sub),
      }
    }

    // 各功能自己待办的逻辑处理（例如各功能的选文件后的读取/解析）；
    if let Err(msg) = self.c1.logic(&mut self.sources) {
      self.fail(msg);
    }
    if let Err(msg) = self.c2.logic() {
      self.fail(msg);
    }
    if let Err(msg) = self.c3.logic() {
      self.fail(msg);
    }
    if let Err(msg) = self.c4.logic() {
      self.fail(msg);
    }
    if let Err(msg) = self.c5.logic() {
      self.fail(msg);
    }
  }

  fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
    let f_ctx = {
      let svg_icon_size = ui.text_style_height(&TextStyle::Body);
      let is_dark = ui.visuals().dark_mode;

      FrameCtx::new(svg_icon_size, is_dark)
    };

    // ui.set_visuals(egui::Visuals::dark());

    // “关于”窗口
    show_about_window(ui, &mut self.about, f_ctx);
    // 密码信息窗口
    show_pwd_info_window(ui, &mut self.preview, &self.item_icon_img, f_ctx);
    // 错误信息显示模态窗口
    if self.err_modal_open {
      show_error_modal(ui, &mut self.err_modal_open, &self.err_msg);
    }
    // 顶部面板，主要用于菜单栏。
    show_top_panel(ui, &mut self.about.window_open, f_ctx);
    // 主面板，核心功能区。
    self.show_main_panel(ui, f_ctx);
    self.toasts.show(ui);
  }
}
