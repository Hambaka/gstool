use arboard::Clipboard;

/// 使用arboard访问剪贴板获取文本；
///
/// `而复制到剪贴板由egui自带的ui.copy_text进行`
pub fn clipboard_get_text() -> Result<String, String> {
  let mut clipboard = Clipboard::new().map_err(|_| "访问剪贴板时出现错误！".to_string())?;
  clipboard
    .get_text()
    .map_err(|_| "获取剪贴板的文本时出现错误！".to_string())
}
