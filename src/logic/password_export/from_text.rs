//! ② 密码文本子页状态：本页的界面字段 + 本页的待办request + 最后一次运行的日志。
//!
//! ui侧只能经`request()`追加待办的request；
//! 唯一进行IO操作的地方是`logic()`与`run()`。
//! `run`分为生成计划 → 执行 → 生成日志三步：`build_job`→ `execute`(IO)→ `render_log`。
//!
//! 本子页的密码文本文件由自己处理（4个条件分支（转换 × 降级）），
//! 不走三个页面共用的 `write_outputs`

use std::{
  env,
  fmt::Write as _,
  path::{Path, PathBuf},
};

use crate::{
  core::{
    enums::{GameRegion, PasswordGrade, PasswordVersion},
    password::{
      decode::read_from_bits,
      encode,
      text::{ParsedPassword, dmp_to_txt, parse_password_text, txt_to_another_version},
    },
  },
  io::{
    clipboard_get_text, derived_output_dir, read_text, sheet_from_password_bytes,
    sheet_from_password_data, write_converted_password_text,
  },
  logic::{
    password_export::shared::{ExportOptions, Subject, output_notes, write_outputs},
    sources::{LoadedPassword, PasswordSources},
  },
};

/// ② 密码文本子页状态
#[derive(Debug, Default)]
pub struct FromTextState {
  /// 密码文本文件路径（来源为剪贴板时为空）
  pub input_file_path: String,
  /// 目标密码级别（仅用于降级；`None` = 与源同级）
  pub grade: Option<PasswordGrade>,
  /// 是否转换密码版本（日 ↔ 英）
  pub to_convert: bool,
  /// 是否生成内存转储文件
  pub export_dump: bool,
  /// 密码金手指版本
  pub cheat_region: Option<GameRegion>,
  /// 是否导出密码数据至文本文件
  pub export_data_text: bool,
  /// 自定义输出目录（空 = 与输入文件同级 / 剪贴板来源时为程序目录）
  pub out_dir: String,
  /// 最近一次运行的日志（空 = 尚未运行）
  log: String,
  /// 本页待办request（私有：ui侧使用`request()`追加）
  pending: Vec<FromTextRequest>,
}

/// 本页的待办request；
/// 只用本页自己的状态，和共享的`sources`就能完成的动作
#[derive(Debug, Clone)]
pub enum FromTextRequest {
  /// 读取并解析用户选中的密码文本文件（读文件必须等`logic()`）
  LoadFile(PathBuf),
  /// 读取并解析剪贴板里的密码文本（读剪贴板必须等`logic()`）
  LoadClipboard,
}

impl FromTextState {
  /// ui侧使用，用于登记一条待办request，由下一帧的`logic()`处理
  pub fn request(&mut self, request: FromTextRequest) {
    self.pending.push(request);
  }

  /// 本页是否可运行（来源已载入，并且至少勾了一项输出）
  pub fn can_run(&self, sources: &PasswordSources) -> bool {
    let bool1 = sources.has_text();
    let bool2 = self.grade.is_some()
      || self.to_convert
      || self.export_dump
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
    sources.reset_text();
    self.reset_options();
  }

  /// 成功载入来源后把五个输出选项复位为默认
  fn reset_options(&mut self) {
    self.grade = None;
    self.to_convert = false;
    self.export_dump = false;
    self.cheat_region = None;
    self.export_data_text = false;
  }

  /// 逻辑侧，处理上一帧ui侧提交的本子页的待办请求
  ///
  /// 从文件或剪贴板读取 → 解析 → 写入到密码来源`sources.from_text` → 记录输入文件路径 → 复位选项。
  /// 失败以`Err`将信息交由`App::logic()`中的`fail()`进行。
  pub fn logic(&mut self, sources: &mut PasswordSources) -> Result<(), String> {
    let pending = std::mem::take(&mut self.pending);
    for request in pending {
      match request {
        FromTextRequest::LoadFile(path) => {
          // 验证密码文本文件
          let text = read_text(&path, 0x10_0000).map_err(|e| e.to_string())?;
          let parsed_password = parse_password_text(&text).map_err(|e| e.to_string())?;

          self.input_file_path = path.to_string_lossy().to_string();
          sources.text = Some(Self::into_loaded(&parsed_password));
          self.reset_options();
        }
        FromTextRequest::LoadClipboard => {
          let text = clipboard_get_text()?;
          let parsed_password = parse_password_text(&text).map_err(|e| e.to_string())?;

          // 剪贴板来源没有输入文件，路径清空
          self.input_file_path.clear();
          sources.text = Some(Self::into_loaded(&parsed_password));
          self.reset_options();
        }
      }
    }
    Ok(())
  }

