#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PasswordGrade {
  Gold = 0,
  Silver = 1,
  Bronze = 2,
}

impl From<usize> for PasswordGrade {
  fn from(value: usize) -> Self {
    match value {
      0 => Self::Gold,
      1 => Self::Silver,
      _ => Self::Bronze,
    }
  }
}

impl PasswordGrade {
  pub const ALL: [Self; 3] = [Self::Gold, Self::Silver, Self::Bronze];

  /// From password char/byte count
  pub fn from_size(size: usize) -> Option<Self> {
    match size {
      260 => Some(Self::Gold),
      61 => Some(Self::Silver),
      16 => Some(Self::Bronze),
      _ => None,
    }
  }

  pub fn include_psy_items(self) -> bool {
    matches!(self, Self::Silver | Self::Bronze)
  }

  pub fn include_stats(self) -> bool {
    matches!(self, Self::Gold | Self::Silver)
  }

  pub fn include_items_and_coins(self) -> bool {
    self == Self::Gold
  }

  pub fn short_name(self) -> &'static str {
    match self {
      Self::Gold => "金",
      Self::Silver => "银",
      Self::Bronze => "铜",
    }
  }

  pub fn priority(self) -> usize {
    self as usize
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PasswordVersion {
  // 平假名
  Japanese = 0,
  // 字母，数字和符号
  English = 1,
}

impl From<GameRegion> for PasswordVersion {
  // GSPI
  fn from(value: GameRegion) -> Self {
    match value {
      GameRegion::Japan => Self::Japanese,
      _ => Self::English,
    }
  }
}

impl PasswordVersion {
  pub const ALL: [Self; 2] = [Self::Japanese, Self::English];

  pub fn opposite(self) -> Self {
    match self {
      Self::Japanese => Self::English,
      Self::English => Self::Japanese,
    }
  }

  pub fn name(self) -> &'static str {
    match self {
      Self::Japanese => "日文",
      Self::English => "英文",
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameRegion {
  Japan = 0,
  USA = 1,
  Germany = 2,
  Spain = 3,
  France = 4,
  Italy = 5,
}

impl GameRegion {
  pub const ALL: [Self; 6] = [
    Self::Japan,
    Self::USA,
    Self::Germany,
    Self::Spain,
    Self::France,
    Self::Italy,
  ];

  pub fn index(self) -> usize {
    self as usize
  }

  pub fn short_name(self) -> &'static str {
    match self {
      Self::Japan => "日",
      Self::USA => "美",
      Self::Germany => "德",
      Self::Spain => "西",
      Self::France => "法",
      Self::Italy => "意",
    }
  }

  pub fn tla_pwd_buffer_address(self) -> u32 {
    match self {
      Self::Japan => 0x0200_A78A,
      Self::USA => 0x0200_A74A,
      Self::Germany | Self::France | Self::Italy => 0x0200_A742,
      Self::Spain => 0x0200_A73E,
    }
  }

  // Raw Save
  pub fn from_save_build_date(build_date: u16) -> Option<Self> {
    match build_date {
      0x159C | 0x198A => Some(Self::Japan),
      0x1652 | 0x1C85 => Some(Self::USA),
      0x1849 | 0x1D97 => Some(Self::Germany),
      0x1885 | 0x1DC7 => Some(Self::Spain),
      0x1713 | 0x1D98 => Some(Self::France),
      0x1886 | 0x1DC8 => Some(Self::Italy),
      _ => None,
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameTitle {
  // GS1
  TheBrokenSeal = 0,
  // GS2
  TheLostAge = 1,
}

impl GameTitle {
  pub fn index(self) -> usize {
    self as usize
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NameType {
  Japanese = 0,
  English = 1,
  German = 2,
  Spanish = 3,
  French = 4,
  Italian = 5,
  // 仅限黄太1
  ChineseFanTranslation2023Team = 6,
  // 仅限黄太2
  ChineseFanTranslationMobileTeam = 7,
}

impl NameType {
  pub const ALL: [Self; 8] = [
    Self::Japanese,
    Self::English,
    Self::German,
    Self::Spanish,
    Self::French,
    Self::Italian,
    Self::ChineseFanTranslation2023Team,
    Self::ChineseFanTranslationMobileTeam,
  ];

  pub fn index(self) -> usize {
    self as usize
  }

  pub fn name(self) -> &'static str {
    match self {
      Self::Japanese => "日文",
      Self::English => "英文",
      Self::German => "德文",
      Self::Spanish => "西班牙文",
      Self::French => "法文",
      Self::Italian => "意大利文",
      Self::ChineseFanTranslation2023Team => "1代新汉化",
      Self::ChineseFanTranslationMobileTeam => "2代汉化",
    }
  }
}

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
