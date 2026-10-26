//! 《开启的封印》新汉化版的角色名字符编码。
//!
//! 汉化字库把汉字拆成两个基础字符的码值组合，所以一个汉字要占两个字节
//! （少量常用汉字直接对应两个基础字符；少量特殊的字则用一个u16码值表示）。
//! 本模块负责玩家输入的文本->存档里角色名字段的字节。

// 4 张由构建脚本生成的查找表（见 "gen_name" in "build.rs"）
mod codegen_name_chars {
  #![allow(clippy::all, clippy::pedantic, clippy::nursery)]
  include!(concat!(env!("OUT_DIR"), "/codegen_name_chars.rs"));
}

/// 界面输入框允许的字符数上限
pub const MAX_NAME_CHARS: usize = 5;
/// 单个字符最多编码成的字节数
pub const BYTES_PER_CHAR_MAX: usize = 2;
/// 角色名字段的字节数上限
pub const MAX_NAME_BYTES: usize = 10;

/// 汉化字库用作控制符，**不能单独输入**。
/// （作为独立存在的字符在汉化版是不合法的，但是部分汉字的编码的后半由控制符组成，只有作为后半的组合字符时才能正常输入）
pub const CONTROL_CHARS: [char; 13] = [
  '#', '$', '<', '=', '>', '@', '[', '￥', ']', '^', '{', '|', '}',
];

/// 起名界面里这个字符对应的键位图编号。返回 `None` 表示界面上打不出这个字符。
pub fn base_character_code(ch: char) -> Option<u8> {
  codegen_name_chars::BASE_CHARACTERS_MAP.get(&ch).copied()
}

/// 某个字符的编码方式（界面按它逐行生成日志）
#[derive(Debug, Clone)]
pub enum NameNote {
  /// 起名界面可以直接输入的字符
  /// `code`暂时无用，战未来.gif
  Direct { input: char, code: u8 },
  /// 由两个基础字符拼成（浊音假名 / 常用汉字），展开成两个字节
  Composed { input: char, components: String },
  /// 字库里有这个字，但起名界面打不出来（占 2 字节，码值大端）
  NotTypable { input: char, code: u16 },
  /// 这个字符没能进入编码结果
  Failed { input: char, reason: NameError },
}

#[derive(Debug, Clone)]
pub enum NameError {
  /// 汉化版把它用作汉字偏旁码 / 控制符，不能单独使用
  ControlChar,
  /// 字库中不包含该字符
  NotInFont,
  /// 编码后超出了名字字段的长度上限
  TooLong,
}

/// 一个角色名的编码结果
#[derive(Debug, Clone)]
pub struct EncodedName {
  /// 编码后的字节（定长，未使用部分为 0）
  bytes: [u8; MAX_NAME_BYTES],
  /// 编码结束时的游标位置（= 有效字节数）
  /// `byte_len`暂时无用，战未来.gif
  byte_len: usize,
  /// 逐个字符的编码说明，**包含失败的字符**（界面要展示它们）
  pub notes: Vec<NameNote>,
}

impl EncodedName {
  /// 是否每个字符都编码成功。
  pub fn is_valid(&self) -> bool {
    !self
      .notes
      .iter()
      .any(|note| matches!(note, NameNote::Failed { .. }))
  }
  /// 完整的定长字节（十六进制 dump 与金手指都按 10 字节输出）
  pub fn all_bytes(&self) -> &[u8; MAX_NAME_BYTES] {
    &self.bytes
  }
  /// 有效字节数（= 编码结束时的游标位置）
  /// `byte_len()`暂时无用，战未来
  pub fn byte_len(&self) -> usize {
    self.byte_len
  }
}

