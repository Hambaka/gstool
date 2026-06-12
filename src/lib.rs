#![warn(clippy::pedantic)]

mod app;
mod widgets {
  pub mod buttons;
  pub mod colors;
}
mod core {
  pub mod convert;
  pub mod data;
  pub mod enums;
  pub mod output;
  pub mod save;
}
mod util {
  pub mod bps;
}
mod ui {
  mod central_panel {
    mod password_export {
      mod memory;
      mod save;
      mod shared_view;
      mod text;
    }
    mod custom_names;
    mod main_panel;
    mod patch_rom;
    mod save_convert;
  }
  pub mod window {
    mod about;
    pub mod pwd_info;
  }
  mod error_modal;
  mod top_panel;
}
pub use app::App;
