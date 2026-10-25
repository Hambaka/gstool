use crate::core::enums::{GameRegion, PasswordGrade, PasswordVersion};
use crate::core::save::pc_name::{EncodedName, MAX_NAME_BYTES, NameError, NameNote};

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

pub fn cheat_code_text(
  password_bytes: &[u8],
  password_grade: PasswordGrade,
  region: GameRegion,
) -> String {
  // The address for password input screen in Golden Sun: The Lost Age
  let mut address = region.tla_pwd_buffer_address();
  let mut text = String::with_capacity(2048);

  let size = password_bytes.len();

  let version_tip = match region {
    GameRegion::Germany => format!(
      "{}级别密码的金手指（适用于{}版《黄金太阳 失落的时代》，亦适用于{}/{}版）：",
      password_grade.short_name(),
      region.short_name(),
      GameRegion::France.short_name(),
      GameRegion::Italy.short_name(),
    ),
    GameRegion::France => format!(
      "{}级别密码的金手指（适用于{}版《黄金太阳 失落的时代》，亦适用于{}/{}版）：",
      password_grade.short_name(),
      region.short_name(),
      GameRegion::Germany.short_name(),
      GameRegion::Italy.short_name(),
    ),
    GameRegion::Italy => format!(
      "{}级别密码的金手指（适用于{}版《黄金太阳 失落的时代》，亦适用于{}/{}版）：",
      password_grade.short_name(),
      region.short_name(),
      GameRegion::Germany.short_name(),
      GameRegion::France.short_name(),
    ),
    _ => format!(
      "{}级别密码的金手指（仅适用于{}版《黄金太阳 失落的时代》）：",
      password_grade.short_name(),
      region.short_name()
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

      if !size.is_multiple_of(2) {
        let _ = writeln!(text, "{:08X}:{:02X}", address + 2, password_bytes[i + 6]);
      }
      break;
    }
  }

  text
}

/// 把一批角色名的编码结果渲染成界面日志文本
///
/// `entries` 的元素为 `(角色中文名, 金手指基址, 编码结果)`；
pub fn name_encode_log(entries: &[(&str, u32, &EncodedName)]) -> String {
  let mut text = String::new();
  let mut first_run = true;

  for &(pc_name, base_address, encoded) in entries {
    if !first_run {
      text.push_str("\n\n");
    }
    first_run = false;

    let _ = writeln!(text, "{pc_name}");
    text.push_str("起名界面的输入方式：\n");
    for note in &encoded.notes {
      let _ = writeln!(text, "{}", name_note_line(note));
    }

    if encoded.is_valid() {
      text.push_str("十六进制值：\n");
      let _ = writeln!(text, "{}", name_hex_text(encoded.all_bytes()));

      text.push_str("金手指：\n");
      text.push_str(&name_cheat_code_text(encoded.all_bytes(), base_address));
    } else {
      let _ = writeln!(
        text,
        "由于存在不可用的字符，故不展示{pc_name}自定义名字所对应的十六进制值和金手指！"
      );
    }
  }

  text
}

/// 单个字符的说明行
fn name_note_line(note: &NameNote) -> String {
  match note {
    NameNote::Direct { input, .. } => format!("{input}：{input}"),
    NameNote::Composed { input, components } => format!("{input}：{components}"),
    NameNote::NotTypable { input, code } => {
      format!("{input}：汉化版字库中包含该字符，但无法在起名界面中正常输入，码值为0x{code:04X}")
    }
    NameNote::Failed { input, reason } => {
      let reason_text = match reason {
        NameError::ControlChar => "汉化版的控制符，不能单独使用！",
        NameError::NotInFont => "汉化版字库中不包含该字符！",
        // 理论上不会出现
        NameError::TooLong => "编码结果超出了名字字段的长度上限！",
      };
      format!("{input}：{reason_text}")
    }
  }
}

/// 十六进制 dump：固定输出全部 10 个字节（含补零）
fn name_hex_text(bytes: &[u8; MAX_NAME_BYTES]) -> String {
  let mut text = String::with_capacity(MAX_NAME_BYTES * 3);
  for (i, name_byte) in bytes.iter().enumerate() {
    if i + 1 == bytes.len() {
      let _ = write!(text, "{name_byte:02X}");
    } else {
      let _ = write!(text, "{name_byte:02X} ");
    }
  }
  text
}

/// 金手指 3 行：4+4+2 字节，地址 = 基址 + i*4
fn name_cheat_code_text(bytes: &[u8; MAX_NAME_BYTES], base_address: u32) -> String {
  let mut text = String::with_capacity(128);

  let _ = writeln!(
    text,
    "{:08X}:{:08X}",
    base_address,
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
  );
  let _ = writeln!(
    text,
    "{:08X}:{:08X}",
    base_address + 4,
    u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]])
  );
  let _ = writeln!(
    text,
    "{:08X}:{:04X}",
    base_address + 8,
    u16::from_le_bytes([bytes[8], bytes[9]])
  );
  text
}
