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
pub struct PasswordView {
  pub levels: [u8; 4],
  pub djinn: [[bool; 7]; 4],
  pub events: [bool; 6],
  pub base_stats: [[u16; 6]; 4],
  pub stats: [[u16; 6]; 4],
  pub psy_items: [bool; 8],
  pub items: [[(u16, u8); 15]; 4],
  pub coins: u32,
}
