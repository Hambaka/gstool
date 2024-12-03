use clap::builder::PossibleValue;
use clap::ValueEnum;

#[derive(Debug, Clone, Copy)]
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
}

impl ValueEnum for PasswordGrade {
  fn value_variants<'a>() -> &'a [Self] {
    &[Self::Gold, Self::Silver, Self::Bronze]
  }

  fn to_possible_value(&self) -> Option<PossibleValue> {
    Some(match self {
      Self::Gold => PossibleValue::new("金").help("Gold grade password"),
      Self::Silver => PossibleValue::new("银").help("Silver grade password"),
      Self::Bronze => PossibleValue::new("铜").help("Bronze grade password"),
    })
  }
}

#[derive(Debug, Clone, Copy)]
pub enum PasswordVersion {
  //Hiragana. Japanese version only
  Japanese,
  //Letter, number, and symbol.
  English,
}

impl PasswordVersion {
  pub fn rev(self) -> PasswordVersion {
    match self {
      PasswordVersion::Japanese => PasswordVersion::English,
      PasswordVersion::English => PasswordVersion::Japanese,
    }
  }
}

impl ValueEnum for PasswordVersion {
  fn value_variants<'a>() -> &'a [Self] {
    &[Self::Japanese, Self::English]
  }

  fn to_possible_value(&self) -> Option<PossibleValue> {
    Some(match self {
      Self::Japanese => PossibleValue::new("日文").help("Japanese version password"),
      Self::English => PossibleValue::new("英文").help("English version password"),
    })
  }
}

#[derive(Debug, Clone, Copy)]
pub enum CheatVersion {
  Japan,
  USA,
  Germany,
  Spain,
  France,
  Italy,
}

// For Golden Sun: Lost Age
impl CheatVersion {
  pub fn get_address(self) -> u32 {
    match self {
      // 0x0200A78A
      CheatVersion::Japan => 0x0200_A78A,
      // 0x0200A74A
      CheatVersion::USA => 0x0200_A74A,
      // 0x0200A742
      CheatVersion::Germany | CheatVersion::France | CheatVersion::Italy => 0x0200_A742,
      // 0x0200A73E
      CheatVersion::Spain => 0x0200_A73E,
    }
  }
}

impl ValueEnum for CheatVersion {
  fn value_variants<'a>() -> &'a [Self] {
    &[Self::Japan, Self::USA, Self::Germany, Self::Spain, Self::France, Self::Italy]
  }

  fn to_possible_value(&self) -> Option<PossibleValue> {
    Some(match self {
      Self::Japan => PossibleValue::new("日").help("Ougon no Taiyou - Ushinawareshi Toki (Japan)"),
      Self::USA => PossibleValue::new("美").help("Golden Sun - The Lost Age (USA, Europe)"),
      Self::Germany => PossibleValue::new("德").help("Golden Sun - Die Vergessene Epoche (Germany)"),
      Self::Spain => PossibleValue::new("西").help("Golden Sun - La Edad Perdida (Spain)"),
      Self::France => PossibleValue::new("法").help("Golden Sun - L'Age Perdu (France)"),
      Self::Italy => PossibleValue::new("意").help("Golden Sun - L'Era Perduta (Italy)"),
    })
  }
}

#[derive(Debug, Clone, Copy)]
pub enum GameType {
  // GS1
  TheBrokenSeal,
  // GS2
  TheLostAge,
}

#[derive(Debug, Clone, Copy)]
pub enum NameType {
  Japanese,
  English,
  German,
  Spanish,
  French,
  Italian,
  // TLA only
  ChineseFanTranslationMobileTeam,
  // TBS only
  ChineseFanTranslation2023Team,
}

impl ValueEnum for NameType {
  fn value_variants<'a>() -> &'a [Self] {
    &[Self::Japanese, Self::English, Self::German, Self::Spanish, Self::French, Self::Italian,
      Self::ChineseFanTranslationMobileTeam, Self::ChineseFanTranslation2023Team]
  }

  fn to_possible_value(&self) -> Option<PossibleValue> {
    Some(match self {
      Self::Japanese => PossibleValue::new("日文").help("Japanese"),
      Self::English => PossibleValue::new("英文").help("English"),
      Self::German => PossibleValue::new("德文").help("German"),
      Self::Spanish => PossibleValue::new("西班牙文").help("Spanish"),
      Self::French => PossibleValue::new("法文").help("French"),
      Self::Italian => PossibleValue::new("意大利文").help("Italian"),
      Self::ChineseFanTranslationMobileTeam => PossibleValue::new("2代汉化").help("Chinese fan translation by Mobile Team, TLA only"),
      Self::ChineseFanTranslation2023Team => PossibleValue::new("1代新汉化").help("Chinese fan translation by 2023 Team, TBS only"),
    })
  }
}

#[derive(Debug, Clone, Copy)]
pub enum BuildDateType {
  Japan,
  // USA/Europe
  USA,
  Germany,
  Spain,
  France,
  Italy,
}

impl ValueEnum for BuildDateType {
  fn value_variants<'a>() -> &'a [Self] {
    &[Self::Japan, Self::USA, Self::Germany, Self::Spain, Self::France, Self::Italy]
  }

  fn to_possible_value(&self) -> Option<PossibleValue> {
    Some(match self {
      Self::Japan => PossibleValue::new("日").help("Japan"),
      Self::USA => PossibleValue::new("美").help("USA, Europe"),
      Self::Germany => PossibleValue::new("德").help("Germany"),
      Self::Spain => PossibleValue::new("西").help("Spain"),
      Self::France => PossibleValue::new("法").help("France"),
      Self::Italy => PossibleValue::new("意").help("Italy"),
    })
  }
}
