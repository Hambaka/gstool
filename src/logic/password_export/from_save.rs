//! ① 游戏存档子页状态：本页的界面字段 + 本页的待办request + 最后一次运行的日志。
//!
//! ui侧只能经`request()`追加待办的request；
//! 唯一进行IO操作的地方是`logic()`与`run()`。
//! `run`分为生成计划 → 执行 → 生成日志三步：`build_job`→ `execute`(IO)→ `render_log`。

use std::{
  fmt::Write as _,
  path::{Path, PathBuf},
};

use crate::{
  core::{
    enums::{GameRegion, PasswordGrade, PasswordVersion},
    password::encode::from_raw_save,
    save::raw::parse_save_slots,
  },
  io::{
    derived_output_dir, read_bytes, sheet_from_password_bytes, sheet_from_save_slot, slot_dir_name,
  },
  logic::{
    password_export::shared::{ExportOptions, Subject, output_notes, write_outputs},
    sources::PasswordSources,
  },
};

/// ① 游戏存档子页状态
#[derive(Debug)]
pub struct FromSaveState {
  /// 存档文件路径
  pub input_file_path: String,
  /// 是否导出全部存档（无论是否通关）
  pub export_all_saves: bool,
  /// 目标密码级别
  pub grade: PasswordGrade,
  /// 密码文本版本（`None` = 默认“英文”）
  pub text_version: Option<PasswordVersion>,
  /// 是否生成内存转储文件
  pub export_dump: bool,
  /// 密码金手指版本
  pub cheat_region: Option<GameRegion>,
  /// 是否导出密码数据至文本文件
  pub export_data_text: bool,
  /// 自定义输出目录（空 = 与输入文件同级）
  pub out_dir: String,
  /// 最近一次运行的日志（空 = 尚未运行）
  log: String,
  /// 本页待办request（私有：ui侧使用`request()`追加）
  pending: Vec<FromSaveRequest>,
}

/// 本页的待办request；
/// 只用本页自己的状态，和共享的`sources`就能完成的动作
#[derive(Debug, Clone)]
pub enum FromSaveRequest {
  /// 读取并解析用户选中的存档文件（读文件必须等 `logic()`）
  Load(PathBuf),
}

impl FromSaveState {
  /// ui侧使用，用于登记一条待办request，由下一帧的`logic()`处理
  pub fn request(&mut self, request: FromSaveRequest) {
    self.pending.push(request);
  }

  /// 本页是否可运行（来源已载入，并且至少勾了一项输出）
  pub fn can_run(&self, sources: &PasswordSources) -> bool {
    let bool1 = sources.has_save();
    let bool2 = self.text_version.is_some()
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
    self.export_all_saves = false;
    self.input_file_path.clear();
    sources.reset_save();
    self.reset_options();
  }

  /// 成功载入存档后把五个输出选项复位为默认（金 / 英文 / 其余全关）
  fn reset_options(&mut self) {
    self.grade = PasswordGrade::Gold;
    self.text_version = Some(PasswordVersion::English);
    self.export_dump = false;
    self.cheat_region = None;
    self.export_data_text = false;
  }

  /// 逻辑侧，处理上一帧ui侧提交的本子页的待办请求
  ///
  /// 读文件 → 解析槽位 → 写入到密码来源`sources.from_save` → 记录输入文件路径 → 复位选项。
  /// 失败以`Err`将信息交由`App::logic()`中的`fail()`进行。
  pub fn logic(&mut self, sources: &mut PasswordSources) -> Result<(), String> {
    let pending = std::mem::take(&mut self.pending);
    for request in pending {
      match request {
        FromSaveRequest::Load(path) => {
          // 验证存档文件
          let bytes = read_bytes(&path, None).map_err(|e| e.to_string())?;
          let save_slots =
            parse_save_slots(&bytes, self.export_all_saves).map_err(|e| e.to_string())?;

          self.input_file_path = path.to_string_lossy().to_string();
          sources.save = save_slots;
          self.reset_options();
        }
      }
    }
    Ok(())
  }

