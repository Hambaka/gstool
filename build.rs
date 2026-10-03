use std::env;

use crate::build::{gen_bin, gen_bps, gen_db, gen_name, gen_pwd};

mod build {
  pub mod gen_bin;
  pub mod gen_bps;
  pub mod gen_db;
  pub mod gen_name;
  pub mod gen_pwd;
}

pub const MAIN_DIR: &str = env!("CARGO_MANIFEST_DIR");

// https://doc.rust-lang.org/cargo/reference/environment-variables.html
fn main() {
  gen_pwd::run();
  gen_name::run();

  /*
  1, 3, 7, 11

  Use ZSTD compression levels - Amazon Athena
  https://docs.aws.amazon.com/athena/latest/ug/compression-support-zstd-levels.html
  https://docs.aws.amazon.com/zh_cn/athena/latest/ug/compression-support-zstd-levels.html

  ------------

  ZSTD Decompression Speed vs Compression Level #4091
  https://github.com/facebook/zstd/issues/4091

  VipulGarg869 opened on Jul 5, 2024
  "...The documentation states that the decompression speed doesn't change much with the compression level.
  While that is roughly the case, I was still able to find that the decompression speed decreases up till compression level 6.
  Beyond level 6, it increases. What is the reason for this?"

  Cyan4973 on Jul 5, 2024 · edited by Cyan4973
  "A variation in the number of sequences produced.
  Best decompression speed is typically achieved at level 15,
  though some sources may prefer level 1."
  */
  // Compress with zstd and write.
  let is_release = env::var("PROFILE").map(|p| "release" == p).unwrap_or(false);
  let level = if is_release { 22 } else { 3 };

  gen_bps::run(level);
  gen_db::run();
  gen_bin::run();
}
