#![warn(clippy::all, clippy::pedantic)]

mod app;
mod components {
  pub mod buttons;
  pub mod colors;
}
mod core {
  pub mod convert;
  pub mod enums;
  pub mod output;
  pub mod save;
}
mod views {
  mod password_exporting {
    mod memory;
    mod save;
    mod shared_view;
    mod text;
  }
  mod name_customization;
  mod save_conversion;
}
pub use app::App;
