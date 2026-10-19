//! “存档转换”功能状态：本页的界面字段 + 本页的待办request + 最后一次运行的日志。
//!
//! ui侧只能经`request()`追加待办的request；
//! 唯一进行IO操作的地方是`logic()`与`run()`。

use std::path::PathBuf;
use std::{fmt::Write as _, path::Path};

use crate::{
  core::{
    enums::{GameRegion, GameTitle, NameType},
    save::{
      convert_save,
      raw::{GameInfo, parse_convertible_save},
    },
  },
  io::{read_bytes, resolve_output_dir, suffixed_file_name, write_named_bytes},
};

/// “存档转换”功能状态
#[derive(Debug)]
pub struct SaveConvertState {
  /// 输入存档路径
  pub input_file_path: String,
  /// 输入存档的原始字节（读取存档文件时写入）
  pub save_bytes: Vec<u8>,
  /// 识别出的游戏类型与语言版本
  pub source: Option<GameInfo>,
  /// 名字语言版本（`None` = 不改名字）
  pub default_name_ver: Option<NameType>,
  /// 构建日期版本（`None` = 不改日期）
  pub build_date_ver: Option<GameRegion>,
  /// 指定输出目录（空 = 与输入文件同级）
  pub out_dir: String,
  /// 最近一次运行的日志（空 = 尚未运行）
  log: String,
  /// 本页待办request（私有：ui侧使用`request()`追加）
  pending: Vec<SaveConvertRequest>,
}

/// 本页的待办request；
/// 只用本页自己的状态就能完成的动作
#[derive(Debug, Clone)]
pub enum SaveConvertRequest {
  /// 读取并识别用户选中的存档文件（读文件必须等 `logic()`）
  Load(PathBuf),
}

impl SaveConvertState {
  /// ui侧使用，用于登记一条待办request，由下一帧的`logic()`处理
  pub fn request(&mut self, request: SaveConvertRequest) {
    self.pending.push(request);
  }

  /// 本页是否可运行（来源已识别 并且至少选了一个转换选项）
  pub fn can_run(&self) -> bool {
    let bool1 = self.source.is_some();
    let bool2 = self.default_name_ver.is_some() || self.build_date_ver.is_some();

    bool1 && bool2
  }

  /// 是否有输出日志（判断是否可复制日志内容到剪贴板）
  pub fn has_output(&self) -> bool {
    !self.log.is_empty()
  }

  /// 日志文本
  pub fn log(&self) -> &str {
    &self.log
  }

  /// 重置本页
  ///
  /// 纯内存改动，因此可在ui侧直接调用（清路径、清字节、清游戏信息源、清选项）
  pub fn reset(&mut self) {
    self.input_file_path.clear();
    self.save_bytes.clear();
    self.source = None;

    self.reset_options();
  }

  /// 成功载入后把两个选项复位为默认
  fn reset_options(&mut self) {
    self.build_date_ver = None;
    self.default_name_ver = None;
  }

  /// 逻辑侧，处理上一帧ui侧提交的本功能的待办请求
  ///
  /// 读文件 → 解析 → 记录输入文件路径 → 写入到存档来源`save_bytes` → 复位选项。
  /// 失败以`Err`将信息交由`App::logic()`中的`fail()进行。
  pub fn logic(&mut self) -> Result<(), String> {
    let pending = std::mem::take(&mut self.pending);
    for request in pending {
      match request {
        SaveConvertRequest::Load(path) => {
          // 验证存档文件
          let (save_vec, game_info) = Self::load_input(&path).map_err(|e| e)?;
          self.input_file_path = path.to_string_lossy().to_string();
          self.save_bytes = save_vec;
          self.source = Some(game_info);

          self.reset_options();
        }
      }
    }

    Ok(())
  }

  /// 读取并识别用户选中的存档文件
  fn load_input(path: &PathBuf) -> Result<(Vec<u8>, GameInfo), String> {
    let bytes = read_bytes(path, None).map_err(|e| e.to_string())?;
    let info = parse_convertible_save(&bytes).map_err(|e| e.to_string())?;
    Ok((bytes, info))
  }

  /// 运行本功能（由`AppRequest::Run`调用）
  pub fn run(&mut self) -> Result<(), String> {
    // Convert save data.
    let mut new_save_file = self.save_bytes.clone();
    convert_save(
      &mut new_save_file,
      self.source.unwrap().title,
      self.source.unwrap().first_slot_index,
      self.default_name_ver,
      self.build_date_ver,
    );

    // Start to create and write output save file.
    let input_path = Path::new(&self.input_file_path);
    let file_name = suffixed_file_name(input_path, "转换");
    let output_dir = resolve_output_dir(
      input_path,
      (!self.out_dir.is_empty()).then(|| Path::new(&self.out_dir)),
    );
    if let Err(e) = write_named_bytes(&output_dir, &file_name, &new_save_file) {
      return Err(e.to_string());
    }

    self.log = render_log(
      &self.source,
      &output_dir,
      &file_name,
      self.default_name_ver,
      self.build_date_ver,
    );

    Ok(())
  }
}

/// 生成运行概况日志。
fn render_log(
  source: &Option<GameInfo>,
  output_dir: &Path,
  file_name: &str,
  default_name_ver: Option<NameType>,
  build_date_ver: Option<GameRegion>,
) -> String {
  let mut log = String::new();
  let _ = writeln!(
    log,
    "输出文件：\n{}\n\n",
    output_dir.join(file_name).to_str().unwrap()
  );
  log.push_str("运行概况：\n");

  let mut result_index = 1;
  let game_type_cn = match source.as_ref().unwrap().title {
    GameTitle::TheBrokenSeal => "《黄金太阳 开启的封印》",
    GameTitle::TheLostAge => "《黄金太阳 失落的时代》",
  };
  let _ = writeln!(
    log,
    "{result_index}. 输入的源文件为{game_type_cn}的存档文件。"
  );

  if let Some(name_type) = default_name_ver {
    result_index += 1;
    let name_type_cn = name_type.name();
    let _ = writeln!(
      log,
      "{result_index}. 生成的存档文件已将主角团成员的名字转换为{name_type_cn}版的默认名。"
    );

    if let Some(build_date_type) = build_date_ver {
      result_index += 1;
      let build_date_type_cn = build_date_type.short_name();
      let _ = writeln!(
        log,
        "{result_index}. 生成的存档文件已将构建日期转换为{build_date_type_cn}版。"
      );
    }
  }

  log
}

impl Default for SaveConvertState {
  fn default() -> Self {
    Self {
      input_file_path: String::new(),
      save_bytes: Vec::new(),
      source: None,
      default_name_ver: None,
      build_date_ver: None,
      out_dir: String::new(),
      log: String::new(),
      pending: Vec::new(),
    }
  }
}
