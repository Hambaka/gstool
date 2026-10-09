const TARGET_ADDRESS: [(u32, GameRegion); 6] = [
  (0x0804_4621, GameRegion::Japan),
  (0x0804_4479, GameRegion::USA),
  (0x0804_44CD, GameRegion::Germany),
  (0x0804_44E5, GameRegion::Spain),
  (0x0804_44E1, GameRegion::France),
  (0x0804_44DD, GameRegion::Italy),
];

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
  pub const ALL: [PasswordGrade; 3] = [Self::Gold, Self::Silver, Self::Bronze];

  pub fn from_size(size: usize) -> Option<Self> {
    match size {
      260 => Some(Self::Gold),
      61 => Some(Self::Silver),
      16 => Some(Self::Bronze),
      _ => None,
    }
  }

  pub fn is_gold(self) -> bool {
    matches!(self, Self::Gold)
  }

  pub fn is_silver(self) -> bool {
    matches!(self, Self::Silver)
  }

  pub fn is_bronze(self) -> bool {
    matches!(self, Self::Bronze)
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
  pub const ALL: [PasswordVersion; 2] = [Self::Japanese, Self::English];

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

  // GSPI
  pub fn diff_pos_password(self, grade: PasswordGrade) -> u8 {
    match (grade, self) {
      (PasswordGrade::Gold, Self::Japanese) => 0xEF,
      (PasswordGrade::Gold, Self::English) => 0xF3,
      (PasswordGrade::Silver, Self::Japanese) => 0x28,
      (PasswordGrade::Silver, Self::English) => 0x2C,
      (PasswordGrade::Bronze, Self::Japanese) => 0x05,
      (PasswordGrade::Bronze, Self::English) => 0x01,
    }
  }

  // GSPI
  pub fn diff_pos_other(self) -> u8 {
    match self {
      Self::Japanese => 0x11,
      Self::English => 0x0D,
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
  pub const ALL: [GameRegion; 6] = [
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

  pub fn get_tla_pwd_buffer_address(self) -> u32 {
    match self {
      Self::Japan => 0x0200_A78A,
      Self::USA => 0x0200_A74A,
      Self::Germany | Self::France | Self::Italy => 0x0200_A742,
      Self::Spain => 0x0200_A73E,
    }
  }

  // GSPI
  pub fn get_tla_pwd_address_end(self) -> u32 {
    // 密码的buffer有260字节，故加260-1=259
    self.get_tla_pwd_buffer_address() + 259
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

  // GSPI
  pub fn from_rom_build_date(build_date: &str) -> Option<Self> {
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

  // GSPI
  pub fn from_target_address(target_address: u32) -> Option<Self> {
    TARGET_ADDRESS
      .iter()
      .find(|(x, _)| *x == target_address)
      .map(|(_, y)| *y)
  }

  // GSPI
  pub fn get_tla_exit_address(self) -> u32 {
    TARGET_ADDRESS
      .iter()
      .find(|(_, y)| self == *y)
      .map(|(x, _)| *x)
      .unwrap()
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
  // 仅限黄太2
  ChineseFanTranslationMobileTeam = 6,
  // 仅限黄太1
  ChineseFanTranslation2023Team = 7,
}

impl NameType {
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
      Self::ChineseFanTranslationMobileTeam => "2代汉化",
      Self::ChineseFanTranslation2023Team => "1代新汉化",
    }
  }
}
