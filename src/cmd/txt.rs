use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use clap::ArgMatches;
use crate::util::save;
use crate::util::output;
use crate::util::convert;
use crate::util::enums::{CheatVersion, PasswordGrade, PasswordVersion};

pub fn run(sub_matches: &ArgMatches) {
  // Read password text file.
  let txt_input_path = sub_matches.get_one::<PathBuf>("输入文件路径").unwrap();
  let mut input_file = File::open(txt_input_path).expect("打开密码文本文件时发生错误！");
  let mut password = String::new();
  input_file.read_to_string(&mut password).expect("输入的不是有效的文本文件！");
  // If the text file is empty, exit.
  if password.is_empty() {
    eprintln!("文本文件内没有任何内容！");
    return;
  }
  // Remove all line break and whitespace.
  let password_without_whitespace = convert::remove_whitespace(password.as_ref());
  // Check password's length, and get its grade.
  let chars_count = password_without_whitespace.chars().count();
  let source_password_grade = match chars_count {
    260 => PasswordGrade::Gold,
    61 => PasswordGrade::Silver,
    16 => PasswordGrade::Bronze,
    _ => {
      eprintln!("密码长度不正确！");
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

  // Check its password version.
  let password_version = convert::get_password_version(password_without_whitespace.as_ref());
  // Check if it contains invalid char
  let has_invalid_char = match password_version {
    PasswordVersion::Japanese => convert::contains_invalid_char_jp(password_without_whitespace.as_str()),
    PasswordVersion::English => convert::contains_invalid_char_en(password_without_whitespace.as_str()),
  };
  if has_invalid_char {
    eprintln!("密码文本包含无效字符！");
    return;
  }

  // Check if the password is valid.
  let password_bytes = convert::txt_to_dmp(password_without_whitespace.as_str(), password_version);
  let mut password_bits;
  if let Some(bits) = save::get_valid_password_bits_option(password_bytes.as_slice(), true) {
    password_bits = bits;
  } else {
    eprintln!("密码无效！");
    return;
  }

  // Get all flags
  let to_convert_password = sub_matches.get_flag("转换密码版本");
  let to_export_memory_dump = sub_matches.get_flag("生成内存转储文件");
  let to_export_data_text = sub_matches.get_flag("导出密码数据至文本文件");
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
      if !to_convert_password && !to_export_memory_dump && cheat_version_option.is_none() && !to_export_data_text {
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
    output_dir_str = output::create_output_dir(txt_input_path, false);
  }

  // Write files.
  if to_convert_password {
    if target_password_grade_option.is_none() {
      output::write_converted_password_text_file(convert::txt_to_another_version(&password_without_whitespace, password_version).as_str(), password_version.rev(), output_dir_str.as_str());
    } else {
      output::write_password_text_file_with_bytes(&target_password_bytes, password_version.rev(), output_dir_str.as_str());
    }
  } else if target_password_grade_option.is_some() && !is_no_need_to_downgrade {
    output::write_password_text_file_with_bytes(&target_password_bytes, password_version, output_dir_str.as_str());
  }

  if to_export_memory_dump {
    output::write_memory_dump_file(&target_password_bytes, output_dir_str.as_str());
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
  let source_password_version_cn = match password_version {
    PasswordVersion::Japanese => "日文",
    PasswordVersion::English => "英文",
  };
  result_str.push_str(format!("{result_index}. 输入的源文件为{source_password_grade_cn}级别的{source_password_version_cn}版密码文本文件。\n").as_str());

  let convert_flag_str_option = if to_convert_password {
    match password_version.rev() {
      PasswordVersion::Japanese => Some("日文"),
      PasswordVersion::English => Some("英文"),
    }
  } else {
    None
  };

  let mut downgrade_str_option = None;
  if let Some(target_password_grade) = target_password_grade_option {
    if !is_no_need_to_downgrade {
      let target_password_grade_cn = match target_password_grade {
        PasswordGrade::Gold => "金",
        PasswordGrade::Silver => "银",
        PasswordGrade::Bronze => "铜",
      };
      downgrade_str_option = Some(target_password_grade_cn); 
    }
  }

  match convert_flag_str_option {
    Some(convert_flag_str) => {
      result_index += 1;
      match downgrade_str_option {
        Some(downgrade_str) => {
          result_str.push_str(format!("{result_index}. 已生成降级为{downgrade_str}级别的、转换为{convert_flag_str}版的“密码文本.txt”文件。\n").as_str());
        },
        None => {
          result_str.push_str(format!("{result_index}. 已生成与源文件同级的{source_password_grade_cn}级别的、转换为{convert_flag_str}版的“密码文本.txt”文件。\n").as_str());
        },
      }
    },
    None => {
      result_index += 1;
      match downgrade_str_option {
        Some(downgrade_str) => {
          result_str.push_str(format!("{result_index}. 已生成降级为{downgrade_str}级别的、未经版本转换的{source_password_version_cn}版“密码文本.txt”文件。\n").as_str());
        },
        None => {
          result_str.push_str(format!("{result_index}. 已生成与源文件同级的{source_password_grade_cn}级别的、未经版本转换的{source_password_version_cn}版“密码文本.txt”文件。\n").as_str());
        },
      }
    },
  }

  if to_export_memory_dump {
    result_index += 1;
    if let Some(target_password_grade_cn) = downgrade_str_option {
      result_str.push_str(format!("{result_index}. 已生成降级为{target_password_grade_cn}级别的“密码内存转储.dmp”文件。\n").as_str());
    } else {
      result_str.push_str(format!("{result_index}. 已生成与源文件同级的{source_password_grade_cn}级别的“密码内存转储.dmp”文件。\n").as_str());
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
      result_str.push_str(format!("{result_index}. 已生成降级为{target_password_grade_cn}级别“密码数据.txt”文件\n").as_str());
    } else {
      result_str.push_str(format!("{result_index}. 已生成与源文件同级的{source_password_grade_cn}级别“密码数据.txt”文件\n").as_str());
    }
  }

  print!("{}", result_str);
}
