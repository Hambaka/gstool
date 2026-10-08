#![warn(clippy::pedantic)]

mod app;
mod widgets {
  pub mod buttons;
  pub mod colors;
  pub mod icons;
}
mod core {
  pub mod bits;
  pub mod convert;
  pub mod data;
  pub mod enums;
  pub mod password;
  pub mod save;
}
mod io;
mod util {
  pub mod bps;
}
pub mod ui {
  pub mod central_panel {
    mod password_export {
      mod memory;
      mod save;
      mod shared_view;
      mod text;
    }
    mod custom_names;
    pub mod inject_pwd;
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
