//! 三个导出页共用的logic侧部分。
//!
//! `ExportOptions`（四段可选输出的选项）+ `write_outputs`（输出导出文件的IO操作）
//! + `Subject` / `output_notes`（运行概况中的“已生成……”说明）。

use std::path::Path;

use crate::{
  core::enums::{GameRegion, PasswordGrade, PasswordVersion},
  io::{
    write_cheat_code, write_memory_dump, write_password_data_text, write_password_text,
  },
};

/// 三个导出页共用的输出选项
#[derive(Debug, Clone, Copy)]
pub struct ExportOptions {
  /// 目标密码级别（金手指文件）
  pub grade: PasswordGrade,
  /// 密码文本版本（`None` = 不生成文本文件）
  pub text_version: Option<PasswordVersion>,
  /// 是否生成内存转储文件
  pub export_dump: bool,
  /// 金手指版本（`None` = 不生成）
  pub cheat_region: Option<GameRegion>,
  /// 是否导出密码数据文本
  pub export_data_text: bool,
}

/// “已生成……”说明，根据情况区分：
/// 1. 使用的是一般的存档记录还是通关的存档记录
/// 2. 是和输入同级别的密码数据，还是降级了的密码数据
#[derive(Debug, Clone, Copy)]
pub enum Subject {
  /// 存档文件子页：“存档记录”/“通关的存档记录”
  SaveRecord { export_all: bool },
  /// 密码文本/内存转储子页：“与源文件同级的{source}”或“降级为 {target}”
  SourceOrDowngrade {
    source_grade: &'static str,
    target_grade: Option<&'static str>,
  },
}

/// 把密码字节按选项写对应的文件到目标目录`dir`重（四段可选输出，三个子页共用）
///
/// 顺序：文本 → 内存转储 → 金手指 → 数据文本。
pub fn write_outputs(
  bytes: &[u8],
  options: &ExportOptions,
  data_sheet: Option<&str>,
  dir: &Path,
) -> Result<(), String> {
  if let Some(password_version) = options.text_version {
    if let Err(e) = write_password_text(bytes, password_version, dir) {
      return Err(e.to_string());
    }
  }

  if options.export_dump {
    if let Err(e) = write_memory_dump(bytes, dir) {
      return Err(e.to_string());
    }
  }

  if let Some(region) = options.cheat_region {
    if let Err(e) = write_cheat_code(bytes, options.grade, region, dir) {
      return Err(e.to_string());
    }
  }

  if let Some(sheet) = data_sheet {
    if let Err(e) = write_password_data_text(sheet, dir) {
      return Err(e.to_string());
    }
  }

  Ok(())
}

/// 生成三子页共用部分的“已生成……”说明
///
/// 固定顺序：文本 → 内存转储 → 金手指 → 数据文本
pub fn output_notes(options: &ExportOptions, subject: Subject) -> Vec<String> {
  let mut notes = Vec::new();
  match subject {
    Subject::SaveRecord { export_all } => {
      let subject_cn = if export_all { "存档记录" } else { "通关的存档记录" };
      let grade_cn = options.grade.short_name();
      if let Some(text_version) = options.text_version {
        notes.push(format!(
          "已生成{subject_cn}所对应的{grade_cn}级别、{}版的“密码文本.txt”文件。",
          text_version.name()
        ));
      }
      if options.export_dump {
        notes.push(format!(
          "已生成{subject_cn}所对应的{grade_cn}级别“密码内存转储.dmp”文件。"
        ));
      }
      if let Some(region) = options.cheat_region {
        notes.push(format!(
          "已生成{subject_cn}所对应的{grade_cn}级别的、适用于{}版《黄金太阳 失落的时代》的“密码金手指.txt”文件。",
          region.short_name()
        ));
      }
      if options.export_data_text {
        notes.push(format!(
          "已生成{subject_cn}所对应的{grade_cn}级别“密码数据.txt”文件"
        ));
      }
    }
    Subject::SourceOrDowngrade {
      source_grade,
      target_grade,
    } => {
      let grade_prefix = match target_grade {
        Some(t) => format!("降级为{t}级别"),
        None => format!("与源文件同级的{source_grade}级别"),
      };
      if let Some(text_version) = options.text_version {
        notes.push(format!(
          "已生成{grade_prefix}{}版的“密码文本.txt”文件。",
          text_version.name()
        ));
      }
      if options.export_dump {
        notes.push(format!("已生成{grade_prefix}“密码内存转储.dmp”文件。"));
      }
      if let Some(region) = options.cheat_region {
        notes.push(format!(
          "已生成{grade_prefix}的、适用于{}版《黄金太阳 失落的时代》的“密码金手指.txt”文件。",
          region.short_name()
        ));
      }
      if options.export_data_text {
        notes.push(format!("已生成{grade_prefix}“密码数据.txt”文件"));
      }
    }
  }

  notes
}
