#[derive(Debug)]
pub struct PasswordData {
  pub levels: [u8; 4],
  pub djinn: [u32; 4],
  pub events: [u8; 6],
  pub stats: [[u16; 6]; 4],
  pub items: [[u16; 15]; 4],
  pub coins: u32,
}

#[derive(Debug, Clone)]
pub struct PasswordInfo {
  pub levels: [u8; 4],
  pub djinn: [[bool; 7]; 4],
  pub events: [bool; 6],
  pub base_stats: [[u16; 6]; 4],
  pub stats: [[u16; 6]; 4],
  pub psy_items: [bool; 8],
  pub items: [[(u16, u8); 15]; 4],
  pub coins: u32,
}

impl PasswordInfo {
  pub fn get_levels(&self) -> [u8; 4] {
    self.levels
  }

  pub fn get_djinn(&self) -> [[bool; 7]; 4] {
    self.djinn
  }

  pub fn get_events(&self) -> [bool; 6] {
    self.events
  }

  pub fn get_base_stats(&self) -> [[u16; 6]; 4] {
    self.base_stats
  }

  pub fn get_stats(&self) -> [[u16; 6]; 4] {
    self.stats
  }

  pub fn get_psy_items(&self) -> [bool; 8] {
    self.psy_items
  }

  pub fn get_items(&self) -> [[(u16, u8); 15]; 4] {
    self.items
  }

  pub fn get_coins(&self) -> u32 {
    self.coins
  }
}
