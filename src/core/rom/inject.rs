use std::fmt;

use crate::core::{
  enums::{GameRegion, PasswordGrade, PasswordVersion},
  rom::format::{
    INJECT_ALIGNMENT, INJECT_BLOCK_HEADER_SIZE, INJECT_BLOCK_MAX_SIZE, INJECT_HEADER, MAX_ROM_SIZE,
    ROM_BUILD_DATE_LEN, ROM_BUILD_DATE_OFFSET, ROM_MEMORY_MAPPING, SOFTWARE_TITLE_LEN,
    SOFTWARE_TITLE_OFFSET, SOFTWARE_TITLE_TLA_JAPAN, SOFTWARE_TITLE_TLA_WORLD,
    TARGET_ADDRESS_OFFSET,
  },
};

include!(concat!(env!("OUT_DIR"), "/codegen_bin.rs"));

#[derive(Debug)]
pub enum RomInjectError {
  UnknownRegion,
  RegionUnmatched,
  NotTlaRom,
  UnableToRead,
  NotEnoughSpace,
  InjectionUnknown,
  InjectionPosNotFound,
}

impl fmt::Display for RomInjectError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::UnknownRegion => write!(f, "未知的《黄金太阳 失落的时代》的游戏版本！"),
      Self::RegionUnmatched => write!(f, "ROM内记录的语言版本信息不匹配！"),
      Self::NotTlaRom => write!(f, "输入的不是《黄金太阳 失落的时代》的ROM文件！"),
      Self::UnableToRead => write!(f, "无法读取ROM中的数据！"),
      Self::NotEnoughSpace => write!(f, "没有足够的空间用于注入自动输入密码的程序！"),
      Self::InjectionUnknown => write!(f, "无法确定ROM是否已经注入了自动输入密码的程序！"),
      Self::InjectionPosNotFound => write!(f, "未找到该ROM的自动输入密码的程序的注入位置"),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PasswordCombination {
  GoldSilverBronze = 0,
  GoldSilver = 1,
  GoldBronze = 2,
  SilverBronze = 3,
  Gold = 4,
  Silver = 5,
  Bronze = 6,
}

impl PasswordCombination {
  fn index(self) -> usize {
    self as usize
  }

  fn from(value: usize) -> Option<Self> {
    match value {
      0 => Some(PasswordCombination::GoldSilverBronze),
      1 => Some(PasswordCombination::GoldSilver),
      2 => Some(PasswordCombination::GoldBronze),
      3 => Some(PasswordCombination::SilverBronze),
      4 => Some(PasswordCombination::Gold),
      5 => Some(PasswordCombination::Silver),
      6 => Some(PasswordCombination::Bronze),
      _ => None,
    }
  }

  pub fn from_sources(sources: [bool; 3]) -> Option<Self> {
    match sources {
      [true, true, true] => Some(PasswordCombination::GoldSilverBronze),
      [true, true, false] => Some(PasswordCombination::GoldSilver),
      [true, false, true] => Some(PasswordCombination::GoldBronze),
      [true, false, false] => Some(PasswordCombination::SilverBronze),
      [false, true, true] => Some(PasswordCombination::Gold),
      [false, true, false] => Some(PasswordCombination::Silver),
      [false, false, true] => Some(PasswordCombination::Bronze),
      [false, false, false] => None,
    }
  }

  pub fn get_name(&self) -> &'static str {
    match self {
      Self::GoldSilverBronze => "金银铜",
      Self::GoldSilver => "金银",
      Self::GoldBronze => "金铜",
      Self::SilverBronze => "银铜",
      Self::Gold => "金",
      Self::Silver => "银",
      Self::Bronze => "铜",
    }
  }
}

#[derive(Debug, Clone, Copy)]
struct InjectTemplate {
  grade_placeholder_positions: [Option<usize>; 3],
  misc_placeholder_positions: &'static [usize],
  address_patch_positions: &'static [(usize, u32)],
  pwd_buffer_start_pos: usize,
  pwd_buffer_end_pos: usize,
  exit_address_pos: usize,
  password_slot_positions: [Option<usize>; 3],
}

/// ROM分析的结果
#[derive(Debug, Clone)]
pub struct TlaRomAnalysis {
  /// 是否与已知原版 ROM 的 CRC32 精确匹配
  pub is_known_rom: bool,
  /// 若该ROM已经注入过自动输入密码程序，记录已有的注入组合
  pub already_injected: Option<PasswordCombination>,
  /// GSPI头将要写入的位置
  pub inject_offset: usize,
  /// 注入所需的最终ROM大小。
  pub final_rom_len: usize,
  /// 空白填充字节（`0x00` 或 `0xFF`）
  pub blank_byte: u8,
  /// 基础游戏版本（决定填入的地址与值）
  pub base_region: GameRegion,
  /// 用于日志展示的ROM名称
  pub rom_name: &'static str,
}

