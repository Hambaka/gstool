//! 视图调用约定：一帧内不变的上下文、页面产生的跨功能请求、一帧的绘制结果。
//!
//! 各功能页面统一返回 [`PageOutcome`]，由 `App::apply` 集中处理，
//! 这样页面函数只需要 `&mut <功能状态>` 就能把「可能失败」和「可能产生请求」
//! 两件事归一成一个返回值，不再直接改 `App` 的全局字段。

use crate::app::SubCommandEnum;

/// 一帧内不变的、来自 egui 的上下文
/// 目前用于处理svg图片的尺寸和颜色以和主题颜色同步……
#[derive(Debug, Clone, Copy)]
pub struct FrameCtx {
  pub svg_icon_size: f32,
  pub is_dark: bool,
}

/// 页面绘制时产生的、需要 `App` 处理的**跨功能**请求
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PageRequest {
  /// 打开`密码信息预览`窗口（`密码导出功能`的三个子页都有这个按钮）；
  /// 对应现有的 `App::read_from(sub)`）
  OpenPasswordPreview(SubCommandEnum),
  /// `注入密码`功能中引用了`密码信息预览`来源的密码来源选择（对应`密码导出功能`三个子页重置时）
  /// `密码信息预览`重置时也会同步重置`注入密码`的密码源的选择
  ResetPasswordSources(SubCommandEnum),
}

/// 页面一帧的绘制结果
#[derive(Default)]
pub struct PageOutcome {
  /// 本帧产生的跨功能请求。`Vec::new()` 在不需要 push 时不分配，
  /// 所以每帧一个 `Vec` 是零成本。
  pub requests: Vec<PageRequest>,
  /// 本帧要弹给用户看的错误（至多一个）
  pub error: Option<String>,
}
