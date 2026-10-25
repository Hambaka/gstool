//! “注入密码”功能状态。
//!
//! `logic()`只负责校验ROM文件，`run()` 读ROM进行实际的密码自动输入程序注入和输出注入后的文件。
//! 与`password_export`联动，会处理`ResetPasswordSources`请求。

use std::path::PathBuf;

use crate::{
  core::{
    enums::PasswordGrade,
    password::{
      encode::{encode, from_raw_save},
      read_from_bits,
    },
    rom::inject::{PasswordCombination, TlaRomAnalysis, inject},
  },
  io::{analyze_rom_file, resolve_output_dir, suffixed_file_name, write_named_bytes},
  logic::command::SubCommandEnum,
  logic::save_slot::SaveSlotEnum,
  logic::sources::PasswordSources,
};

/// 一个级别（金/银/铜）的密码数据来源选择
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SourceChoice {
  /// 密码数据来源种类
  pub kind: SubCommandEnum,
  /// 存档槽位;
  /// 所以密码数据源不是存档的话，这里的值都是`None`
  pub slot: Option<SaveSlotEnum>,
}

/// “注入密码”的状态
#[derive(Debug, Default)]
pub struct InjectPwdState {
  /// 输入ROM的路径（空 = 未选择）
  pub input_file_path: String,
  /// 三个级别的来源选择（0/1/2对应金/银/铜）
  pub source: [Option<SourceChoice>; 3],
  /// 指定输出目录（空 = 与输入同级）
  pub out_dir: String,
  /// 最近一次运行的日志（空 = 尚未运行）
  log: String,
  /// 本页待办request（私有：ui侧使用`request()`追加）
  pending: Vec<InjectPwdRequest>,
}

/// 本页的待办
#[derive(Debug, Clone)]
pub enum InjectPwdRequest {
  /// 读取并校验用户选中的ROM（只做校验，不保留读取的字节）
  Load(PathBuf),
}

impl InjectPwdState {
  /// ui侧使用，用于登记一条待办request，由下一帧的`logic()`处理
  pub fn request(&mut self, request: InjectPwdRequest) {
    self.pending.push(request);
  }

  /// 本页是否可运行（ROM已选择，并且至少选了一个密码数据来源）
  pub fn can_run(&self) -> bool {
    !self.input_file_path.is_empty() && self.source.iter().any(Option::is_some)
  }

  /// 是否有输出日志（判断是否可复制日志内容到剪贴板）
  pub fn has_output(&self) -> bool {
    !self.log.is_empty()
  }

  /// 日志文本
  pub fn log(&self) -> &str {
    &self.log
  }

  /// 纯内存改动，视图里直接调（清路径、清来源、清输出目录）
  pub fn reset(&mut self) {
    self.input_file_path.clear();
    self.source = [None, None, None];
    self.out_dir.clear();
  }

  /// 处理`AppRequest::ResetPasswordSources`的请求
  /// 把 `source[i].kind == kind` 的项设为 `None`
  pub fn clear_source(&mut self, kind: SubCommandEnum) {
    for i in 0..3 {
      if let Some(choice) = &self.source[i]
        && choice.kind == kind
      {
        self.source[i] = None;
      }
    }
  }

  /// 逻辑侧，处理上一帧ui侧提交的本功能的待办请求
  ///
  /// 用`analyze_rom_file`输入的ROM文件
  pub fn logic(&mut self) -> Result<(), String> {
    let pending = std::mem::take(&mut self.pending);
    for request in pending {
      match request {
        InjectPwdRequest::Load(path) => {
          let result = analyze_rom_file(&path);
          match result {
            Ok(_) => {
              self.input_file_path = path.to_string_lossy().into_owned();
            }
            Err(e) => return Err(e),
          }
        }
      }
    }
    Ok(())
  }

