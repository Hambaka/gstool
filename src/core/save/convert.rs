use crate::core::{
  enums::{GameRegion, GameTitle, NameType},
  save::format::{
    CHECKSUM_OFFSET, LEADER_NAME_FIELD_SIZE, MAX_VALID_SLOT_INDEX, PC_DEFAULT_NAMES,
    PC_NAME_FIELD_SIZE, PC_RECORD_SIZE, SLOT_HEADER_SIZE, SLOT_NUMBER_OFFSET,
  },
};

pub fn convert_save(
  raw_save_file: &mut [u8],
  game_title: GameTitle,
  loop_start_index: usize,
  rename_pcs: Option<NameType>,
  target_region: Option<GameRegion>,
) {
  let save_layout = game_title.save_layout();
  let names_and_len_option = match rename_pcs {
    Some(name_type) => {
      let names = PC_DEFAULT_NAMES[name_type.index()];
      let len = names[0].len();
      Some((names, len))
    }
    None => None,
  };

  for i in loop_start_index..save_layout.max_loop {
    let start_offset = i * save_layout.slot_size;
    let leader_name_offset = start_offset + SLOT_HEADER_SIZE;
    // 虽然游戏的存档文件会记录很多个存档，但是玩家只能在主菜单选择其中的3个存档。
    // 这3个可供玩家选择的存档槽位数字分别是0、1、2
    // 其中，《黄金太阳 开启的封印》有特殊情况：一个完整的存档还有下半部分，也就是分别对应槽位数字3、4、5。
    // 举例来说，0+3组合起来才是一份完整的存档，但在这个场景下存档的下半部分完全可以忽略。
    if raw_save_file[start_offset + SLOT_NUMBER_OFFSET] > MAX_VALID_SLOT_INDEX {
      continue;
    }

    // 简易验证一下存档中第一处记录游戏构建日期的数据，如果没有匹配结果就跳过这个槽位的数据
    let build_date_from_raw_save = u16::from_le_bytes([
      raw_save_file[start_offset + save_layout.build_date_offset[0]],
      raw_save_file[start_offset + save_layout.build_date_offset[0] + 1],
    ]);
    if !save_layout.build_dates.contains(&build_date_from_raw_save) {
      continue;
    }

    if let Some((pc_default_names, name_byte_len)) = names_and_len_option {
      let mut is_main_leader = true;
      /*
      队长角色的名字，但这里指的是仅仅只在存档选择界面出现的名字。
      主角团中的队长名字在存档中有两处，其他主角团成员的名字都只有一处。
      这里的名字最大可12字节，不过一般允许的角色名最大10字节（例如日文版的假名）。

      在《黄金太阳 开启的封印》中，主角团唯一的队长是罗宾。
      在《黄金太阳 失落的时代》中，虽然加西亚是主角团的队长，但是游戏的序幕加斯敏亦是队长。
      */
      if game_title == GameTitle::TheLostAge {
        for j in 0..LEADER_NAME_FIELD_SIZE {
          // 将存档界面的队长名字和游戏实际使用的加西亚的名字进行对比，以此判断当前队长是否是加西亚
          if raw_save_file[leader_name_offset + j]
            != raw_save_file[start_offset
              + save_layout.pc_name_offset
              + save_layout.main_leader_pc_index * PC_RECORD_SIZE
              + j]
          {
            is_main_leader = false;
            break;
          }
        }
      }

      if is_main_leader {
        /*
        《黄金太阳 开启的封印》 + is_main_leader -> 罗宾（is_main_leader永远为真）
        《黄金太阳 失落的时代》 + is_main_leader -> 加西亚
        */
        for j in 0..LEADER_NAME_FIELD_SIZE {
          if j < name_byte_len {
            raw_save_file[leader_name_offset + j] =
              pc_default_names[save_layout.main_leader_pc_index][j];
          } else {
            raw_save_file[leader_name_offset + j] = 0;
          }
        }
      } else {
        // 《黄金太阳 失落的时代》 + !is_main_leader -> 加斯敏
        // 因为加西亚的下一位就是加斯敏，所以索引直接基于加西亚的加1即可。
        for j in 0..LEADER_NAME_FIELD_SIZE {
          if j < name_byte_len {
            raw_save_file[leader_name_offset + j] =
              pc_default_names[save_layout.main_leader_pc_index + 1][j];
          } else {
            raw_save_file[leader_name_offset + j] = 0;
          }
        }
      }

      // 更改主角团所有人的名字
      for j in 0..save_layout.pc_name_record_count {
        for k in 0..PC_NAME_FIELD_SIZE {
          if k < name_byte_len {
            raw_save_file[start_offset + save_layout.pc_name_offset + j * PC_RECORD_SIZE + k] =
              pc_default_names[j][k];
          } else {
            raw_save_file[start_offset + save_layout.pc_name_offset + j * PC_RECORD_SIZE + k] = 0;
          }
        }
      }
    }

    /*
    修改存档中的构建日期（以此实现“转换”存档版本）。
    如果存档文件中的构建日期与游戏的构建日期不匹配，游戏会在读取存档后强制玩家从神殿开始游戏。
    各个语言版本都有各自的构建日期。
    */
    if let Some(region) = target_region {
      let build_date = save_layout.build_dates[region.index()].to_le_bytes();
      for byte_index in 0..2 {
        for entry_index in 0..3 {
          raw_save_file[start_offset + save_layout.build_date_offset[entry_index] + byte_index] =
            build_date[byte_index];
        }
      }
    }

    /*
    计算存档的校验和（16位，但无需计算每个槽位存档的开始的16字节的header）
    如果存档的校验和与header中记录的不一致，游戏将不认为是有效的存档。
    如果校验和的值超过了4位（16进制的4位），直接舍弃多余的位。
    */
    let mut checksum = 0;
    for j in 0..save_layout.checksum_byte_count {
      checksum += u32::from(raw_save_file[leader_name_offset + j]);
    }
    let checksum_bytes = checksum.to_le_bytes();
    for byte_index in 0..2 {
      raw_save_file[start_offset + CHECKSUM_OFFSET + byte_index] = checksum_bytes[byte_index];
    }
  }
}
