#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PasswordGrade {
  Gold,
  Silver,
  Bronze,
}

impl PasswordGrade {
  pub const PASSWORD_GRADES: [PasswordGrade; 3] = [
    PasswordGrade::Gold,
    PasswordGrade::Silver,
    PasswordGrade::Bronze,
  ];

  pub fn is_gold(self) -> bool {
    matches!(self, PasswordGrade::Gold)
  }

  pub fn is_silver(self) -> bool {
    matches!(self, PasswordGrade::Silver)
  }

  pub fn is_bronze(self) -> bool {
    matches!(self, PasswordGrade::Bronze)
  }

  pub fn get_name(self) -> &'static str {
    match self {
      PasswordGrade::Gold => "金",
      PasswordGrade::Silver => "银",
      PasswordGrade::Bronze => "铜",
    }
  }

  pub fn get_priority(self) -> usize {
    self as usize
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PasswordVersion {
  // 平假名
  Japanese,
  // 字母，数字和符号
  English,
}

impl PasswordVersion {
  pub const PASSWORD_VERSIONS: [PasswordVersion; 2] =
    [PasswordVersion::Japanese, PasswordVersion::English];

  pub fn rev(self) -> PasswordVersion {
    match self {
      PasswordVersion::Japanese => PasswordVersion::English,
      PasswordVersion::English => PasswordVersion::Japanese,
    }
  }

  pub fn get_name(self) -> &'static str {
    match self {
      PasswordVersion::Japanese => "日文",
      PasswordVersion::English => "英文",
    }
  }

  // GSPI
  pub fn from(game_version: GameVersion) -> Self {
    match game_version {
      GameVersion::Japan => PasswordVersion::Japanese,
      _ => PasswordVersion::English,
    }
  }

  // GSPI
  pub fn get_diff_u8_gold(self) -> u8 {
    match self {
      PasswordVersion::Japanese => 0xEF,
      PasswordVersion::English => 0xF3,
    }
  }

  // GSPI
  pub fn get_diff_u8_silver(self) -> u8 {
    match self {
      PasswordVersion::Japanese => 0x28,
      PasswordVersion::English => 0x2C,
    }
  }

  // GSPI
  pub fn get_diff_u8_bronze(self) -> u8 {
    match self {
      PasswordVersion::Japanese => 0x05,
      PasswordVersion::English => 0x01,
    }
  }

  // GSPI
  pub fn get_diff_u8_other(self) -> u8 {
    match self {
      PasswordVersion::Japanese => 0x11,
      PasswordVersion::English => 0x0D,
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameVersion {
  Japan,
  USA,
  Germany,
  Spain,
  France,
  Italy,
}

impl GameVersion {
  pub const GAME_VERSIONS: [GameVersion; 6] = [
    GameVersion::Japan,
    GameVersion::USA,
    GameVersion::Germany,
    GameVersion::Spain,
    GameVersion::France,
    GameVersion::Italy,
  ];

  pub fn get_name(self) -> &'static str {
    match self {
      GameVersion::Japan => "日",
      GameVersion::USA => "美",
      GameVersion::Germany => "德",
      GameVersion::Spain => "西",
      GameVersion::France => "法",
      GameVersion::Italy => "意",
    }
  }

  pub fn get_tla_pwd_buffer_address(self) -> u32 {
    match self {
      GameVersion::Japan => 0x0200_A78A,
      GameVersion::USA => 0x0200_A74A,
      GameVersion::Germany | GameVersion::France | GameVersion::Italy => 0x0200_A742,
      GameVersion::Spain => 0x0200_A73E,
    }
  }

  // GSPI
  pub fn get_tla_pwd_address_end(self) -> u32 {
    self.get_tla_pwd_buffer_address() + 259
  }

  // Raw Save
  pub fn from_save_build_date(build_date: u16) -> Option<Self> {
    match build_date {
      0x159C | 0x198A => Some(GameVersion::Japan),
      0x1652 | 0x1C85 => Some(GameVersion::USA),
      0x1849 | 0x1D97 => Some(GameVersion::Germany),
      0x1885 | 0x1DC7 => Some(GameVersion::Spain),
      0x1713 | 0x1D98 => Some(GameVersion::France),
      0x1886 | 0x1DC8 => Some(GameVersion::Italy),
      _ => None,
    }
  }

  // GSPI
  pub fn from_rom_build_date(build_date: &str) -> Option<Self> {
    match build_date {
      "020610172024" => Some(GameVersion::Japan),
      "030205144838" => Some(GameVersion::USA),
      "030623214832" => Some(GameVersion::Germany),
      "030707212715" => Some(GameVersion::Spain),
      "030624204924" => Some(GameVersion::France),
      "030708203047" => Some(GameVersion::Italy),
      _ => None,
    }
  }

  // GSPI
  pub fn from_target_address(target_address: u32) -> Option<Self> {
    match target_address {
      0x0804_4621 => Some(GameVersion::Japan),
      0x0804_4479 => Some(GameVersion::USA),
      0x0804_44CD => Some(GameVersion::Germany),
      0x0804_44E5 => Some(GameVersion::Spain),
      0x0804_44E1 => Some(GameVersion::France),
      0x0804_44DD => Some(GameVersion::Italy),
      _ => None,
    }
  }

  // GSPI
  pub fn get_tla_exit_address(self) -> u32 {
    match self {
      GameVersion::Japan => 0x0804_4621,
      GameVersion::USA => 0x0804_4479,
      GameVersion::Germany => 0x0804_44CD,
      GameVersion::Spain => 0x0804_44E5,
      GameVersion::France => 0x0804_44E1,
      GameVersion::Italy => 0x0804_44DD,
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub enum GameType {
  // GS1
  TheBrokenSeal,
  // GS2
  TheLostAge,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NameType {
  Japanese,
  English,
  German,
  Spanish,
  French,
  Italian,
  // 仅限黄太2
  ChineseFanTranslationMobileTeam,
  // 仅限黄太1
  ChineseFanTranslation2023Team,
}

impl NameType {
  pub fn get_name(self) -> &'static str {
    match self {
      NameType::Japanese => "日文",
      NameType::English => "英文",
      NameType::German => "德文",
      NameType::Spanish => "西班牙文",
      NameType::French => "法文",
      NameType::Italian => "意大利文",
      NameType::ChineseFanTranslationMobileTeam => "2代汉化",
      NameType::ChineseFanTranslation2023Team => "1代新汉化",
    }
  }
}
