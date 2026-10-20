//! “应用汉化补丁”功能的状态：本页的界面字段 + 本页的待办request + 最后一次运行的日志。
//!
//! ui侧只能经`request()`追加待办的request；
//! 唯一进行IO操作的地方是`logic()`与`run()`。

use std::path::PathBuf;
use std::{fmt::Write as _, path::Path};

use crate::{
  core::rom::{
    format::MAX_ROM_SIZE,
    patch::{PatchSource, apply_chinese_patch, rom_display_name},
  },
  io::{
    classify_rom_file, patched_rom_file_name, read_bytes, resolve_output_dir, write_named_bytes,
  },
};

/// 载入的输入ROM的信息（路径与CRC32成对存在）
#[derive(Debug, Clone)]
pub struct RomInput {
  /// 输入ROM的路径
  pub path: String,
  /// 输入ROM的的CRC32（选文件时由 `classify_rom_file` 得到）
  pub crc32: u32,
}

/// “应用汉化补丁”功能状态
#[derive(Debug)]
pub struct PatchRomState {
  /// 输入 ROM（None = 未选择）
  pub input: Option<RomInput>,
  /// 仅用于显示：与 `input` 同步（`file_row` 需要 `&mut String`）
  pub display_path: String,
  /// 指定输出目录（空 = 与输入同级）
  pub out_dir: String,
  /// 最近一次运行的日志（空 = 尚未运行）
  log: String,
  /// 本页待办request（私有：ui侧使用`request()`追加）
  pending: Vec<PatchRomRequest>,
}

/// 本页的待办request；
/// 只用本页自己的状态就能完成的动作
#[derive(Debug, Clone)]
pub enum PatchRomRequest {
  /// 读取并分类用户选中的 ROM（读文件必须等 `logic()`）
  Load(PathBuf),
}

impl PatchRomState {
  /// ui侧使用，用于登记一条待办request，由下一帧的`logic()`处理
  pub fn request(&mut self, request: PatchRomRequest) {
    self.pending.push(request);
  }

  /// 本页是否可运行（ROM 已选择）
  pub fn can_run(&self) -> bool {
    self.input.is_some()
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
  /// 纯内存改动，因此可在ui侧直接调用（清输入、清显示）
  pub fn reset(&mut self) {
    self.input = None;
    self.display_path.clear();
    self.out_dir.clear();

    //self.log.clear();
  }

  /// 逻辑侧，处理上一帧ui侧提交的本功能的待办请求
  /// 由`classify_rom_file`读取ROM文件并解析 → 记录输入文件路径和crc32值
  ///
  /// 失败以`Err`将信息交由`App::logic()`中的`fail()进行。
  pub fn logic(&mut self) -> Result<(), String> {
    let pending = std::mem::take(&mut self.pending);
    for request in pending {
      match request {
        PatchRomRequest::Load(path) => {
          let result = classify_rom_file(&path);
          match result {
            Ok(PatchSource::Patchable { crc32 }) => {
              self.input = Some(RomInput {
                path: path.to_string_lossy().into_owned(),
                crc32,
              });
              self.display_path = path.to_string_lossy().to_string();
            }
            Ok(PatchSource::AlreadyLatestChinese) => {
              return Err("输入的是已经应用了最新版汉化补丁的ROM文件！".to_owned());
            }
            Ok(PatchSource::Unsupported) => {
              return Err("不支持输入的 ROM 文件！".to_owned());
            }
            Err(e) => return Err(e),
          }
        }
      }
    }
    Ok(())
  }

  /// 运行本功能（由`AppRequest::Run`调用）
  pub fn run(&mut self) -> Result<(), String> {
    let Some(input) = self.input.clone() else {
      return Err("请先选择要打补丁的 ROM 文件！".to_owned());
    };

    let rom = read_bytes(Path::new(&input.path), Some(MAX_ROM_SIZE)).map_err(|e| e.to_string())?;
    let patched = apply_chinese_patch(rom).map_err(|e| e.to_string())?;

    let out_dir = resolve_output_dir(
      Path::new(&input.path),
      (!self.out_dir.is_empty()).then(|| Path::new(&self.out_dir)),
    );
    let file_name = patched_rom_file_name();
    let out_file = out_dir.join(&file_name);
    write_named_bytes(&out_dir, &file_name, &patched.bytes).map_err(|e| e.to_string())?;

    self.log = render_log(input.crc32, patched.source_crc32, &out_file);

    Ok(())
  }
}

impl Default for PatchRomState {
  fn default() -> Self {
    Self {
      input: None,
      display_path: String::new(),
      out_dir: String::new(),
      log: String::new(),
      pending: Vec::new(),
    }
  }
}

/// 生成运行概况日志。
fn render_log(input_crc32: u32, source_crc32: u32, out_file: &Path) -> String {
  let mut log = String::new();
  let _ = writeln!(log, "输出文件：\n{}\n\n", out_file.display());
  log.push_str("运行概况：\n");

  let mut result_index = 0;

  if input_crc32 != source_crc32 {
    result_index += 1;
    let _ = writeln!(
      log,
      "{result_index}. 输入的源文件（\"{}\"）和运行前提供的源文件（\"{}\"）不一致！",
      rom_display_name(input_crc32).unwrap(),
      rom_display_name(source_crc32).unwrap()
    )
    .unwrap();
  }

  result_index += 1;
  let _ = writeln!(
    log,
    "{result_index}. 输入的源文件版本信息：{}",
    rom_display_name(input_crc32).unwrap()
  );

  log
}
