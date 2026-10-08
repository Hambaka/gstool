use crate::core::enums::{GameVersion, PasswordGrade, PasswordVersion};

use std::fmt::Write as _;

pub fn password_text_beautify(converted: &str, version: PasswordVersion) -> String {
  let mut text = String::with_capacity(1024);
  let whitespace = match version {
    PasswordVersion::Japanese => '　',
    PasswordVersion::English => ' ',
  };

  for (i, char) in converted.chars().enumerate() {
    text.push(char);
    if (i + 1) % 50 == 0 {
      text.push_str("\n\n");
    } else if (i + 1) % 10 == 0 {
      text.push('\n');
    } else if (i + 1) % 5 == 0 {
      text.push(whitespace);
    }
  }

  text
}

pub fn cheat_code_text(password_bytes: &[u8], version: GameVersion) -> String {
  // The address for password input screen in Golden Sun: The Lost Age
  let mut address = version.get_tla_pwd_buffer_address();
  let mut text = String::with_capacity(2048);

  let size = password_bytes.len();
  let password_grade = PasswordGrade::from_size(size).unwrap();

  let version_tip = match version {
    GameVersion::Germany => format!(
      "{}级别密码的金手指（适用于德版《黄金太阳 失落的时代》，亦适用于法/意版）：",
      password_grade.name()
    ),
    GameVersion::France => format!(
      "{}级别密码的金手指（适用于法版《黄金太阳 失落的时代》，亦适用于德/意版）：",
      password_grade.name()
    ),
    GameVersion::Italy => format!(
      "{}级别密码的金手指（适用于意版《黄金太阳 失落的时代》，亦适用于德/法版）：",
      password_grade.name()
    ),
    _ => format!(
      "{}级别密码的金手指（仅适用于{}版《黄金太阳 失落的时代》）：",
      password_grade.name(),
      version.name()
    ),
  };

  let _ = writeln!(
    text,
    r"注意事项：
1. 密码金手指请最好只在输入密码时开启，如果一直启用密码金手指会产生意料之外的问题，例如进入游戏后对话卡死无法继续游戏，所以个人建议最好在输入密码界面点击“完成”之前把密码金手指关掉。（换句话说，在密码输入界面开启密码金手指后，再立刻关闭密码金手指，就一定不会有任何问题）
2. 如果想要了解具体的使用方法，请见《黄金太阳 开启的封印》新汉化发布时附带的文档“01 - 通关数据继承说明.html”中的“1.2.4 输入密码金手指”章节。

{version_tip}"
  );

  let max_loop = (size - 2) / 4;
  let mut loop_count = 0;

  let _ = writeln!(
    text,
    "{:08X}:{:04X}",
    address,
    u16::from_le_bytes([password_bytes[0], password_bytes[1]])
  );
  address += 2;

  for i in (2..size).step_by(4) {
    let _ = writeln!(
      text,
      "{:08X}:{:08X}",
      address,
      u32::from_le_bytes([
        password_bytes[i],
        password_bytes[i + 1],
        password_bytes[i + 2],
        password_bytes[i + 3]
      ])
    );

    address += 4;
    loop_count += 1;

    if loop_count == max_loop {
      let _ = writeln!(
        text,
        "{:08X}:{:04X}",
        address,
        u16::from_le_bytes([password_bytes[i + 4], password_bytes[i + 5]])
      );

      if size % 2 != 0 {
        let _ = writeln!(text, "{:08X}:{:02X}", address + 2, password_bytes[i + 6]);
      }
      break;
    }
  }

  text
}
