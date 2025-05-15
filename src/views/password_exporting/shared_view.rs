use egui::{Label, Ui, Widget};

use crate::{
  App,
  app::CommandEnum,
  core::enums::{PasswordGrade, PasswordVersion},
};

pub const PASSWORD_GRADES: [PasswordGrade; 3] = [
  PasswordGrade::Gold,
  PasswordGrade::Silver,
  PasswordGrade::Bronze,
];
pub const PASSWORD_VERSIONS: [PasswordVersion; 2] =
  [PasswordVersion::Japanese, PasswordVersion::English];

const DESC_C1: &str = r#"使用该功能可导出密码，可导出以下几类文件：
（1）密码的文本文件（支持导出日文版/英文版密码）
（2）密码的内存转储文件
（3）密码的金手指文本文件，可以在《黄金太阳 失落的时代》中使用（支持日/美/德/西/法/意版本）
（4）密码数据的文本文件，可结合 Dyrati 的“Golden Sun Password Generator”表格使用

注：Mobile&Elffinal/SOMA 的《黄金太阳 失落的时代》汉化版基于美版制作，因此金手指版本选择美版即可。"#;

impl App {
  pub fn show_c1(&mut self, ui: &mut Ui) {
    Label::new(DESC_C1).selectable(false).ui(ui);
    ui.separator();
    ui.columns(3, |columns| {
      columns[0].selectable_value(
        &mut self.subcommand,
        CommandEnum::First,
        "（1.1）使用存档文件",
      );
      columns[1].selectable_value(
        &mut self.subcommand,
        CommandEnum::Second,
        "（1.2）使用密码文本文件",
      );
      columns[2].selectable_value(
        &mut self.subcommand,
        CommandEnum::Third,
        "（1.3）使用内存转储文件",
      );
    });

    match self.subcommand {
      CommandEnum::First => self.show_c1s1(ui),
      CommandEnum::Second => self.show_c1s2(ui),
      CommandEnum::Third => self.show_c1s3(ui),
    }
  }

  pub fn run_c1(&mut self) {
    match self.subcommand {
      CommandEnum::First => self.run_c1s1(),
      CommandEnum::Second => self.run_c1s2(),
      CommandEnum::Third => self.run_c1s3(),
    }
  }

  pub fn show_log_c1(&mut self, ui: &mut Ui) {
    match self.subcommand {
      CommandEnum::First => Label::new(&self.out_log_c1s1).selectable(false).ui(ui),
      CommandEnum::Second => Label::new(&self.out_log_c1s2).selectable(false).ui(ui),
      CommandEnum::Third => Label::new(&self.out_log_c1s3).selectable(false).ui(ui),
    };
  }
}
