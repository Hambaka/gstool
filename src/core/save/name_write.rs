//! 把已编码的自定义角色名写进《开启的封印》存档的某个槽位。
//!
//! 这里是 [`pc_name`](super::pc_name) 的写侧：`encode_pc_name` 负责“文本 → 字节”，
//! 本模块负责“字节 → 存档文件”，并重算名字字段和校验和。
//!
//! 只写**一个**槽位（界面选中的那个最高优先级副本），不碰其他槽位、不碰第二半存档。
//! 校验和算法与 `convert.rs::convert_save` 同源，改动时必须两边一起看。

use std::fmt;

use crate::core::save::format::{
  CHECKSUM_OFFSET, LEADER_NAME_FIELD_SIZE, PC_NAME_FIELD_SIZE, PC_RECORD_SIZE, SLOT_HEADER_SIZE,
  SaveLayout,
};

/// 写名字时可能遇到的存档格式问题。
#[derive(Debug)]
pub enum SaveWriteError {
  /// 名字字段 / 校验和字段超出了存档长度
  FieldOutOfRange,
}

impl fmt::Display for SaveWriteError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::FieldOutOfRange => write!(f, "写入数据时发生错误（字段超出范围）！"),
    }
  }
}

/// 把一个槽位的自定义角色名写进 `raw`（原地修改）。
///
/// - `slot_number`：0/1/2，对应界面上的 1/2/3 号存档；
/// - `section_index`：该槽位在文件中的段索引（`SaveSlot::section_index`）；
/// - `names`：`(角色下标, 已编码字节)`，只写传入的项；
/// - `leader`：队长名字节（存档选择界面显示的那一份），`None` 表示不动。
///
/// 写完后会重算该槽位的校验和。任何格式问题都返回 `Err`，不会 panic。
pub fn write_custom_names(
  raw: &mut [u8],
  layout: &SaveLayout,
  section_index: usize,
  names: &[(usize, &[u8])],
  leader: Option<&[u8]>,
) -> Result<(), SaveWriteError> {
  let section_start = section_index * layout.slot_size;

  for &(name_index, bytes) in names {
    let record_offset = name_index * PC_RECORD_SIZE;
    let offset = section_start + layout.pc_name_offset + record_offset;
    write_name_field(raw, offset, PC_NAME_FIELD_SIZE, bytes)?;
  }

  // 存档选择界面显示的队长名。
  if let Some(bytes) = leader {
    let offset = section_start + SLOT_HEADER_SIZE;
    write_name_field(raw, offset, LEADER_NAME_FIELD_SIZE, bytes)?;
  }

  update_checksum(raw, section_start, layout)
}

/// 把名字字节写进指定长度的字节，先清零再复制
fn write_name_field(
  raw: &mut [u8],
  offset: usize,
  field_size: usize,
  bytes: &[u8],
) -> Result<(), SaveWriteError> {
  let end = offset + field_size;
  let name_field = raw
    .get_mut(offset..end)
    .ok_or(SaveWriteError::FieldOutOfRange)?;

  name_field.fill(0);
  let len = bytes.len().min(field_size);

  let (Some(name_source), Some(target)) = (bytes.get(..len), name_field.get_mut(..len)) else {
    return Err(SaveWriteError::FieldOutOfRange);
  };
  target.copy_from_slice(name_source);

  Ok(())
}

/// 重算一个槽位的 16 位校验和：`0x10..槽位末尾` 逐字节求和，取低 16 位。
///
/// 与 `convert.rs::convert_save` 中的算法一致。
fn update_checksum(
  raw: &mut [u8],
  section_start: usize,
  layout: &SaveLayout,
) -> Result<(), SaveWriteError> {
  let sum_start = section_start + SLOT_HEADER_SIZE;
  let sum_end = sum_start + layout.checksum_byte_count;
  let checksum = raw
    .get(sum_start..sum_end)
    .ok_or(SaveWriteError::FieldOutOfRange)?
    .iter()
    .map(|&byte| u32::from(byte))
    .sum::<u32>();

  let checksum_start = section_start + CHECKSUM_OFFSET;
  let checksum_end = checksum_start + 2;
  raw
    .get_mut(checksum_start..checksum_end)
    .ok_or(SaveWriteError::FieldOutOfRange)?
    .copy_from_slice(&checksum.to_le_bytes()[..2]);

  Ok(())
}
