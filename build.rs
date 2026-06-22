use std::env;

use crate::build::{compress_font, gen_bin, gen_bps, gen_db, gen_name, gen_pwd};

mod build {
  pub mod compress_font;
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

  let is_release = env::var("PROFILE").map(|p| "release" == p).unwrap_or(false);
  let level = if is_release { 22 } else { 3 };

  compress_font::run(level);
  gen_bps::run(level);
  gen_db::run();
  gen_bin::run();
}
