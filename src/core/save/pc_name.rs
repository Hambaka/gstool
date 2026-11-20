//! 《开启的封印》新汉化版的角色名字符编码。
//!
//! 在汉化版字库中，字符分为单字节和双字节两种：
//! 1. 单字节：英文字母、数字、半角标点符号、平假名、片假名等……
//! （绝大多数都是可以起名界面直接输入）
//! 2. 双字节：汉字、浊音/半浊音假名，个别全角数字和标点符号等……
//! （绝大多数字符直接对应两个基础单字节字符；少量特殊的无法起名界面输入的汉字则用一个u16码值表示）。
//!
//! 本模块负责玩家输入的文本转换为存档里角色名字段的字节。

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
  codegen_name_chars::CHAR_1_BYTE_TYPABLE_MAP
    .get(&ch)
    .copied()
}

/// 某个字符的编码方式（界面按它逐行生成日志）
#[derive(Debug, Clone)]
pub enum NameNote {
  /// 起名界面可以直接输入的字符
  /// `code`暂时无用，战未来.gif
  Direct { input: char },
  /// 由两个基础字符拼成（浊音假名 / 常用汉字），展开成两个字节
  Composed { input: char, components: String },
  /// 字库里有这个字，但起名界面打不出来（占 1 字节）
  NotTypableByte { input: char, code: u8 },
  /// 字库里有这个字，但起名界面打不出来（占 2 字节，码值大端）
  NotTypableBytes { input: char, code: u16 },
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
}

/// 表示单个的主角团名字字符的编码步骤策略，由 `classify_char` 根据字符归属产出。
#[derive(Debug, Clone)]
enum EncodingStep {
  /// 单字节字符，起名界面可直接输入：CHAR_1_BYTE_TYPABLE_MAP
  Direct(u8),
  /// 单字节字符，起名界面无法直接输入：CHAR_1_BYTE_UNTYPABLE_MAP
  NotTypableByte(u8),
  /// 双字节字符，起名界面可组合输入：CHAR_2_BYTES_TYPABLE_MAP
  Composed(String),
  /// 双字节字符，起名界面无法直接输入（直接返回码值）：CHAR_2_BYTES_UNTYPABLE_MAP
  NotTypableBytes(u16),
  /// 单独输入控制符无效
  ControlChar,
  /// 未找到（不在字库中）
  NotFound,
}

/// 判断一个字符属于哪种编码类别。
fn classify_char(c: char) -> EncodingStep {
  // CHAR_2_BYTES_TYPABLE_MAP（双字节，来自两个基础字符）
  if let Some(ch) = codegen_name_chars::CHAR_2_BYTES_TYPABLE_MAP.get(&c) {
    return EncodingStep::Composed(ch.to_string());
  }

  // CHAR_2_BYTES_UNTYPABLE_MAP（双字节，匹配得到一个u16码值）
  if let Some(ch) = codegen_name_chars::CHAR_2_BYTES_UNTYPABLE_MAP.get(&c) {
    return EncodingStep::NotTypableBytes(*ch);
  }

  // CHAR_1_BYTE_UNTYPABLE_MAP（单字节，不可直接输入）
  if let Some(code) = codegen_name_chars::CHAR_1_BYTE_UNTYPABLE_MAP.get(&c) {
    return EncodingStep::NotTypableByte(*code);
  }

  // 单独的控制符 -> 失败
  if CONTROL_CHARS.contains(&c) {
    return EncodingStep::ControlChar;
  }

  // CHAR_1_BYTE_TYPABLE_MAP（单字节，可输入）
  // 上一步判断已经提前排除控制符，所以不用再加`&& !CONTROL_CHARS.contains(&c)`了
  if let Some(code) = base_character_code(c) {
    return EncodingStep::Direct(code);
  }

  // 其他不在字库中的字符
  EncodingStep::NotFound
}

