use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use clap::ArgMatches;
use crate::util::save;
use crate::util::output;
use crate::util::convert;
use crate::util::enums::{CheatVersion, PasswordGrade, PasswordVersion};

pub fn run(sub_matches: &ArgMatches) {
  // Read password memory dump file.
  let dmp_input_path = sub_matches.get_one::<PathBuf>("输入文件路径").unwrap();
  let mut input_file = File::open(dmp_input_path).expect("打开密码的内存转储二进制文件时发生错误！");
  // Check its file size, and get its grade.
  let file_size = input_file.metadata().unwrap().len();
  let source_password_grade = match file_size {
    260 => PasswordGrade::Gold,
    61 => PasswordGrade::Silver,
    16 => PasswordGrade::Bronze,
    _ => {
      eprintln!("文件大小应为 16 字节，61 字节或 260 字节！");
      return;
    }
  };

  // "grade" argument.
  let mut target_password_grade_option: Option<PasswordGrade> = None;
  if let Some(password_grade) = sub_matches.get_one("密码级别") {
    target_password_grade_option = Some(*password_grade);
  }
  // Check if it is possible to downgrade password
  let mut is_no_need_to_downgrade = true;
  if let Some(target_password_grade) = target_password_grade_option {
    if save::can_downgrade(source_password_grade, target_password_grade) {
      is_no_need_to_downgrade = save::no_need_to_downgrade(source_password_grade, target_password_grade);
    } else {
      eprintln!("无法将密码降级至指定的密码级别！");
      return;
    }
  }

  let mut password_bytes = Vec::new();
  input_file.read_to_end(&mut password_bytes).unwrap();
  // Check if it has contains invalid byte
  let has_invalid_byte = convert::contains_invalid_byte(password_bytes.as_slice());
  if has_invalid_byte {
    eprintln!("密码包含无效字节！");
    return;
  }

  // Check if the password is valid;
  let mut password_bits;
  if let Some(bits) = save::get_valid_password_bits_option(password_bytes.as_slice(), true) {
    password_bits = bits;
  } else {
    eprintln!("密码无效！");
    return;
  }

  // Get flag
  let to_export_data_text = sub_matches.get_flag("导出密码数据至文本文件");
  // "text" argument.
  let mut password_version_option: Option<PasswordVersion> = None;
  if let Some(password_version) = sub_matches.get_one("密码文本版本") {
    password_version_option = Some(*password_version);
  }
  // "cheat" argument.
  let mut cheat_version_option: Option<CheatVersion> = None;
  if let Some(cheat_version) = sub_matches.get_one("生成密码金手指") {
    cheat_version_option = Some(*cheat_version);
  }

  // Generate save data, and set target password bytes
  let save_data = save::get_save_data_with_password_bits(&mut password_bits, source_password_grade);
  let target_password_bytes;
  if let Some(target_password_grade) = target_password_grade_option {
    if is_no_need_to_downgrade {
      if password_version_option.is_none() && cheat_version_option.is_none() && !to_export_data_text {
        eprintln!("密码无需降级，也没有生成任何文件……");
        return;
      }
      target_password_bytes = password_bytes;
    } else {
      target_password_bytes = save::get_password_bytes_with_save_data(&save_data, target_password_grade);
    }
  } else {
    target_password_bytes = password_bytes;
  }

  // Create output directory.
  let output_dir_str;
  if let Some(output_path_buf) = sub_matches.get_one::<PathBuf>("指定输出文件保存目录") {
    output_dir_str = output::create_output_dir(output_path_buf, true);
  } else {
    output_dir_str = output::create_output_dir(dmp_input_path, false);
  }

  // Write files.
  if target_password_grade_option.is_some() && !is_no_need_to_downgrade {
    output::write_memory_dump_file(&target_password_bytes, output_dir_str.as_str());
  }

  if let Some(password_version) = password_version_option {
    output::write_password_text_file_with_bytes(&target_password_bytes, password_version, output_dir_str.as_str());
  }

  if let Some(cheat_version) = cheat_version_option {
    output::write_cheat_file(&target_password_bytes, cheat_version, output_dir_str.as_str());
  }

  if to_export_data_text {
    if let Some(target_password_grade) = target_password_grade_option {
      if is_no_need_to_downgrade {
        output::write_game_data_text_file(save::get_exported_data_for_dyrati_sheet_by_save_data(&save_data).as_str(), output_dir_str.as_str());
      } else {
        output::write_game_data_text_file(save::get_exported_data_for_dyrati_sheet_by_bytes(target_password_bytes.as_slice(), target_password_grade).as_str(), output_dir_str.as_str());
      }
    } else {
      output::write_game_data_text_file(save::get_exported_data_for_dyrati_sheet_by_save_data(&save_data).as_str(), output_dir_str.as_str());
    }
  }

  // More detailed output logs, to improve user experience.
  // Sh*t code, but at least it works.
  let mut result_str = String::new();
  result_str.push_str(format!("输出目录：\n{output_dir_str}\n\n").as_str());  
  result_str.push_str("运行概况：\n");

  let mut result_index = 1;
  let source_password_grade_cn = match source_password_grade {
    PasswordGrade::Gold => "金",
    PasswordGrade::Silver => "银",
    PasswordGrade::Bronze => "铜",
  };
  result_str.push_str(format!("{result_index}. 输入的源文件为{source_password_grade_cn}级别的密码内存转储文件。\n").as_str());

  let mut downgrade_str_option = None;
  if let Some(target_password_grade) = target_password_grade_option {
    if !is_no_need_to_downgrade {
      result_index += 1;
      let target_password_grade_cn = match target_password_grade {
        PasswordGrade::Gold => "金",
        PasswordGrade::Silver => "银",
        PasswordGrade::Bronze => "铜",
      };
      downgrade_str_option = Some(target_password_grade_cn); 
      result_str.push_str(format!("{result_index}. 已生成降级为{target_password_grade_cn}级别的“密码内存转储.dmp”文件。\n").as_str());
    }
  }

  if let Some(password_version) = password_version_option {
    result_index += 1;
    let password_version_cn = match password_version {
      PasswordVersion::Japanese => "日文",
      PasswordVersion::English => "英文",
    };
    if let Some(target_password_grade_cn) = downgrade_str_option {
      result_str.push_str(format!("{result_index}. 已生成降级为{target_password_grade_cn}级别的{password_version_cn}版“密码文本.txt”文件。\n").as_str());
    } else {
      result_str.push_str(format!("{result_index}. 已生成与源文件同级的{source_password_grade_cn}级别的{password_version_cn}版“密码文本.txt”文件。\n").as_str());
    }
  }

  if let Some(cheat_version) = cheat_version_option {
    result_index += 1;
    let cheat_version_cn = match cheat_version {
        CheatVersion::Japan => "日",
        CheatVersion::USA => "美",
        CheatVersion::Germany => "德",
        CheatVersion::Spain => "西",
        CheatVersion::France => "法",
        CheatVersion::Italy => "意",
    };
    if let Some(target_password_grade_cn) = downgrade_str_option {
      result_str.push_str(format!("{result_index}. 已生成降级为{target_password_grade_cn}级别的、适用于{cheat_version_cn}版《黄金太阳 失落的时代》的“密码金手指.txt”文件。\n").as_str());
    } else {
      result_str.push_str(format!("{result_index}. 已生成与源文件同级的{source_password_grade_cn}级别的、适用于{cheat_version_cn}版《黄金太阳 失落的时代》的“密码金手指.txt”文件。\n").as_str());
    }
  }

  if to_export_data_text{
    result_index += 1;
    if let Some(target_password_grade_cn) = downgrade_str_option {
      result_str.push_str(format!("{result_index}. 已生成降级为{target_password_grade_cn}级别的“密码数据.txt”文件\n").as_str());
    } else {
      result_str.push_str(format!("{result_index}. 已生成与源文件同级的{source_password_grade_cn}级别的“密码数据.txt”文件\n").as_str());
    }
  }

  print!("{}", result_str);
}
