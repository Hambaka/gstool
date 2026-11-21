use jiff::{Unit, ZonedDateTime};

pub fn now() -> String {
  let now = ZonedDateTime::now().round(Unit::Millisecond).unwrap();
  // https://docs.rs/jiff/latest/jiff/fmt/strtime/index.html
  now.strftime("%Y/%m/%d %T%.3f").to_string()
}
