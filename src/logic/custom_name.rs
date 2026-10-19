//! “自定义角色名”功能状态：本页的界面字段 + 本页的待办request + 最后一次运行的日志；
//! 但其实目前根本没有需要处理的IO操作，可能会未来再加
//!
//! 所以规则依旧是ui侧只能经`request()`追加待办的request；
//! 唯一进行IO操作的地方是`logic()`与`run()`。
use crate::{
  core::enums::GameTitle, core::save::pc_name::encode_pc_name, io::render::name_encode_log,
};

/// 主角团 7 位成员的默认中文名。
/// 元素个数和`PC_NAME_RECORD_COUNT[0]`是一致的
pub const PC_NAMES: [&str; 7] = [
  "罗宾",
  "杰拉德",
  "伊万",
  "梅雅莉",
  "加西亚",
  "加斯敏",
  "西芭",
];

/// “自定义角色名”功能状态
#[derive(Debug)]
pub struct CustomNameState {
  /// 7个输入框的自定义名字内容
  pub names: Vec<String>,
  /// 最近一次运行的日志（空 = 尚未运行）
  log: String,
  /// 本页待办request（私有：ui侧使用`request()`追加）
  /// 不过目前会一直为空，只是为将来可能会做的“写进存档”预留
  pending: Vec<CustomNameRequest>,
}

/// 本页的待办request；
/// 只用本页自己的状态就能完成的动作
#[derive(Debug, Clone)]
pub enum CustomNameRequest {
  // TODO
  // 提前预留，也许会考虑未来实现的把自定义名字写进存档：
  // `WriteToSave(PathBuf)`,
}

impl CustomNameState {
  /// ui侧使用，用于登记一条待办request，由下一帧的`logic()`处理
  pub fn request(&mut self, request: CustomNameRequest) {
    self.pending.push(request);
  }

  /// 本页是否可运行（至少有一个名字输入框填写了内容）
  pub fn can_run(&self) -> bool {
    self.names.iter().any(|n| !n.is_empty())
  }

  /// 是否有输出日志（判断是否可复制日志内容到剪贴板）
  pub fn has_output(&self) -> bool {
    !self.log.is_empty()
  }

  /// 日志文本
  pub fn log(&self) -> &str {
    &self.log
  }

  /// 重置本页（但是目前暂时没使用）
  ///
  /// 纯内存改动，视图里直接调（清7个输入框 ）
  pub fn reset(&mut self) {
    for name in &mut self.names {
      name.clear();
    }

    // self.log.clear();
  }

  /// 逻辑侧，处理上一帧ui侧提交的本功能的待办请求
  ///
  /// 不过因为目前没有需要处理的request，所以直接返回Ok
  pub fn logic(&mut self) -> Result<(), String> {
    Ok(())
  }

  /// 运行本功能（由`AppRequest::Run`调用）
  pub fn run(&mut self) -> Result<(), String> {
    let layout = GameTitle::TheBrokenSeal.save_layout();

    let mut custom_entries = Vec::new();
    for (index, pc_name_new) in self.names.iter().enumerate() {
      if pc_name_new.is_empty() {
        continue;
      }

      let encoded = encode_pc_name(pc_name_new);
      let base_address = layout.pc_name_ram_address(index);
      custom_entries.push((PC_NAMES[index], base_address, encoded));
    }

    let entries = custom_entries
      .iter()
      .map(|(name, addr, enc)| (*name, *addr, enc))
      .collect::<Vec<_>>();

    self.log = name_encode_log(&entries);

    Ok(())
  }
}

impl Default for CustomNameState {
  fn default() -> Self {
    Self {
      names: vec![String::new(); 7],
      log: String::new(),
      pending: Vec::new(),
    }
  }
}
