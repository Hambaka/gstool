use crate::core::{
  bits::BitVec,
  enums::Element,
  password::{
    PasswordData, PasswordView,
    encode::password_data_from_save_slot,
    items::{PSYNERGY_ITEMS, STACKABLE_ITEMS},
    stats::in_game_stats,
  },
};

/// 帮忙搭救乌鲁木齐事件的索引/下标，准备对值进行反转
const INVERTED_EVENT_INDEX: usize = 2;

pub fn view_from_data(data: &PasswordData) -> PasswordView {
  // Levels
  let levels = data.levels;
  // Djinn
  let mut djinn_bits = vec![BitVec::with_capacity(7); 4];

  for ele in Element::ALL {
    djinn_bits[ele.index()].push_bits(data.djinn[ele.index()], 7);
  }

  let mut djinn = [[false; 7]; 4];

  // 将密码数据记录的默认顺序的精灵数据，按照游戏排列顺序存储
  for (slot, ele) in Element::GAMEPLAY_ORDER.iter().enumerate() {
    // Reversed bit order & Adjust djinn to gameplay order for easier viewing.
    for (i, djinn) in djinn[slot].iter_mut().enumerate() {
      if djinn_bits[ele.index()].bit_at(6 - i) == 1 {
        *djinn = true;
      }
    }
  }

  // Events
  let mut events = [false; 6];
  for (i, event) in data.events.iter().enumerate() {
    events[i] = if i == INVERTED_EVENT_INDEX {
      // 简化自：`(event ^ 1) & 1 == 1`
      *event == 0
    } else {
      *event == 1
    }
  }
  // Base Stats
  let base_stats = data.stats;
  // Stats
  let stats = in_game_stats(base_stats);
  // Psynergy Items
  let mut psy_items = [false; 8];
  // Items
  let mut items = [[(0, 0); 15]; 4];
  for (item_pwd_data, item_new) in data.items.iter().flatten().zip(items.iter_mut().flatten()) {
    let item_id = item_pwd_data & 0x1FF;

    if item_id != 0 {
      let count = if STACKABLE_ITEMS.contains(&item_id) {
        (item_pwd_data >> 11) + 1
      } else {
        1
      };
      *item_new = (item_id, (count & 0b1_1111) as u8);

      if PSYNERGY_ITEMS.contains(&item_id) {
        psy_items[(item_id - PSYNERGY_ITEMS[0]) as usize] = true;
      }
    }
  }
  // Coins
  let coins = data.coins;

  PasswordView {
    levels,
    djinn,
    events,
    base_stats,
    stats,
    psy_items,
    items,
    coins,
  }
}

pub fn view_from_raw_save(raw_save: &[u8]) -> PasswordView {
  let save_data = password_data_from_save_slot(raw_save);
  view_from_data(&save_data)
}
