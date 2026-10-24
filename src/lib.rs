#![warn(clippy::pedantic)]

mod app;
mod core {
  pub mod bits;
  pub mod data;
  pub mod enums;
  pub mod password;
  pub mod rom;
  pub mod save;
}
mod io;
mod logic;
mod util {
  pub mod bps;
  pub mod whitespace;
}
pub mod ui {
  pub mod central_panel {
    mod custom_names;
    pub mod inject_pwd;
    mod main_panel;
    mod password_export;
    mod patch_rom;
    mod save_convert;
  }
  pub mod window {
    pub mod about;
    pub mod pwd_info;
  }
  pub mod error_modal;
  pub mod page;
  pub mod top_panel;
  pub mod widgets {
    pub mod atoms;
    pub mod buttons;
    pub mod colors;
    pub mod icons;
    pub mod rows;
  }
}
pub use app::App;
