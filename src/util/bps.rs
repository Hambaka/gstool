// Converted to Rust from byuu/near's reference implementation.

use std::fmt;

pub enum BpsError {
  NotABps,
}

impl fmt::Display for BpsError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::NotABps => write!(f, "提供的不是BPS补丁文件！"),
    }
  }
}

struct Bps {
  beat: Vec<u8>,
  beat_offset: usize,
  target: Vec<u8>,
}

impl Bps {
  fn new(patch: &[u8]) -> Self {
    Self {
      beat: Vec::from(patch),
      beat_offset: 0,
      target: Vec::new(),
    }
  }

  fn read(&mut self) -> u8 {
    let result = self.beat.get(self.beat_offset).unwrap();
    self.beat_offset += 1;
    *result
  }

  fn decode(&mut self) -> u64 {
    let (mut data, mut shift) = (0u64, 1u64);
    loop {
      let x = self.read();
      data += u64::from(x & 0x7F) * shift;
      if x & 0x80 != 0 {
        break;
      }
      shift <<= 7;
      data += shift;
    }
    data
  }

  fn skip(&mut self, size: usize) {
    self.beat_offset += size;
  }

  fn write(&mut self, data: u8) {
    self.target.push(data);
  }

  fn read_footer_checksum(&mut self) -> u32 {
    let mut v = 0u32;
    for shift in (0..32).step_by(8) {
      v |= (u32::from(self.read())) << shift;
    }
    v
  }
}

// BPS1
const BPS_HEADER: [u8; 4] = [0x42, 0x50, 0x53, 0x31];

pub fn apply_bps_patch(source: &[u8], beat: &[u8]) -> Result<(Vec<u8>, u32), BpsError> {
  let mut bps = Bps::new(beat);

  for ele in BPS_HEADER {
    if bps.read() != ele {
      // 理论上是unreachable!()
      return Err(BpsError::NotABps);
    }
  }

  let _source_size = bps.decode() as usize;
  let _target_size = bps.decode() as usize;

  let metadata_size = bps.decode() as usize;
  bps.skip(metadata_size);

  const SOURCE_READ: u32 = 0;
  const TARGET_READ: u32 = 1;
  const SOURCE_COPY: u32 = 2;
  const _TARGET_COPY: u32 = 3;

  let mut source_relative_offset: usize = 0;
  let mut target_relative_offset: usize = 0;

  while bps.beat_offset < beat.len() - 12 {
    let mut length = bps.decode();
    let mode = (length & 3) as u32;
    length = (length >> 2) + 1;

    if mode == SOURCE_READ {
      for _ in 0..length {
        bps.write(source[bps.target.len()]);
      }
    } else if mode == TARGET_READ {
      for _ in 0..length {
        let temp = bps.read();
        bps.write(temp);
      }
    } else {
      let mut offset = bps.decode() as isize;
      offset = if offset & 1 != 0 {
        -(offset >> 1)
      } else {
        offset >> 1
      };
      if mode == SOURCE_COPY {
        source_relative_offset = (source_relative_offset as isize + offset) as usize;
        for _ in 0..length {
          bps.write(source[source_relative_offset]);
          source_relative_offset += 1;
        }
      } else {
        target_relative_offset = (target_relative_offset as isize + offset) as usize;
        for _ in 0..length {
          bps.write(bps.target[target_relative_offset]);
          target_relative_offset += 1;
        }
      }
    }
  }

  let _source_hash = bps.read_footer_checksum();
  let target_hash = bps.read_footer_checksum();
  let _beat_hash = bps.read_footer_checksum();

  Ok((bps.target, target_hash))
}
