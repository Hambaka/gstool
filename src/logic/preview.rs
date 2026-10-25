//! 密码信息预览窗口状态。
//!
//! 读取`PasswordSources`中的存档/文本/内存转储数据，生成预览数据供窗口显示。
//! 窗口在ui侧的`ui/window/pwd_info.rs`，只消费 `PwdInfoState`。

use crate::{
  core::{
    bits::BitVec,
    data::Language,
    enums::PasswordGrade,
    password::{read_from_bits, structs::PasswordView, view_from_data, view_from_raw_save},
  },
  logic::{command::SubCommandEnum, save_slot::SaveSlotEnum, sources::PasswordSources},
};

/// 预览数据：密码级别 + 解析出的 `PasswordView`
#[derive(Debug, Clone)]
pub struct PreviewData {
  grade: PasswordGrade,
  view: PasswordView,
}

impl PreviewData {
  pub fn new(grade: PasswordGrade, view: PasswordView) -> Self {
    PreviewData { grade, view }
  }

  pub fn grade(&self) -> PasswordGrade {
    self.grade
  }

  /// 角色等级
  pub fn levels(&self, character_idx: usize) -> u8 {
    self.view.levels[character_idx]
  }

  /// 精灵状态
  pub fn djinn(&self, element_idx: usize, djinn_idx: usize) -> bool {
    self.view.djinn[element_idx][djinn_idx]
  }

  /// 事件状态
  pub fn events(&self, event_idx: usize) -> bool {
    self.view.events[event_idx]
  }

  /// 基础能力状态（金、银）
  pub fn base_stats(&self, character_idx: usize, stat_idx: usize) -> u16 {
    self.view.base_stats[character_idx][stat_idx]
  }

  /// 能力状态（金、银）
  pub fn stats(&self, character_idx: usize, stat_idx: usize) -> u16 {
    self.view.stats[character_idx][stat_idx]
  }

  /// 精神力道具（银、铜）
  pub fn psy_items(&self, psy_item_idx: usize) -> bool {
    self.view.psy_items[psy_item_idx]
  }

  /// 物品（仅金密码有）
  pub fn items(&self, character_idx: usize, inventory_idx: usize) -> (u16, u8) {
    self.view.items[character_idx][inventory_idx]
  }

  /// 金币数（仅金密码有）
  pub fn coins(&self) -> u32 {
    self.view.coins
  }
}

/// 预览的显示模式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PreviewModeEnum {
  /// 基础能力状态属性
  Main = 0,
  /// 能力状态属性
  Alt = 1,
}

/// 标签页（角色/物品/精灵/其他）
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PreviewPageEnum {
  /// 角色
  Character = 0,
  /// 物品
  Item = 1,
  /// 精灵
  Djinn = 2,
  /// 其他
  Misc = 3,
}

impl PreviewPageEnum {
  pub const ALL: [Self; 4] = [Self::Character, Self::Item, Self::Djinn, Self::Misc];

  pub fn name(&self) -> &'static str {
    match self {
      Self::Character => "角色",
      Self::Item => "物品",
      Self::Djinn => "精灵",
      Self::Misc => "其他",
    }
  }
}

/// 预览窗口状态
#[derive(Debug)]
pub struct PwdInfoState {
  /// 三个槽位的预览数据（`None` 表示该槽位无数据）
  pub data: [Option<PreviewData>; 3],
  /// 当前选中的存档槽位
  pub slot: SaveSlotEnum,
  /// 当前选中的页签（角色/物品/精灵/其他）
  pub page: PreviewPageEnum,
  /// 角色页：显示角色属性 or 不显示
  pub char_mode: PreviewModeEnum,
  /// 角色页：基础能力值属性 or 游戏内默认能力值
  pub stats_mode: PreviewModeEnum,
  /// 物品页：显示全物品 or 显示精神力物品
  pub item_mode: PreviewModeEnum,
  /// 物品名称语言
  pub item_lang: Language,
  /// 精灵名称语言
  pub djinn_lang: Language,
  /// 窗口是否打开
  pub window_open: bool,
}

impl Default for PwdInfoState {
  fn default() -> Self {
    Self {
      data: [None, None, None],
      slot: SaveSlotEnum::First,
      page: PreviewPageEnum::Character,
      char_mode: PreviewModeEnum::Main,
      stats_mode: PreviewModeEnum::Main,
      item_mode: PreviewModeEnum::Main,
      item_lang: Language::ChineseTBS,
      djinn_lang: Language::ChineseTBS,
      window_open: false,
    }
  }
}

impl PwdInfoState {
  /// 从密码来源加载预览数据
  pub fn load_from(
    &mut self,
    sources: &PasswordSources,
    sub: SubCommandEnum,
    slot: Option<SaveSlotEnum>,
  ) {
    match sub {
      SubCommandEnum::First => {
        self.data = [None, None, None];

        for (index, slot) in sources.from_save.slots.iter().enumerate() {
          if let Some(raw_save) = slot {
            let pwd_info = view_from_raw_save(&raw_save.data);
            self.data[index] = Some(PreviewData::new(PasswordGrade::Gold, pwd_info));
          }
        }

        // 理论上.unwrap_or应该是不会触发的……
        self.slot = slot
          .or_else(|| sources.first_available_slot())
          .unwrap_or(SaveSlotEnum::First);
        self.page = PreviewPageEnum::Character;
        // 显示角色的全部属性
        self.char_mode = PreviewModeEnum::Main;
        // 显示基础状态属性
        self.stats_mode = PreviewModeEnum::Main;
        // 显示全部物品
        self.item_mode = PreviewModeEnum::Main;

        self.window_open = true;
      }
      SubCommandEnum::Second => {
        self.load_from_text_or_dump(sources.from_text.as_ref().map(|p| (&p.bits, p.grade)));
      }
      SubCommandEnum::Third => {
        self.load_from_text_or_dump(sources.from_dump.as_ref().map(|p| (&p.bits, p.grade)));
      }
    }
  }

  /// 从密码文本或内存转储来源加载预览数据
  fn load_from_text_or_dump(&mut self, src: Option<(&BitVec, PasswordGrade)>) {
    self.data = [None, None, None];

    if let Some((bits, src_grade)) = src {
      let password_data = read_from_bits(bits, src_grade);
      let pwd_info = view_from_data(&password_data);
      self.data[0] = Some(PreviewData::new(src_grade, pwd_info));
      self.slot = SaveSlotEnum::First;

      match src_grade {
        PasswordGrade::Gold => {
          // 显示角色的全部属性
          self.char_mode = PreviewModeEnum::Main;
          // 显示角色拥有的全部物品
          self.item_mode = PreviewModeEnum::Main;
        }
        PasswordGrade::Silver => {
          // 显示角色的全部属性
          self.char_mode = PreviewModeEnum::Main;
          // 显示角色拥有的全部物品
          self.item_mode = PreviewModeEnum::Alt;
        }
        PasswordGrade::Bronze => {
          // 仅显示角色的等级
          self.char_mode = PreviewModeEnum::Alt;
          // 仅显示角色拥有的精神力物品
          self.item_mode = PreviewModeEnum::Alt;
        }
      }
    }

    self.page = PreviewPageEnum::Character;
    self.stats_mode = PreviewModeEnum::Main;
    self.window_open = true;
  }
}
