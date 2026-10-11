//! 移植自Dyrati的`Golden-Sun-Password-Transfer`中的lua脚本，该脚本适用于`vba-rr`和`Bizhawk`模拟器\
//! https://github.com/Dyrati/Golden-Sun-Password-Transfer/blob/40ef45e4a77b2cf17a0e325f9f0f41817ef21140/password.lua\
//! 即15行到32行之间的所有内容
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone)]
pub struct BitVec {
  bits: Vec<u8>,
}

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

// 实现Deref后即可直接使用.len()、.is_empty()等
impl Deref for BitVec {
  type Target = [u8];

  fn deref(&self) -> &Self::Target {
    &self.bits
  }
}

impl DerefMut for BitVec {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.bits
  }
}

impl BitVec {
  pub fn new() -> Self {
    BitVec { bits: Vec::new() }
  }

  pub fn with_capacity(capacity: usize) -> Self {
    BitVec {
      bits: Vec::with_capacity(capacity),
    }
  }

  pub fn as_slice(&self) -> &[u8] {
    self
  }

  pub fn remove(&mut self, index: usize) {
    self.bits.remove(index);
  }

  pub fn bit_at(&self, index: usize) -> u8 {
    self[index]
  }

  pub fn push_bit(&mut self, value: u32) {
    self.bits.push((value & 1) as u8);
  }

  /// 视一个整数值为左高位右低位的二进制数，将每一个二进制位的值依次写入/推到内部bits中
  pub fn push_bits(&mut self, value: u32, size: usize) {
    for i in 0..size {
      self.bits.push(((value >> (size - i - 1)) & 1) as u8);
    }
  }

  /// 视一个整数值为左高位右低位的二进制数，将每一个二进制位的值依次替换掉内部bits中指定下标位置的值
  pub fn replace_bits(&mut self, value: u32, size: usize, pos: usize) {
    for i in 0..size {
      self[pos + i] = ((value >> (size - i - 1)) & 1) as u8;
    }
  }

  /// 从内部的位数组中，读出指定下标范围内的位，并将其拼成完整的u8值取出
  pub fn sub_bits_u8(&self, min: usize, max: usize) -> u8 {
    let mut acc = 0;
    for i in 0..=(max - min) {
      acc = 2 * acc + self.get(min + i).copied().unwrap_or(0);
    }

    acc
  }

  /// 从内部的位数组中，读出指定下标范围内的位，并将其拼成完整的u16值取出
  pub fn sub_bits_u16(&self, min: usize, max: usize) -> u16 {
    let mut acc = 0;
    for i in 0..=(max - min) {
      acc = 2 * acc + u16::from(self.get(min + i).copied().unwrap_or(0));
    }

    acc
  }

  /// 从内部的位数组中，读出指定下标范围内的位，并将其拼成完整的u32值取出
  pub fn sub_bits_u32(&self, min: usize, max: usize) -> u32 {
    let mut acc = 0;
    for i in 0..=(max - min) {
      acc = 2 * acc + u32::from(self.get(min + i).copied().unwrap_or(0));
    }

    acc
  }
}
