// Port from Dyrati's "Golden-Sun-Password-Transfer" lua script for "vba-rr" and "Bizhawk" emulators.
#[derive(Debug, Clone)]
pub struct BitVec {
  bits: Vec<u8>,
}

// Port from Dyrati's "Golden-Sun-Password-Transfer" lua script for "vba-rr" and "Bizhawk" emulators.
impl Default for BitVec {
  fn default() -> Self {
    Self::new()
  }
}

impl From<&[u8]> for BitVec {
  fn from(value: &[u8]) -> Self {
    BitVec {
      bits: Vec::from(value),
    }
  }
}

impl BitVec {
  pub fn new() -> Self {
    BitVec { bits: Vec::new() }
  }

  pub fn as_slice(&self) -> &[u8] {
    self.bits.as_slice()
  }

  pub fn len(&self) -> usize {
    self.bits.len()
  }

  pub fn remove(&mut self, index: usize) {
    self.bits.remove(index);
  }

  pub fn bit_at(&self, index: usize) -> u8 {
    self.bits[index]
  }

  // Link: https://github.com/Dyrati/Golden-Sun-Password-Transfer/blob/5ec2d52553ec8f4e0fe77854bc2b31956ac09a11/password.lua#L16
  pub fn push_bit(&mut self, value: u32) {
    self.bits.push((value & 1) as u8);
  }

  // Link: https://github.com/Dyrati/Golden-Sun-Password-Transfer/blob/5ec2d52553ec8f4e0fe77854bc2b31956ac09a11/password.lua#L16
  pub fn push_bits(&mut self, value: u32, size: usize) {
    for i in 0..size {
      self.bits.push(((value >> (size - i - 1)) & 1) as u8);
    }
  }

  // Link: https://github.com/Dyrati/Golden-Sun-Password-Transfer/blob/5ec2d52553ec8f4e0fe77854bc2b31956ac09a11/password.lua#L16
  pub fn replace_bits(&mut self, value: u32, size: usize, pos: usize) {
    for i in 0..size {
      self.bits[pos + i] = ((value >> (size - i - 1)) & 1) as u8;
    }
  }

  // Link: https://github.com/Dyrati/Golden-Sun-Password-Transfer/blob/5ec2d52553ec8f4e0fe77854bc2b31956ac09a11/password.lua#L23
  pub fn sub_bits_u8(&self, min: usize, max: usize) -> u8 {
    let mut acc = 0;
    for i in 0..=(max - min) {
      if min < self.bits.len() {
        acc = 2 * acc + self.bits[min + i];
      } else {
        acc *= 2;
      }
    }
    acc
  }

  // Link: https://github.com/Dyrati/Golden-Sun-Password-Transfer/blob/5ec2d52553ec8f4e0fe77854bc2b31956ac09a11/password.lua#L23
  pub fn sub_bits_u16(&self, min: usize, max: usize) -> u16 {
    let mut acc = 0;
    for i in 0..=(max - min) {
      if min < self.bits.len() {
        acc = 2 * acc + u16::from(self.bits[min + i]);
      } else {
        acc *= 2;
      }
    }
    acc
  }

  // Link: https://github.com/Dyrati/Golden-Sun-Password-Transfer/blob/5ec2d52553ec8f4e0fe77854bc2b31956ac09a11/password.lua#L23
  pub fn sub_bits_u32(&self, min: usize, max: usize) -> u32 {
    let mut acc = 0;
    for i in 0..=(max - min) {
      if min < self.bits.len() {
        acc = 2 * acc + u32::from(self.bits[min + i]);
      } else {
        acc *= 2;
      }
    }
    acc
  }
}