  /// 运行本子页功能（由`AppRequest::Run`调用）：计划 → 执行 → 日志
  pub fn run(&mut self, sources: &mut PasswordSources) -> Result<(), String> {
    let job = build_job(self, sources);
    execute(&job)?;
    self.log = render_log(&job);
    Ok(())
  }
}

/// 一次运行的计划：输出目录 + 输出选项 + 每个槽位预先算好的结果
struct ExportJob {
  /// 输出目录（用户指定，或由输入文件派生）
  output_dir: PathBuf,
  /// 「导出全部存档」是否勾选（只影响日志措辞）
  export_all_saves: bool,
  options: ExportOptions,
  /// 只有数据的槽位（计算结果已备好，`execute` 里只有 4 个写盘分支）
  slots: Vec<SlotJob>,
}

/// 每个槽位的计算结果
struct SlotJob {
  slot_number: usize,
  is_clear_data: bool,
  password_bytes: Vec<u8>,
  /// 数据文本（先算好，再在`execute`里直接写）
  data_sheet: Option<String>,
}

/// 把每个槽位要生成哪些输出文件先算好
fn build_job(state: &FromSaveState, sources: &PasswordSources) -> ExportJob {
  let output_dir = if state.out_dir.is_empty() {
    derived_output_dir(Path::new(&state.input_file_path))
  } else {
    PathBuf::from(&state.out_dir)
  };

  let options = ExportOptions {
    grade: state.grade,
    text_version: state.text_version,
    export_dump: state.export_dump,
    cheat_region: state.cheat_region,
    export_data_text: state.export_data_text,
  };

  let slots = sources
    .save
    .slots
    .iter()
    .enumerate()
    .filter_map(|(slot_number, val_option)| {
      val_option.as_ref().map(|val| {
        let password_bytes = from_raw_save(&val.data, options.grade);
        SlotJob {
          is_clear_data: val.is_clear_data,
          data_sheet: if options.export_data_text {
            // 金级别直接读存档槽位，其他级别从密码字节换算
            Some(if options.grade == PasswordGrade::Gold {
              sheet_from_save_slot(&val.data)
            } else {
              sheet_from_password_bytes(&password_bytes, options.grade)
            })
          } else {
            None
          },
          slot_number,
          password_bytes,
        }
      })
    })
    .collect();

  ExportJob {
    output_dir,
    export_all_saves: state.export_all_saves,
    options,
    slots,
  }
}

/// 把四个可选的输出文件写到每个槽位存档数据的子目录
fn execute(job: &ExportJob) -> Result<(), String> {
  for slot in &job.slots {
    // Key is save slot number: 0, 1, 2 -> 1, 2, 3
    let sub_dir_str = job
      .output_dir
      .join(slot_dir_name(slot.slot_number, slot.is_clear_data));
    write_outputs(
      &slot.password_bytes,
      job.options,
      slot.data_sheet.as_deref(),
      &sub_dir_str,
    )?;
  }
  Ok(())
}

/// 生成运行概况日志。
fn render_log(job: &ExportJob) -> String {
  let mut log = String::new();
  let _ = write!(
    &mut log,
    "输出目录：\n{}\n\n",
    job.output_dir.to_string_lossy()
  );
  log.push_str("运行概况：\n");

  let mut result_index = 1;
  if job.export_all_saves {
    let _ = writeln!(
      &mut log,
      "{result_index}. 已勾选“导出所有存档”，导出包含未通关存档在内的所有存档的数据。"
    );
  } else {
    let _ = writeln!(
      &mut log,
      "{result_index}. 未勾选“导出所有存档”，仅导出通关存档的数据。"
    );
  }

  for note in output_notes(
    job.options,
    Subject::SaveRecord {
      export_all: job.export_all_saves,
    },
  ) {
    result_index += 1;
    let _ = writeln!(&mut log, "{result_index}. {note}");
  }

  log
}

impl Default for FromSaveState {
  fn default() -> Self {
    Self {
      input_file_path: String::new(),
      export_all_saves: false,
      grade: PasswordGrade::Gold,
      text_version: Some(PasswordVersion::English),
      export_dump: false,
      cheat_region: None,
      export_data_text: false,
      out_dir: String::new(),
      log: String::new(),
      pending: Vec::new(),
    }
  }
}
