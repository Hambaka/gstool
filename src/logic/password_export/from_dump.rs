//! ③ 内存转储子页状态：本页的界面字段 + 本页的待办request + 最后一次运行的日志。
//!
//! ui侧只能经`request()`追加待办的request；
//! 唯一进行IO操作的地方是`logic()`与`run()`。
//! `run`分为生成计划 → 执行 → 生成日志三步：`build_job`→ `execute`(IO)→ `render_log`。
//!
//! 本子页**没有**生成内存转储文件的选框
//! 内存转储文件的输出由`grade.is_some()`隐含触发（选了降级级别才生成）。

use std::{
  fmt::Write as _,
  path::{Path, PathBuf},
};

use crate::{
  core::{
    enums::{GameRegion, PasswordGrade, PasswordVersion},
    password::{
      decode::{parse_password_bytes, read_from_bits},
      encode,
    },
  },
  io::{
    derived_output_dir, read_bytes, sheet_from_password_bytes, sheet_from_password_data,
    write_memory_dump,
  },
  logic::{
    password_export::shared::{ExportOptions, Subject, output_notes, write_outputs},
    sources::{LoadedPassword, PasswordSources},
  },
};

/// ③ 内存转储子页状态
#[derive(Debug, Default)]
pub struct FromDumpState {
  /// 内存转储文件路径
  pub input_file_path: String,
  /// 目标密码级别（仅用于降级；`None` = 与源同级）
  pub grade: Option<PasswordGrade>,
  /// 密码文本版本（`None` = 不生成文本文件）
  pub text_version: Option<PasswordVersion>,
  /// 密码金手指版本
  pub cheat_region: Option<GameRegion>,
  /// 是否导出密码数据至文本文件
  pub export_data_text: bool,
  /// 自定义输出目录（空 = 与输入文件同级）
  pub out_dir: String,
  /// 最近一次运行的日志（空 = 尚未运行）
  log: String,
  /// 本页待办request（私有：ui侧使用`request()`追加）
  pending: Vec<FromDumpRequest>,
}

/// 本页的待办request；
/// 只用本页自己的状态，和共享的`sources`就能完成的动作
#[derive(Debug, Clone)]
pub enum FromDumpRequest {
  /// 读取并解析用户选中的内存转储文件（读文件必须等`logic()`）
  Load(PathBuf),
}

impl FromDumpState {
  /// ui侧使用，用于登记一条待办request，由下一帧的`logic()`处理
  pub fn request(&mut self, request: FromDumpRequest) {
    self.pending.push(request);
  }