  /// 把解析结果装进共享的 `LoadedPassword`（文本来源：版本与原文都有）
  fn into_loaded(parsed_password: &ParsedPassword) -> LoadedPassword {
    LoadedPassword {
      bytes: parsed_password.bytes.clone(),
      bits: parsed_password.bits.clone(),
      grade: parsed_password.grade,
      version: Some(parsed_password.version),
      text: Some(parsed_password.text.clone()),
    }
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
struct TextJob {
  /// 输出目录（用户指定 / 与输入文件同级 / `剪贴板来源时为程序目录下的「剪贴板_输出`」）
  output_dir: PathBuf,
  /// 来源密码的级别与版本（日志措辞用）
  src_grade: PasswordGrade,
  src_version: PasswordVersion,
  /// 是否转换版本（日志措辞用）
  to_convert: bool,
  /// 目标级别（`None` = 与源同级；日志措辞用）
  target_grade: Option<PasswordGrade>,
  /// 文本文件要写的内容与目标版本（`None` = 不写文本文件）
  text_output: Option<(String, PasswordVersion)>,
  /// 目标密码字节
  password_bytes: Vec<u8>,
  /// 是否生成内存转储文件
  export_dump: bool,
  /// 金手指文件（级别, 版本）
  cheat: Option<(PasswordGrade, GameRegion)>,
  /// 数据文本
  data_sheet: Option<String>,
}

/// 把要生成哪些输出文件算好
fn build_job(state: &FromTextState, sources: &PasswordSources) -> Result<TextJob, String> {
  // 理论上不会触发
  let src = sources
    .text
    .as_ref()
    .ok_or_else(|| "尚未载入密码文本来源".to_string())?;
  let password_version = src
    .version
    .ok_or_else(|| "密码文本来源缺少版本信息".to_string())?;

  // Generate save data, and set target password bytes
  let password_data = read_from_bits(&src.bits, src.grade);
  let password_bytes = match state.grade {
    Some(password_grade) => encode(&password_data, password_grade),
    None => src.bytes.clone(),
  };

  // Get output directory.
  let output_dir = if state.out_dir.is_empty() {
    if state.input_file_path.is_empty() {
      let current_exe_path =
        env::current_exe().map_err(|_| "无法获取程序所在目录！".to_string())?;
      let parent = current_exe_path
        .parent()
        .ok_or_else(|| "无法获取程序所在目录！".to_string())?;
      parent.join("剪贴板_输出")
    } else {
      derived_output_dir(Path::new(&state.input_file_path))
    }
  } else {
    PathBuf::from(&state.out_dir)
  };

  // 文本文件输出的各情况分支（转换 × 降级）：这里只算要写的文本内容 + 目标版本，
  // 再在`execute`里写入
  let text_output = if state.to_convert {
    if state.grade.is_some() {
      Some((
        dmp_to_txt(&password_bytes, password_version.opposite()),
        password_version.opposite(),
      ))
    } else {
      Some((
        txt_to_another_version(
          src
            .text
            .as_ref()
            .ok_or_else(|| "密码文本来源缺少原文".to_string())?,
          password_version,
        ),
        password_version.opposite(),
      ))
    }
  } else if state.grade.is_some() {
    Some((
      dmp_to_txt(&password_bytes, password_version),
      password_version,
    ))
  } else {
    None
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

  Ok(TextJob {
    output_dir,
    src_grade: src.grade,
    src_version: password_version,
    to_convert: state.to_convert,
    target_grade: state.grade,
    text_output,
    password_bytes,
    export_dump: state.export_dump,
    cheat,
    data_sheet,
  })
}

/// 把四个可选的输出文件写到每个槽位存档数据的子目录；
/// 文本单独处理，其余三个（转储 + 金手指 + 数据文本）使用共用的`write_outputs`
fn execute(job: &TextJob) -> Result<(), String> {
  let dir = &job.output_dir;

  if let Some((text, version)) = &job.text_output
    && let Err(e) = write_converted_password_text(text, *version, dir)
  {
    return Err(e.to_string());
  }

  let options = export_options_text(job);

  write_outputs(&job.password_bytes, options, job.data_sheet.as_deref(), dir)
}

/// 生成运行概况日志。
fn render_log(job: &TextJob) -> String {
  let mut log = String::new();
  let _ = write!(
    &mut log,
    "输出目录：\n{}\n\n",
    job.output_dir.to_string_lossy()
  );
  log.push_str("运行概况：\n");

  let mut result_index = 1;
  let source_password_grade_cn = job.src_grade.short_name();
  let source_password_version_cn = job.src_version.name();

  let _ = writeln!(
    &mut log,
    "{result_index}. 输入的源文件为{source_password_grade_cn}级别的{source_password_version_cn}版密码文本文件。"
  );

  let convert_flag_str_option = if job.to_convert {
    Some(job.src_version.opposite().name())
  } else {
    None
  };

  let mut downgrade_str_option = None;
  if let Some(target_password_grade) = job.target_grade {
    let target_password_grade_cn = target_password_grade.short_name();
    downgrade_str_option = Some(target_password_grade_cn);
  }

  match convert_flag_str_option {
    Some(convert_flag_str) => {
      if let Some(downgrade_str) = downgrade_str_option {
        result_index += 1;
        let _ = writeln!(
          &mut log,
          "{result_index}. 已生成降级为{downgrade_str}级别的、转换为{convert_flag_str}版的“密码文本.txt”文件。"
        );
      } else {
        result_index += 1;
        let _ = writeln!(
          &mut log,
          "{result_index}. 已生成与源文件同级的{source_password_grade_cn}级别的、转换为{convert_flag_str}版的“密码文本.txt”文件。"
        );
      }
    }
    None => {
      if let Some(downgrade_str) = downgrade_str_option {
        result_index += 1;
        let _ = writeln!(
          &mut log,
          "{result_index}. 已生成降级为{downgrade_str}级别的、未经版本转换的{source_password_version_cn}版“密码文本.txt”文件。"
        );
      }
    }
  }

  // 转储 / 金手指 / 数据文本三条说明（文本文件那条在上方，由 4 分支单独处理）
  let options = export_options_text(job);

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

/// 从`TextJob`提取`ExportOptions`
fn export_options_text(job: &TextJob) -> ExportOptions {
  ExportOptions {
    grade: job.target_grade.unwrap_or(job.src_grade),
    text_version: None,
    export_dump: job.export_dump,
    cheat_region: job.cheat.map(|(_, region)| region),
    export_data_text: job.data_sheet.is_some(),
  }
}
