#![warn(clippy::pedantic)]

mod app;
mod widgets {
  pub mod buttons;
  pub mod colors;
}
mod core {
  pub mod convert;
  pub mod enums;
  pub mod output;
  pub mod save;
}
mod util {
  pub mod bps;
}
mod ui {
  mod password_export {
    mod memory;
    mod save;
    mod shared_view;
    mod text;
  }
  mod custom_names;
  mod patch_rom;
  mod save_convert;
}
pub use app::App;
