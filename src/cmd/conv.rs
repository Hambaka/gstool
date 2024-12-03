use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use clap::ArgMatches;
use crate::util;
use crate::util::enums::{BuildDateType, NameType, GameType};

pub fn run(matches: &ArgMatches) {
  let mut pc_name_type_option: Option<NameType> = None;
  if let Some(name_type) = matches.get_one("名字版本") {
    pc_name_type_option = Some(*name_type);
  }

  let mut build_date_type_option: Option<BuildDateType> = None;
  if let Some(build_date_type) = matches.get_one("构建日期版本") {
    build_date_type_option = Some(*build_date_type);
  }

  // Read save file.
  let raw_input_path = matches.get_one::<PathBuf>("输入文件路径").unwrap();
  let mut input_file = File::open(raw_input_path).expect("打开存档文件时发生错误！");

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

  // Get raw save.
  let mut raw_save_file = Vec::new();
  input_file.read_to_end(&mut raw_save_file).unwrap();

  // Detect game/save type, also get loop start index.
  let (game_type_option, loop_start_index_option) = util::save::get_game_type_with_loop_start_index_option(&raw_save_file);
  if game_type_option.is_none() {
    eprintln!("输入的不是《黄金太阳 开启的封印》或《黄金太阳 失落的时代》的存档文件！或者存档文件中没有数据！");
    return;
  }
  let loop_start_index = loop_start_index_option.unwrap();

  // Simple validation for name type and game type combination.
  if let Some(name_type) = pc_name_type_option {
    let game_type = game_type_option.unwrap();
    if matches!(name_type, NameType::ChineseFanTranslationMobileTeam) && matches!(game_type, GameType::TheBrokenSeal) {
      eprintln!("输入的是《黄金太阳 开启的封印》存档文件，不适用于2代汉化的主角团中文名！");
      return;
    } else if matches!(name_type, NameType::ChineseFanTranslation2023Team) && matches!(game_type, GameType::TheLostAge) {
      eprintln!("输入的是《黄金太阳 失落的时代》存档文件，不适用于1代新汉化的主角团中文名！");
      return;
    }
  }

  // Convert save data.
  util::save::convert_save_file(&mut raw_save_file, game_type_option, loop_start_index, pc_name_type_option, build_date_type_option);
  // Start to create and write output save file.
  let output_path;
  let mut output_file;
  if let Some(raw_output) = matches.get_one::<PathBuf>("指定输出文件保存路径") {
    output_path = PathBuf::from(raw_output);
    let output_dir = output_path.parent().unwrap();
    fs::create_dir_all(output_dir).expect("创建目录失败！");
  } else {
    let path = Path::new(raw_input_path);
    let parent = path.parent().unwrap().to_str().unwrap();
    let file_stem = path.file_stem().unwrap().to_str().unwrap();

    let file_extension = if path.extension().is_none() {
      ""
    } else {
      path.extension().unwrap().to_str().unwrap()
    };

    let mut file_name_str = String::new();
    file_name_str.push_str(file_stem);
    file_name_str.push_str("_转换");

    if !file_extension.is_empty() {
      file_name_str.push('.');
      file_name_str.push_str(file_extension);
    }

    output_path = PathBuf::from(parent).join(file_name_str);
  }
  output_file = File::create(output_path.clone()).unwrap_or_else(|_| panic!("创建 “{}” 失败！", output_path.to_str().unwrap()));
  output_file.write_all(&raw_save_file).unwrap_or_else(|_| panic!("创建 “{}” 失败！", output_path.to_str().unwrap()));

  // More detailed output logs, to improve user experience.
  // Sh*t code, but at least it works.
  let mut result_str = String::new();
  result_str.push_str(format!("输出文件：\n{}\n\n", output_path.to_str().unwrap()).as_str());  
  result_str.push_str("运行概况：\n");

  let mut result_index = 0;
  if let Some(game_type) = game_type_option {
    result_index += 1;
    let game_type_cn = match game_type {
        GameType::TheBrokenSeal => "《黄金太阳 开启的封印》",
        GameType::TheLostAge => "《黄金太阳 失落的时代》",
    };
    result_str.push_str(format!("{result_index}. 输入的源文件为{game_type_cn}的存档文件。\n").as_str());
  }

  if let Some(name_type) = pc_name_type_option {
    result_index += 1;
    let name_type_cn = match name_type {
        NameType::Japanese => "日文",
        NameType::English => "英文",
        NameType::German => "德文",
        NameType::Spanish => "西班牙文",
        NameType::French => "法文",
        NameType::Italian => "意大利文",
        NameType::ChineseFanTranslationMobileTeam => "2代汉化",
        NameType::ChineseFanTranslation2023Team => "1代新汉化",
    };
    result_str.push_str(format!("{result_index}. 生成的存档文件已将主角团成员的名字转换为{name_type_cn}版的默认名。\n").as_str());
  }

  if let Some(build_date_type) = build_date_type_option {
    result_index += 1;
    let build_date_type_cn = match build_date_type {
        BuildDateType::Japan => "日",
        BuildDateType::USA => "美",
        BuildDateType::Germany => "德",
        BuildDateType::Spain => "西",
        BuildDateType::France => "法",
        BuildDateType::Italy => "意",
    };
    result_str.push_str(format!("{result_index}. 生成的存档文件已将构建日期转换为{build_date_type_cn}版。\n").as_str());
  }

  print!("{}", result_str);
}
