//! 视图调用约定：一帧内不变的上下文、页面产生的跨功能请求。
//!
//! 页面想处理的跨功能动作（运行 / 预览 / 重置联动）登记成 `AppRequest`；
//! 入队（`App::requests`） -> 消费（`App::logic()`）。
//! 一帧内 `logic()` 先于 `ui()`，所以本帧登记的请求一定由**下一帧**的 `logic()` 消费。
//! 而纯内存改动（勾选、切页签、清空字段）直接在 `ui()` 里改。
//! 例外：`ui.copy_text`是egui框架的命令，不视为违规IO。

use egui::Color32;

use crate::{
  logic::{
    command::{CommandEnum, SubCommandEnum},
    save_slot::SaveSlotEnum,
  },
  ui::widgets::colors::{get_default_svg_color, get_default_text_color},
};

/// 一帧内不变的、来自 egui 的上下文
/// 目前用于处理svg图片的尺寸和颜色以和主题颜色同步……
#[derive(Debug, Clone, Copy)]
pub struct FrameCtx {
  pub svg_icon_size: f32,
  is_dark: bool,
}

impl FrameCtx {
  pub fn new(svg_icon_size: f32, is_dark: bool) -> Self {
    Self {
      svg_icon_size,
      is_dark,
    }
  }

  pub fn svg_color(self) -> Color32 {
    get_default_svg_color(self.is_dark)
  }

  pub fn text_color(self) -> Color32 {
    get_default_text_color(self.is_dark)
  }
}

/// 页面绘制时产生的、需要 `App` 处理的**跨功能**请求（入队，下一帧 `App::logic()` 消费）
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppRequest {
  /// 运行当前功能（主面板【运行】按钮 / Ctrl+Enter）
  Run(CommandEnum),
  /// 打开`密码信息预览`窗口（`密码导出功能`的三个子页都有这个按钮）；
  OpenPasswordPreview(SubCommandEnum, Option<SaveSlotEnum>),
  /// `注入密码`功能中引用了`密码信息预览`来源的密码来源选择（对应`密码导出功能`三个子页重置时）
  /// `密码信息预览`重置时也会同步重置`注入密码`的密码源的选择
  ResetPasswordSources(SubCommandEnum),
}