  /// 运行本功能（由`AppRequest::Run`调用）
  pub fn run(&mut self, sources: &PasswordSources) -> Result<(), String> {
    // 1. 读输入 ROM + 分析
    let path = std::path::Path::new(&self.input_file_path);
    let (rom_bytes, analysis) = analyze_rom_file(path).map_err(|e| e.clone())?;

    // 2. 获得注入类型
    let flags = [
      self.source[0].is_some(),
      self.source[1].is_some(),
      self.source[2].is_some(),
    ];
    let combination =
      PasswordCombination::from_sources(flags).ok_or_else(|| "密码来源组合无效！".to_owned())?;

    // 3. 三个级别的密码字节
    let passwords = [
      Self::password_bytes_for(self.source[0], PasswordGrade::Gold, sources),
      Self::password_bytes_for(self.source[1], PasswordGrade::Silver, sources),
      Self::password_bytes_for(self.source[2], PasswordGrade::Bronze, sources),
    ];

    // 4. 注入并生成新ROM
    let new_rom =
      inject(rom_bytes, &analysis, combination, passwords).map_err(|e| e.to_string())?;

    // 5. 输出文件
    let file_name = suffixed_file_name(path, combination.get_name());
    let out_dir = resolve_output_dir(
      path,
      (!self.out_dir.is_empty()).then(|| std::path::Path::new(&self.out_dir)),
    );
    write_named_bytes(&out_dir, &file_name, &new_rom).map_err(|e| e.to_string())?;

    // 6. 生成日志
    self.log = render_log(&analysis, combination, &out_dir.join(&file_name));

    Ok(())
  }

  /// 按「来源选择 + 目标级别」取出密码字节。三处共用。
  ///
  /// - 存档来源：`from_raw_save` 直接按目标级别编码；
  /// - 文本 / 内存转储来源：源级别 == 目标级别则直接复用字节，否则解码后按目标级别重新编码。
  fn password_bytes_for(
    choice: Option<SourceChoice>,
    target: PasswordGrade,
    sources: &PasswordSources,
  ) -> Option<Vec<u8>> {
    match choice?.kind {
      SubCommandEnum::First => {
        let slot = choice?.slot?;
        let save = sources.from_save.slots[slot.index()].as_ref()?;
        Some(from_raw_save(&save.data, target))
      }
      SubCommandEnum::Second => {
        let src = sources.from_text.as_ref()?;
        Some(if src.grade == target {
          src.bytes.clone()
        } else {
          encode(&read_from_bits(&src.bits, src.grade), target)
        })
      }
      SubCommandEnum::Third => {
        let src = sources.from_dump.as_ref()?;
        Some(if src.grade == target {
          src.bytes.clone()
        } else {
          encode(&read_from_bits(&src.bits, src.grade), target)
        })
      }
    }
  }
}

/// 生成运行概况日志。
fn render_log(
  analysis: &TlaRomAnalysis,
  combination: PasswordCombination,
  out_file: &std::path::Path,
) -> String {
  use std::fmt::Write as _;

  let mut log = String::new();
  let _ = writeln!(log, "输出文件：\n{}\n\n", out_file.display());
  log.push_str("运行概况：\n");

  let mut result_index = 1;

  if analysis.is_known_rom {
    let _ = writeln!(
      log,
      "{result_index}. 输入的源文件版本信息：“{}”（无任何修改）",
      analysis.rom_name
    );
  } else {
    match analysis.already_injected {
      Some(combo) => {
        let _ = writeln!(
          log,
          "{result_index}. 输入的源文件（本质）版本信息：“{}”，（已注入密码，密码类型：{}）",
          analysis.rom_name,
          combo.get_name()
        );
      }
      None => {
        let _ = writeln!(
          log,
          "{result_index}. 输入的源文件（本质）版本信息：“{}”，（未注入密码）",
          analysis.rom_name
        );
      }
    }
  }

  result_index += 1;
  let _ = writeln!(
    log,
    "{result_index}. 本次注入的密码类型为：“{}”",
    combination.get_name()
  );

  log
}