/// 分析ROM是什么版本、能否注入自动输入密码程序、注入位置/偏移在哪里
pub fn analyze(rom: &[u8]) -> Result<TlaRomAnalysis, RomInjectError> {
  // 先获取输入ROM的文件大小
  let file_size = rom.len();

  // 优先与已知的ROM的CRC32进行匹配
  let crc32 = crc32fast::hash(rom);
  if let Some((rom_name, base_region, inject_offset, blank_byte)) =
    KNOWN_INPUT_ROM_INFO_MAP.get(&crc32)
  {
    return Ok(TlaRomAnalysis {
      is_known_rom: true,
      already_injected: None,
      inject_offset: *inject_offset,
      final_rom_len: file_size,
      blank_byte: *blank_byte,
      base_region: *base_region,
      rom_name: *rom_name,
    });
  }

  // 如果不匹配，则判断游戏的语言版本
  let base_region = identify_tla_region(rom)?;
  // 根据语言版本对应的指定偏移记录的默认地址值来判断ROM是否已经注入了自动输入密码程序
  let target_address = read_u32(rom, TARGET_ADDRESS_OFFSET)?;
  if let Some(region) = GameRegion::from_target_address(target_address) {
    if region != base_region {
      return Err(RomInjectError::RegionUnmatched);
    }
    // 能匹配 = 未被注入过 → 需要找插入点
    find_injection_offset(rom, base_region, file_size)
  } else {
    read_existing_injection(rom, base_region, target_address, file_size)
  }
}

/// 靠软件标题 + ROM 内构建日期判定《失落的时代》的语言版本
fn identify_tla_region(rom: &[u8]) -> Result<GameRegion, RomInjectError> {
  let title = read_ascii(rom, SOFTWARE_TITLE_OFFSET, SOFTWARE_TITLE_LEN);
  if title == SOFTWARE_TITLE_TLA_JAPAN {
    // 仅有日版（或基于日版修改的版本）的标题会是 `OUGONTAIYO_B`
    Ok(GameRegion::Japan)
  } else if title == SOFTWARE_TITLE_TLA_WORLD {
    // 其余版本标题都是 `GOLDEN_SUN_B`，改用ROM内的构建日期区分
    let build_date = read_ascii(rom, ROM_BUILD_DATE_OFFSET, ROM_BUILD_DATE_LEN);
    GameRegion::from_rom_build_date(&build_date).ok_or(RomInjectError::UnknownRegion)
  } else {
    Err(RomInjectError::NotTlaRom)
  }
}

fn find_injection_offset(
  rom: &[u8],
  region: GameRegion,
  file_size: usize,
) -> Result<TlaRomAnalysis, RomInjectError> {
  // 若能匹配，则证明游戏ROM没有被注入自动输入密码的程序
  let rom_name = BASE_ROM_NAME[region.index()];
  let last_byte = rom[file_size - 1];
  let blank_byte;

  let remain = file_size % INJECT_ALIGNMENT;
  let padding_size = if remain > 0 {
    INJECT_ALIGNMENT - remain
  } else {
    0
  };

  // 如果文件最后一个字节不是空白字节（0xFF或0x00），那么基本上新程序数据的添加点就是下一个十六字节的起始位置
  if last_byte != 0xFF && last_byte != 0x00 {
    blank_byte = 0x00;

    // 不做情况区分了，直接默认使用最大的金、银、铜的程序的对齐十六字节的大小0x370 + 0x10
    if MAX_ROM_SIZE as usize - file_size - padding_size >= INJECT_BLOCK_MAX_SIZE {
      return Ok(TlaRomAnalysis {
        is_known_rom: false,
        already_injected: None,
        inject_offset: file_size + padding_size,
        final_rom_len: file_size + padding_size + INJECT_BLOCK_MAX_SIZE,
        blank_byte: blank_byte,
        base_region: region,
        rom_name: rom_name,
      });
    } else {
      return Err(RomInjectError::NotEnoughSpace);
    }
  }

  // 若最后一个字节是空白字节（0xFF或0x00），那么需要从文件末尾往前查找空白字节的位置
  blank_byte = last_byte;
  let mut new_pos = 0;
  let mut counter = 0;
  let mut inject_offset_found = false;

  // 如果文件不是16字节对齐，那么优先把未对齐的部分判断一下
  let mut found_pos_in_remain = false;
  for i in file_size - remain..file_size {
    if rom[i] != blank_byte {
      found_pos_in_remain = true;
      break;
    }
  }

  // 如果未对齐的部分里已经有字节不是空白字节，那么插入点就是下一个十六字节的起始位置
  if found_pos_in_remain {
    inject_offset_found = true;
    new_pos = file_size + padding_size;
  } else {
    // 如果未对齐部分都是空白字节，则起始下标跳过未对齐的部分，以16字节为步长往前查找插入的位置
    let start_index = file_size - remain;

    // 先把起始下标赋值（即下一个十六字节的起始位置）（应对第一遍循环就找到的情况）
    let mut new_inject_offset = start_index;
    for i in (0..start_index).step_by(INJECT_ALIGNMENT).rev() {
      for j in 0..INJECT_ALIGNMENT {
        if rom[i - j] != blank_byte {
          inject_offset_found = true;
          break;
        }
      }

      if inject_offset_found {
        new_pos = new_inject_offset;
        break;
      } else {
        new_inject_offset -= INJECT_ALIGNMENT;
        counter += 1;
      }
    }
  }

  if !inject_offset_found {
    return Err(RomInjectError::InjectionPosNotFound);
  }

  if MAX_ROM_SIZE as usize - file_size - (counter * INJECT_ALIGNMENT) >= INJECT_BLOCK_MAX_SIZE {
    return Ok(TlaRomAnalysis {
      is_known_rom: false,
      already_injected: None,
      inject_offset: new_pos,
      final_rom_len: if counter < 38 {
        file_size + INJECT_BLOCK_MAX_SIZE - counter * INJECT_ALIGNMENT + padding_size
      } else {
        file_size
      },
      blank_byte: blank_byte,
      base_region: region,
      rom_name: rom_name,
    });
  } else {
    return Err(RomInjectError::NotEnoughSpace);
  }
}