  /// 本页是否可运行（来源已载入，并且至少勾了一项输出）
  pub fn can_run(&self, sources: &PasswordSources) -> bool {
    let bool1 = sources.has_dump();
    let bool2 = self.grade.is_some()
      || self.text_version.is_some()
      || self.cheat_region.is_some()
      || self.export_data_text;

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

  /// 重置本页（纯内存改动，因此可在ui侧直接调用）
  ///
  /// 清空`注入密码`中`pwd_src`的联动由`AppRequest::ResetPasswordSources`负责（ui侧另行登记）。
  pub fn reset(&mut self, sources: &mut PasswordSources) {
    self.input_file_path.clear();
    sources.reset_dump();
    self.reset_options();
  }

  /// 成功载入来源后把四个输出选项复位为默认
  fn reset_options(&mut self) {
    self.grade = None;
    self.text_version = None;
    self.cheat_region = None;
    self.export_data_text = false;
  }

  /// 逻辑侧，处理上一帧ui侧提交的本子页的待办请求
  ///
  /// 读文件 → 解析 → 写入到密码来源`sources.from_dump` → 记录输入文件路径 → 复位选项。
  /// 失败以`Err`将信息交由`App::logic()`中的`fail()`进行。
  pub fn logic(&mut self, sources: &mut PasswordSources) -> Result<(), String> {
    let pending = std::mem::take(&mut self.pending);
    for request in pending {
      match request {
        FromDumpRequest::Load(path) => {
          // 验证内存转储文件
          let bytes = read_bytes(&path, Some(0x10_0000)).map_err(|e| e.to_string())?;
          let parsed_password_bytes = parse_password_bytes(bytes).map_err(|e| e.to_string())?;

          self.input_file_path = path.to_string_lossy().to_string();
          sources.dump = Some(LoadedPassword {
            bytes: parsed_password_bytes.bytes,
            bits: parsed_password_bytes.bits,
            grade: parsed_password_bytes.grade,
            version: None,
            text: None,
          });
          self.reset_options();
        }
      }
    }
    Ok(())
  }

  /// 运行本子页功能（由`AppRequest::Run`调用）：计划 → 执行 → 日志
  pub fn run(&mut self, sources: &mut PasswordSources) -> Result<(), String> {
    let job = build_job(self, sources)?;
    execute(&job)?;
    self.log = render_log(&job);
    Ok(())
  }
}

/// 一次运行的计划：输出目录 + 每个输出文件预先算好的内容/参数
struct DumpJob {
  /// 输出目录（用户指定，或与输入文件同级）
  output_dir: PathBuf,
  /// 来源密码的级别（日志措辞用）
  src_grade: PasswordGrade,
  /// 目标级别（`None` = 与源同级；日志措辞用）
  target_grade: Option<PasswordGrade>,
  /// 目标密码字节
  password_bytes: Vec<u8>,
  /// 文本文件的目标版本（`None` = 不写文本文件）
  text_version: Option<PasswordVersion>,
  /// 金手指文件（级别, 版本）
  cheat: Option<(PasswordGrade, GameRegion)>,
  /// 数据文本
  data_sheet: Option<String>,
}

/// 把要生成哪些输出文件先算好
fn build_job(state: &FromDumpState, sources: &PasswordSources) -> Result<DumpJob, String> {
  let src = sources
    .dump
    .as_ref()
    .ok_or_else(|| "尚未载入内存转储来源".to_string())?;

  // Generate save data, and set target password bytes
  let password_data = read_from_bits(&src.bits, src.grade);
  let password_bytes = match state.grade {
    Some(password_grade) => encode(&password_data, password_grade),
    None => src.bytes.clone(),
  };

  let output_dir = if state.out_dir.is_empty() {
    derived_output_dir(Path::new(&state.input_file_path))
  } else {
    PathBuf::from(&state.out_dir)
  };

  // 金手指文件：级别是降级目标（未降级则为来源级别）
  let cheat = state
    .cheat_region
    .map(|region| (state.grade.unwrap_or(src.grade), region));

  // 数据文本
  let data_sheet = if state.export_data_text {
    Some(if let Some(val) = state.grade {
      sheet_from_password_bytes(&password_bytes, val)
    } else {
      sheet_from_password_data(&password_data)
    })
  } else {
    None
  };

  Ok(DumpJob {
    output_dir,
    src_grade: src.grade,
    target_grade: state.grade,
    password_bytes,
    text_version: state.text_version,
    cheat,
    data_sheet,
  })
}

/// 把四个可选的输出文件写到每个槽位存档数据的子目录；
/// 内存转储单独处理，其余三个（密码 + 金手指 + 数据文本）使用共用的`write_outputs`
fn execute(job: &DumpJob) -> Result<(), String> {
  let dir = &job.output_dir;

  if job.target_grade.is_some()
    && let Err(e) = write_memory_dump(&job.password_bytes, dir)
  {
    return Err(e.to_string());
  }

  let options = export_options_dump(job);

  write_outputs(&job.password_bytes, options, job.data_sheet.as_deref(), dir)
}

/// 生成运行概况日志。
fn render_log(job: &DumpJob) -> String {
  let mut log = String::new();
  let _ = write!(
    &mut log,
    "输出目录：\n{}\n\n",
    job.output_dir.to_string_lossy()
  );
  log.push_str("运行概况：\n");

  let mut result_index = 1;
  let source_password_grade_cn = job.src_grade.short_name();
  let _ = writeln!(
    &mut log,
    "{result_index}. 输入的源文件为{source_password_grade_cn}级别的密码内存转储文件。"
  );

  let mut downgrade_str_option = None;
  if let Some(target_password_grade) = job.target_grade {
    result_index += 1;
    let target_password_grade_cn = target_password_grade.short_name();
    downgrade_str_option = Some(target_password_grade_cn);
    let _ = writeln!(
      &mut log,
      "{result_index}. 已生成降级为{target_password_grade_cn}级别的“密码内存转储.dmp”文件。"
    );
  }

  if let Some(password_version) = job.text_version {
    result_index += 1;
    let password_version_cn = password_version.name();
    if let Some(target_password_grade_cn) = downgrade_str_option {
      let _ = writeln!(
        &mut log,
        "{result_index}. 已生成降级为{target_password_grade_cn}级别的{password_version_cn}版“密码文本.txt”文件。"
      );
    } else {
      let _ = writeln!(
        &mut log,
        "{result_index}. 已生成与源文件同级的{source_password_grade_cn}级别的{password_version_cn}版“密码文本.txt”文件。"
      );
    }
  }

  // 金手指 / 数据文本两条说明（转储与文本两条在上方，另外本子页转储文件的展示顺序在文本之前）
  let options = export_options_dump(job);

  for note in output_notes(
    options,
    Subject::SourceOrDowngrade {
      source_grade: source_password_grade_cn,
      target_grade: downgrade_str_option,
    },
  ) {
    result_index += 1;
    let _ = writeln!(&mut log, "{result_index}. {note}");
  }

  log
}

/// 从`DumpJob`提取`ExportOptions`
fn export_options_dump(job: &DumpJob) -> ExportOptions {
  ExportOptions {
    grade: job.target_grade.unwrap_or(job.src_grade),
    text_version: job.text_version,
    export_dump: false,
    cheat_region: job.cheat.map(|(_, region)| region),
    export_data_text: job.data_sheet.is_some(),
  }
}
