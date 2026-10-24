use crate::core::{
  bits::BitVec,
  password::{
    PasswordData, PasswordView,
    encode::password_data_from_save_slot,
    items::{PSYNERGY_ITEMS, STACKABLE_ITEMS},
    stats::in_game_stats,
  },
};

/// 帮忙搭救乌鲁木齐事件的索引/下标，准备对值进行反转
const INVERTED_EVENT_INDEX: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Element {
  Venus = 0,
  Mercury = 1,
  Mars = 2,
  Jupiter = 3,
}

impl Element {
  pub const ALL: [Self; 4] = [Self::Venus, Self::Mercury, Self::Mars, Self::Jupiter];
  /// 游戏内展示顺序（地/火/风/水），用于 PasswordView（密码预览窗口）与Dyrati数据文本。
  pub const GAMEPLAY_ORDER: [Self; 4] = [Self::Venus, Self::Mars, Self::Jupiter, Self::Mercury];

  pub fn index(self) -> usize {
    self as usize
  }

  pub fn name(self) -> &'static str {
    match self {
      Self::Venus => "维纳斯",
      Self::Mercury => "墨丘利",
      Self::Mars => "玛尔斯",
      Self::Jupiter => "朱庇特",
    }
  }
}

pub fn view_from_data(data: &PasswordData) -> PasswordView {
  // Levels
  let levels = data.levels;
  // Djinn
  let mut djinn_bits = vec![BitVec::with_capacity(7); 4];

  for ele in Element::ALL {
    djinn_bits[ele.index()].push_bits(data.djinn[ele.index()], 7);
  }

  let mut djinn = [[false; 7]; 4];

  // Reversed bit order & Adjust djinn to gameplay order for easier viewing.
  for i in 0..7 {
    // 将密码数据记录的默认顺序的精灵数据，按照游戏排列顺序存储
    for (slot, ele) in Element::GAMEPLAY_ORDER.iter().enumerate() {
      if djinn_bits[ele.index()].bit_at(6 - i) == 1 {
        djinn[slot][i] = true;
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
  for i in 0..15 {
    for j in 0..4 {
      let item = data.items[j][i];
      let item_id = item & 0x1FF;

      if item_id != 0 {
        let count = if STACKABLE_ITEMS.contains(&item_id) {
          (item >> 11) + 1
        } else {
          1
        };
        items[j][i] = (item_id, (count & 0b1_1111) as u8);

        if PSYNERGY_ITEMS.contains(&item_id) {
          psy_items[(item_id - PSYNERGY_ITEMS[0]) as usize] = true;
        }
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
