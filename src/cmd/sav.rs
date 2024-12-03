use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use clap::ArgMatches;
use crate::util::save;
use crate::util::output;
use crate::util::enums::{CheatVersion, PasswordVersion, PasswordGrade};

pub fn run(sub_matches: &ArgMatches) {
  // Read save file.
  let sav_input_path = sub_matches.get_one::<PathBuf>("输入文件路径").unwrap();
  let mut input_file = File::open(sav_input_path).expect("打开存档文件时发生错误！");
  /* Check the size of save file.
     The size of save file should be 64KB,
     though the .SaveRAM file created by Bizhawk is 128KB.
     The size of the .srm file created by the libretro fork of VBA-M, VBA-Next and
     the .4gs file created by 44VBA is 136KB.
     Although the size of those files is over 64KB, seems it only use first 64KB space to store save data? */
  let file_size = input_file.metadata().unwrap().len();
  if file_size != 0x10000 && file_size != 0x20000 && file_size != 0x22000 {
    eprintln!("存档文件的大小应为 64/128/136 KB！");
    return;
  }
  let mut raw_save_file = Vec::new();
  input_file.read_to_end(&mut raw_save_file).unwrap();

  // Check if it is GS1(TBS) save file, also get loop start index.
  let loop_start_index;
  if let Some(index) = save::get_loop_start_index_option(&raw_save_file) {
    loop_start_index = index;
  } else {
    eprintln!("输入的不是《黄金太阳 开启的封印》的存档文件！或者存档文件中没有数据！");
    return;
  };

  /* "all" flag.
     Default value is "false", only export password from clear save data.
     Set it to "true" will export password from all existing save data in save file,
     even it is not a clear data. */
  let to_export_all_data = sub_matches.get_flag("导出所有存档");
  /* Get save data from save file with slot number.
     Also check if the save data is clear data. */
  let save_data_map = save::get_raw_save_data_map(to_export_all_data, &raw_save_file, loop_start_index);
  if save_data_map.is_empty() {
    if to_export_all_data {
      eprintln!("存档文件中没有数据！");
    } else {
      eprintln!("存档文件中没有通关数据！");
    }
    return;
  }

  // Create main output directory.
  let output_dir_str;
  if let Some(output_path_buf) = sub_matches.get_one::<PathBuf>("指定输出文件保存目录") {
    output_dir_str = output::create_output_dir(output_path_buf, true);
  } else {
    output_dir_str = output::create_output_dir(sav_input_path, false);
  }

  // "grade" argument.
  let target_password_grade = *sub_matches.get_one("密码级别").unwrap();
  // "text" argument.
  let password_version = *sub_matches.get_one("密码文本版本").unwrap();
  // "memory" argument.
  let to_export_memory_dump = sub_matches.get_flag("生成内存转储文件");
  // "cheat" argument.
  let mut cheat_version_option: Option<CheatVersion> = None;
  if let Some(cheat_version) = sub_matches.get_one("生成密码金手指") {
    cheat_version_option = Some(*cheat_version);
  }
  // "export" argument.
  let to_export_data_text = sub_matches.get_flag("导出密码数据至文本文件");

  // Write files.
  for (key, val) in &save_data_map {
    let password_bytes = save::get_password_bytes_with_raw_save_bytes(val.get_data(), target_password_grade);
    // Key is save slot number: 0, 1, 2 -> 1, 2, 3
    let sub_dir_str = output::create_sav_sub_dir(*key + 1, val.get_is_clear(), output_dir_str.as_str());

    output::write_password_text_file_with_bytes(&password_bytes, password_version, sub_dir_str.as_str());

    if to_export_memory_dump {
      output::write_memory_dump_file(&password_bytes, sub_dir_str.as_str());
    }

    if let Some(cheat_version) = cheat_version_option {
      output::write_cheat_file(&password_bytes, cheat_version, sub_dir_str.as_str());
    }

    if to_export_data_text {
      if target_password_grade.is_gold() {
        output::write_game_data_text_file(save::get_exported_data_for_dyrati_sheet_with_raw_save_bytes(val.get_data()).as_str(), sub_dir_str.as_str());
      } else {
        output::write_game_data_text_file(save::get_exported_data_for_dyrati_sheet_by_bytes(password_bytes.as_slice(), target_password_grade).as_str(), sub_dir_str.as_str());
      }
    }
  }

  // More detailed output logs, to improve user experience.
  // Sh*t code, but at least it works.
  let mut result_str = String::new();
  result_str.push_str(format!("输出目录：\n{output_dir_str}\n\n").as_str());  
  result_str.push_str("运行概况：\n");
  
  let mut result_index = 1;
  if to_export_all_data {
    result_str.push_str(format!("{result_index}. 已勾选“导出所有存档”，导出包含未通关存档在内的所有存档的数据。\n").as_str());
  } else {
    result_str.push_str(format!("{result_index}. 未勾选“导出所有存档”，仅导出通关存档的数据。\n").as_str());
  };

  result_index += 1;
  let password_grade_cn = match target_password_grade {
    PasswordGrade::Gold => "金",
    PasswordGrade::Silver => "银",
    PasswordGrade::Bronze => "铜",
  };
  let password_version_cn = match password_version {
    PasswordVersion::Japanese => "日文",
    PasswordVersion::English => "英文",
  };
  if to_export_all_data {
    result_str.push_str(format!("{result_index}. 已生成存档记录所对应的{password_grade_cn}级别、{password_version_cn}版的“密码文本.txt”文件。\n").as_str());
  } else {
    result_str.push_str(format!("{result_index}. 已生成通关的存档记录所对应的{password_grade_cn}级别、{password_version_cn}版的“密码文本.txt”文件。\n").as_str());
  }

  if to_export_memory_dump {
    result_index += 1;
    if to_export_all_data {
      result_str.push_str(format!("{result_index}. 已生成存档记录所对应的{password_grade_cn}级别“密码内存转储.dmp”文件。\n").as_str());
    } else {
      result_str.push_str(format!("{result_index}. 已生成通关的存档记录所对应的{password_grade_cn}级别“密码内存转储.dmp”文件。\n").as_str());
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
    if to_export_all_data {
      result_str.push_str(format!("{result_index}. 已生成存档记录所对应的{password_grade_cn}级别的、适用于{cheat_version_cn}版《黄金太阳 失落的时代》的“密码金手指.txt”文件。\n").as_str());
    } else {
      result_str.push_str(format!("{result_index}. 已生成通关的存档记录所对应的{password_grade_cn}级别的、适用于{cheat_version_cn}版《黄金太阳 失落的时代》的“密码金手指.txt”文件。\n").as_str());
    }
  }

  if to_export_data_text {
    result_index += 1;
    if to_export_all_data {
      result_str.push_str(format!("{result_index}. 已生成存档记录所对应的{password_grade_cn}级别“密码数据.txt”文件\n").as_str());
    } else {
      result_str.push_str(format!("{result_index}. 已生成通关的存档记录所对应的{password_grade_cn}级别“密码数据.txt”文件\n").as_str());
    }
  }

  print!("{}", result_str);
}
