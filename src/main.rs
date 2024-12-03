#![windows_subsystem = "windows"]

use std::borrow::Cow;
use klask::Settings;

mod args;
mod cmd;
mod util;

fn main() {
  let cli = args::build_cli();
  let mut settings = Settings::default();
  // 在此处定义嵌入程序的自定义字体，支持ttf或otf字体，例如：
  // settings.custom_font = Some(Cow::Borrowed(include_bytes!(r"font/字体.ttf")));
  settings.custom_font = Some(Cow::Borrowed(include_bytes!(r"font/unifont-16.0.02.otf")));
  settings.localization = args::zh_hans_localization();

  klask::run_app(cli, settings, |matches| {
    match matches.subcommand() {
      Some(("（1）密码导出功能", sub_matches)) => {
        match sub_matches.subcommand() {
          Some(("（1.1）使用存档文件", matches)) => cmd::sav::run(matches),
          Some(("（1.2）使用密码文本文件", matches)) => cmd::txt::run(matches),
          Some(("（1.3）使用内存转储文件", matches)) => cmd::dmp::run(matches),
          _ => unreachable!(),
        }
      }
      Some(("（2）存档转换功能", sub_matches)) => cmd::conv::run(sub_matches),
      Some(("（3）自定义角色名", sub_matches)) => cmd::rename::run(sub_matches),
      _ => unreachable!(),
    }
  });
}