/// 把玩家输入的角色名编码成游戏内字符字节。
///
/// 返回结构体而不是 `Result`：界面需要**逐个字符**的说明（包括失败的字符），
/// `"名字整体能不能用"`由调用方查看 `is_valid` 方法。
pub fn encode_pc_name(name: &str) -> EncodedName {
  let mut bytes = [0u8; MAX_NAME_BYTES];
  let mut notes: Vec<NameNote> = Vec::new();
  let mut cursor = 0;

  for c in name.chars() {
    match classify_char(c) {
      EncodingStep::Direct(code) => {
        encode_single_byte_char(code, c, &mut bytes, &mut notes, &mut cursor)
      }
      EncodingStep::NotTypableByte(code) => {
        encode_single_byte_not_typable(code, c, &mut bytes, &mut notes, &mut cursor);
      }
      EncodingStep::Composed(components) => {
        encode_composed_char(&components, c, &mut bytes, &mut notes, &mut cursor);
      }
      EncodingStep::NotTypableBytes(code) => {
        encode_not_typable_bytes(code, c, &mut bytes, &mut notes, &mut cursor);
      }
      EncodingStep::ControlChar => {
        notes.push(NameNote::Failed {
          input: c,
          reason: NameError::ControlChar,
        });
      }
      EncodingStep::NotFound => {
        notes.push(NameNote::Failed {
          input: c,
          reason: NameError::NotInFont,
        });
      }
    }
  }

  EncodedName { bytes, notes }
}

/// 编码单字节的可直接输入字符
fn encode_single_byte_char(
  code: u8,
  ch: char,
  bytes: &mut [u8; MAX_NAME_BYTES],
  notes: &mut Vec<NameNote>,
  cursor: &mut usize,
) {
  if *cursor + 1 > MAX_NAME_BYTES {
    notes.push(NameNote::Failed {
      input: ch,
      reason: NameError::TooLong,
    });
    return;
  }

  bytes[*cursor] = code;
  *cursor += 1;
  notes.push(NameNote::Direct { input: ch });
}

/// 编码单字节的不可输入字符
fn encode_single_byte_not_typable(
  code: u8,
  ch: char,
  bytes: &mut [u8; MAX_NAME_BYTES],
  notes: &mut Vec<NameNote>,
  cursor: &mut usize,
) {
  if *cursor + 1 > MAX_NAME_BYTES {
    notes.push(NameNote::Failed {
      input: ch,
      reason: NameError::TooLong,
    });
    return;
  }

  bytes[*cursor] = code;
  *cursor += 1;
  notes.push(NameNote::NotTypableByte { input: ch, code });
}

/// 编码组合双字节字符（来自两个基础字符）
fn encode_composed_char(
  components: &str,
  ch: char,
  bytes: &mut [u8; MAX_NAME_BYTES],
  notes: &mut Vec<NameNote>,
  cursor: &mut usize,
) {
  if *cursor + BYTES_PER_CHAR_MAX > MAX_NAME_BYTES {
    notes.push(NameNote::Failed {
      input: ch,
      reason: NameError::TooLong,
    });
    return;
  }

  for value in components.chars() {
    if let Some(num) = base_character_code(value) {
      bytes[*cursor] = num;
      *cursor += 1;
    } else {
      // 理论上不会发生（编码表已保证组合字符皆由可输入基础字符构成）
      notes.push(NameNote::Failed {
        input: value,
        reason: NameError::NotInFont,
      });
      break; // 部分失败：已写入的字节保留，后续字符继续处理
    }
  }

  notes.push(NameNote::Composed {
    input: ch,
    components: components.to_string(),
  });
}

/// 编码双字节的不可输入字符（u16 大端码值）
fn encode_not_typable_bytes(
  code: u16,
  ch: char,
  bytes: &mut [u8; MAX_NAME_BYTES],
  notes: &mut Vec<NameNote>,
  cursor: &mut usize,
) {
  if *cursor + BYTES_PER_CHAR_MAX > MAX_NAME_BYTES {
    notes.push(NameNote::Failed {
      input: ch,
      reason: NameError::TooLong,
    });
    return;
  }

  let code_bytes = code.to_be_bytes();
  bytes[*cursor] = code_bytes[0];
  bytes[*cursor + 1] = code_bytes[1];
  *cursor += 2;
  notes.push(NameNote::NotTypableBytes { input: ch, code });
}
