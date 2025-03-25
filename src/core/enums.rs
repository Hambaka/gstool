#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PasswordGrade {
  Gold,
  Silver,
  Bronze,
}

impl PasswordGrade {
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

  pub fn get_address(self) -> u32 {
    match self {
      GameVersion::Japan => 0x0200_A78A,
      GameVersion::USA => 0x0200_A74A,
      GameVersion::Germany | GameVersion::France | GameVersion::Italy => 0x0200_A742,
      GameVersion::Spain => 0x0200_A73E,
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