fn read_existing_injection(
  rom: &[u8],
  region: GameRegion,
  target_address: u32,
  size: usize,
) -> Result<TlaRomAnalysis, RomInjectError> {
  // 若不能匹配，则证明游戏ROM已经被注入自动输入密码的程序，直接使用ROM已经记录的信息，无需二次计算
  let inject_offset =
    target_address as usize - 1 - INJECT_BLOCK_HEADER_SIZE - ROM_MEMORY_MAPPING as usize;
  let header = read_ascii(rom, inject_offset, 4);
  if header.as_bytes() != &INJECT_HEADER {
    return Err(RomInjectError::InjectionUnknown);
  }

  // 虽然不太可能发生，但是以防万一，还是检查一下剩余的空间够不够
  if inject_offset + INJECT_BLOCK_MAX_SIZE > size {
    return Err(RomInjectError::NotEnoughSpace);
  }

  let inject_type_num = read_u32(rom, inject_offset + 4)?;
  let inject_option = PasswordCombination::from(inject_type_num as usize);
  if inject_option.is_none() {
    return Err(RomInjectError::InjectionUnknown);
  }

  let blank_byte = rom[inject_offset + 8];
  let rom_name = BASE_ROM_NAME[region.index()];

  Ok(TlaRomAnalysis {
    is_known_rom: false,
    already_injected: inject_option,
    inject_offset: inject_offset,
    final_rom_len: size,
    blank_byte: blank_byte,
    base_region: region,
    rom_name: rom_name,
  })
}