/// 把玩家输入的角色名编码成游戏内字符字节。。
///
/// 返回结构体而不是 `Result`：界面需要**逐个字符**的说明（包括失败的字符），
/// `“名字整体能不能用”由调用方看查看is_valid方法`。
pub fn encode_pc_name(name: &str) -> EncodedName {
  let mut bytes = [0u8; MAX_NAME_BYTES];
  let mut notes: Vec<NameNote> = Vec::new();
  let mut cursor = 0usize;

  for c in name.chars() {
    // 1. 在 BASE_CHARACTERS_MAP 里且不在 CONTROL_CHARS 里 -> 1 字节，Direct
    //    注意：两个条件是合在一起的，控制符字符会落到第 5 条
    if base_character_code(c).is_some() && !CONTROL_CHARS.contains(&c) {
      if cursor + 1 > MAX_NAME_BYTES {
        // 理论上不会发生
        notes.push(NameNote::Failed {
          input: c,
          reason: NameError::TooLong,
        });
        break;
      }
      let code = base_character_code(c).unwrap();
      bytes[cursor] = code;
      cursor += 1;
      notes.push(NameNote::Direct { input: c, code });
    }
    // 2. 在 DAKUTEN_KANA_MAP 里 -> 展开成 2 个基础字符 -> 2 字节，Composed
    else if codegen_name_chars::DAKUTEN_KANA_MAP.contains_key(&c) {
      let components = codegen_name_chars::DAKUTEN_KANA_MAP
        .get(&c)
        .unwrap()
        .to_string();
      if cursor + BYTES_PER_CHAR_MAX > MAX_NAME_BYTES {
        notes.push(NameNote::Failed {
          input: c,
          reason: NameError::TooLong,
        });
        break;
      }
      for value in components.chars() {
        if let Some(num) = base_character_code(value) {
          bytes[cursor] = num;
          cursor += 1;
        } else {
          // 理论上不会发生
          notes.push(NameNote::Failed {
            input: value,
            reason: NameError::NotInFont,
          });
        }
      }
      notes.push(NameNote::Composed {
        input: c,
        components,
      });
    }
    // 3. 在 COMMON_HANZI_MAP 里 -> 展开成 2 个基础字符 -> 2 字节，Composed
    else if codegen_name_chars::COMMON_HANZI_MAP.contains_key(&c) {
      let components = codegen_name_chars::COMMON_HANZI_MAP
        .get(&c)
        .unwrap()
        .to_string();
      if cursor + BYTES_PER_CHAR_MAX > MAX_NAME_BYTES {
        // 理论上不会发生
        notes.push(NameNote::Failed {
          input: c,
          reason: NameError::TooLong,
        });
        break;
      }
      for value in components.chars() {
        if let Some(num) = base_character_code(value) {
          bytes[cursor] = num;
          cursor += 1;
        } else {
          // 理论上不会发生
          notes.push(NameNote::Failed {
            input: value,
            reason: NameError::NotInFont,
          });
        }
      }
      notes.push(NameNote::Composed {
        input: c,
        components,
      });
    }
    // 4. 在 SPECIAL_HANZI_MAP 里 -> u16 大端 -> 2 字节，NotTypable
    else if codegen_name_chars::SPECIAL_HANZI_MAP.contains_key(&c) {
      let code = *codegen_name_chars::SPECIAL_HANZI_MAP.get(&c).unwrap();
      if cursor + BYTES_PER_CHAR_MAX > MAX_NAME_BYTES {
        // 理论上不会发生
        notes.push(NameNote::Failed {
          input: c,
          reason: NameError::TooLong,
        });
        break;
      }
      let code_bytes = code.to_be_bytes();
      bytes[cursor] = code_bytes[0];
      bytes[cursor + 1] = code_bytes[1];
      cursor += 2;
      notes.push(NameNote::NotTypable { input: c, code });
    }
    // 5. 在 CONTROL_CHARS 里 -> Failed(ControlChar)
    else if CONTROL_CHARS.contains(&c) {
      notes.push(NameNote::Failed {
        input: c,
        reason: NameError::ControlChar,
      });
    }
    // 6. 其它 -> Failed(NotInFont)
    else {
      notes.push(NameNote::Failed {
        input: c,
        reason: NameError::NotInFont,
      });
    }
  }

  EncodedName {
    bytes,
    byte_len: cursor,
    notes,
  }
}
