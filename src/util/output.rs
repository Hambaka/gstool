use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use crate::util::enums;
use crate::util::convert::dmp_to_txt;
use crate::util::enums::PasswordVersion;

pub fn create_sav_sub_dir(slot_num: u8, is_clear_data: bool, output_dir_str: &str) -> String {
  let output_path = if is_clear_data {
    Path::new(output_dir_str).join(format!("存档{slot_num:02}（通关）"))
  } else {
    Path::new(output_dir_str).join(format!("存档{slot_num:02}"))
  };
  let sub_dir_str = String::from(output_path.to_str().unwrap());
  fs::create_dir_all(output_path).expect("创建子文件夹失败！");
  sub_dir_str
}

pub fn create_output_dir(output_path: &Path, has_output_arg: bool) -> String {
  let output_dir_str = if has_output_arg {
    String::from(output_path.to_str().unwrap())
  } else {
    String::from(output_path.parent().unwrap().join(format!("{}_输出", output_path.file_stem().unwrap().to_str().unwrap())).to_str().unwrap())
  };
  let output_dir_path = Path::new(output_dir_str.as_str());
  fs::create_dir_all(output_dir_path).expect("创建输出文件夹失败！");

  output_dir_str
}

pub fn write_password_text_file_with_bytes(password_bytes: &[u8], password_version: PasswordVersion, output_dir_str: &str) {
  write_converted_password_text_file(dmp_to_txt(password_bytes, password_version).as_str(), password_version, output_dir_str);
}

pub fn write_converted_password_text_file(converted_password_text: &str, password_version: PasswordVersion, output_dir_str: &str) {
  let mut password_text = String::new();
  let whitespace = match password_version {
    PasswordVersion::Japanese => '　',
    PasswordVersion::English => ' ',
  };

  for (i, char) in converted_password_text.chars().enumerate() {
    password_text.push(char);
    if (i + 1) % 50 == 0 {
      password_text.push_str("\n\n");
    } else if (i + 1) % 10 == 0 {
      password_text.push('\n');
    } else if (i + 1) % 5 == 0 {
      password_text.push(whitespace);
    }
  }

  let output_path = Path::new(output_dir_str).join("密码文本.txt");
  let mut output_file = File::create(output_path).expect("创建密码文本文件失败！");
  output_file.write_all(password_text.as_bytes()).expect("写入密码文本文件失败！");
}

/// Write password bytes to a binary file.
/// After you go to password input screen in GS2, you can import it via emulator's memory viewer, so that means you don't have to input password manually.
/// Though you have to choose the correct address and import it, you can check the address in `get_cheat_address` function of "enums.rs".
pub fn write_memory_dump_file(password_bytes: &[u8], sub_dir_str: &str) {
  let output_path = Path::new(sub_dir_str).join("密码内存转储.dmp");
  let mut output_file = File::create(output_path).expect("创建密码内存转储二进制文件失败！");
  output_file.write_all(password_bytes).expect("写入密码内存转储二进制文件失败！");
}

/// Well, cheat file is much easier to use,
/// After you go to password input screen in GS2, you can copy and paste cheat codes in emulators.
/// But you have to remove the cheat codes you just entered immediately after applying it, otherwise it may cause problems.
/// Though I'm not sure, maybe you can use this kind of raw cheat code on your phone as well?
pub fn write_cheat_file(password_bytes: &[u8], cheat_version: enums::CheatVersion, sub_dir_str: &str) {
  // The address for password input screen in Golden Sun: The Lost Age
  let mut address = cheat_version.get_address();
  let mut text = String::new();
  text.push_str("注意事项：\n");
  text.push_str("1. 密码金手指请最好只在输入密码时开启，如果一直启用密码金手指会产生意料之外的问题，例如进入游戏后对话卡死无法继续游戏，");
  text.push_str("所以个人建议最好在输入密码界面点击“完成”之前把密码金手指关掉。");
  text.push_str("（换句话说，在密码输入界面开启密码金手指后，再立刻关闭密码金手指，就一定不会有任何问题）\n");
  text.push_str("2. 如果想要了解具体的使用方法，请见《黄金太阳 开启的封印》新汉化发布时附带的文档“01 - 通关数据继承说明.html”中的“1.2.4 输入密码金手指”章节。\n\n");
  text.push_str("金手指：\n");
  let size = password_bytes.len();
  let loop_num = (size - 2) / 4;
  let mut loop_count = 0;
  text.push_str(format!("{:08X}:{:04X}\n", address, u16::from_le_bytes([password_bytes[0], password_bytes[1]])).as_str());
  address += 2;
  for i in (2..size).step_by(4) {
    text.push_str(format!("{:08X}:{:08X}\n", address, u32::from_le_bytes([password_bytes[i], password_bytes[i + 1], password_bytes[i + 2], password_bytes[i + 3]])).as_str());
    address += 4;
    loop_count += 1;
    if loop_count == loop_num {
      if size % 2 == 0 {
        text.push_str(format!("{:08X}:{:04X}\n", address, u16::from_le_bytes([password_bytes[i + 4], password_bytes[i + 5]])).as_str());
      } else {
        text.push_str(format!("{:08X}:{:04X}\n", address, u16::from_le_bytes([password_bytes[i + 4], password_bytes[i + 5]])).as_str());
        text.push_str(format!("{:08X}:{:02X}\n", address + 2, password_bytes[i + 6]).as_str());
      }
      break;
    }
  }
  let output_path = Path::new(sub_dir_str).join("密码金手指.txt");
  let mut output_file = File::create(output_path).expect("创建密码金手指文本文件失败！");
  output_file.write_all(text.as_bytes()).expect("写入密码金手指文本文件失败！");
}

pub fn write_game_data_text_file(text_data: &str, output_dir_str: &str) {
  let output_path = Path::new(output_dir_str).join("密码数据.txt");
  let mut output_file = File::create(output_path).expect("创建密码数据文本文件失败！");
  output_file.write_all(text_data.as_bytes()).expect("写入密码数据文本文件失败！");
}