/// 注入：消费输入 ROM，返回注入后的新 ROM。
///
/// 取代原来的 `inject(path, combination, passwords)`（100 行 + 内部再读一遍文件）。
pub fn inject(
  mut rom: Vec<u8>,
  analysis: &TlaRomAnalysis,
  combination: PasswordCombination,
  passwords: [Option<Vec<u8>>; 3],
) -> Result<Vec<u8>, RomInjectError> {
  // 将ROM调整到此前分析后得出的最终大小
  if rom.len() < analysis.final_rom_len {
    rom.resize(analysis.final_rom_len, analysis.blank_byte);
  }
  // 模板信息
  // 根据密码组合使用不同的模板，依据其中记录的信息修改游戏ROM
  let template = INJECT_TEMPLATES[combination.index()];
  // 要注入的程序，用此前编译好的二进制文件作为基底
  let payload = INJECT_PAYLOADS[combination.index()]; // 原 BIN_BYTES[...]

  let mut cursor = analysis.inject_offset;

  // 写入16字节的头：GSPI + 密码组合编号(LE u32) + 8字节空白
  rom[cursor..cursor + 4].copy_from_slice(&INJECT_HEADER);
  rom[cursor + 4..cursor + 8].copy_from_slice(&(combination.index() as u32).to_le_bytes());
  rom[cursor + 8..cursor + 16].fill(analysis.blank_byte);
  cursor += INJECT_BLOCK_HEADER_SIZE;

  // 若原先注入过程序，先把旧程序体抹成空白（因为新的可能更短）
  if let Some(previous) = analysis.already_injected {
    let old_len = INJECT_PAYLOADS[previous.index()].len();
    rom[cursor..cursor + old_len].fill(analysis.blank_byte);
  } else {
    // 首次注入需要把ROM指定偏移记录的入口地址指向新程序的地址
    let entry = u32::try_from(cursor + ROM_MEMORY_MAPPING as usize + 1).unwrap();
    rom[TARGET_ADDRESS_OFFSET..TARGET_ADDRESS_OFFSET + 4].copy_from_slice(&entry.to_le_bytes());
  }

  // 复制程序到ROM中
  rom[cursor..cursor + payload.len()].copy_from_slice(payload);

  // 按密码的语言版本和级别填入值与地址
  let password_version = PasswordVersion::from(analysis.base_region);
  for (grade_index, pos) in template.grade_placeholder_positions.iter().enumerate() {
    if let Some(pos) = pos {
      rom[cursor + pos] = password_version.diff_pos_password(PasswordGrade::from(grade_index));
    }
  }
  for pos in template.misc_placeholder_positions {
    rom[cursor + *pos] = password_version.diff_pos_other();
  }
  for (pos, value) in template.address_patch_positions {
    let final_value = value + u32::try_from(cursor + ROM_MEMORY_MAPPING as usize).unwrap();
    rom[cursor + pos..cursor + pos + 4].copy_from_slice(&final_value.to_le_bytes());
  }
  write_u32(
    &mut rom,
    cursor + template.pwd_buffer_start_pos,
    analysis.base_region.tla_pwd_buffer_address(),
  );
  write_u32(
    &mut rom,
    cursor + template.pwd_buffer_end_pos,
    analysis.base_region.tla_pwd_address_end(),
  );
  write_u32(
    &mut rom,
    cursor + template.exit_address_pos,
    analysis.base_region.tla_exit_address(),
  );

  // 5. 把三种级别的密码字节放进各自的槽位
  for (i, slot_pos) in template.password_slot_positions.iter().enumerate() {
    if let (Some(bytes), Some(pos)) = (&passwords[i], slot_pos) {
      rom[cursor + pos..cursor + pos + bytes.len()].copy_from_slice(bytes);
    }
  }

  Ok(rom)
}

fn read_ascii(rom: &[u8], pos: usize, size: usize) -> String {
  String::from_utf8_lossy(&rom[pos..pos + size]).into_owned()
}

fn read_u32(rom: &[u8], pos: usize) -> Result<u32, RomInjectError> {
  let bytes = rom[pos..(pos + 4)]
    .try_into()
    .map_err(|_| RomInjectError::UnableToRead)?;
  Ok(u32::from_le_bytes(bytes))
}

fn write_u32(rom: &mut [u8], pos: usize, val: u32) {
  let bytes = u32::to_le_bytes(val);

  for (i, ele) in bytes.iter().enumerate() {
    rom[pos + i] = *ele;
  }
}

const TARGET_ADDRESS: [(u32, GameRegion); 6] = [
  (0x0804_4621, GameRegion::Japan),
  (0x0804_4479, GameRegion::USA),
  (0x0804_44CD, GameRegion::Germany),
  (0x0804_44E5, GameRegion::Spain),
  (0x0804_44E1, GameRegion::France),
  (0x0804_44DD, GameRegion::Italy),
];

impl PasswordVersion {
  fn diff_pos_password(self, grade: PasswordGrade) -> u8 {
    match (grade, self) {
      (PasswordGrade::Gold, Self::Japanese) => 0xEF,
      (PasswordGrade::Gold, Self::English) => 0xF3,
      (PasswordGrade::Silver, Self::Japanese) => 0x28,
      (PasswordGrade::Silver, Self::English) => 0x2C,
      (PasswordGrade::Bronze, Self::Japanese) => 0x05,
      (PasswordGrade::Bronze, Self::English) => 0x01,
    }
  }

  fn diff_pos_other(self) -> u8 {
    match self {
      Self::Japanese => 0x11,
      Self::English => 0x0D,
    }
  }
}

impl GameRegion {
  /// 密码的buffer有260字节，故加获取密码buff结束的地址只需要加上`260 - 1 = 259`
  fn tla_pwd_address_end(self) -> u32 {
    self.tla_pwd_buffer_address() + 259
  }

  fn from_rom_build_date(build_date: &str) -> Option<Self> {
    match build_date {
      "020610172024" => Some(Self::Japan),
      "030205144838" => Some(Self::USA),
      "030623214832" => Some(Self::Germany),
      "030707212715" => Some(Self::Spain),
      "030624204924" => Some(Self::France),
      "030708203047" => Some(Self::Italy),
      _ => None,
    }
  }

  fn from_target_address(target_address: u32) -> Option<Self> {
    TARGET_ADDRESS
      .iter()
      .find(|(x, _)| *x == target_address)
      .map(|(_, y)| *y)
  }

  fn tla_exit_address(self) -> u32 {
    TARGET_ADDRESS[self.index()].0
  }
}
