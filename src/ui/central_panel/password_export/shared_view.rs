use egui::{Label, Ui, Widget};

use crate::{App, app::SubCommandEnum};

const DESC_C1: &str = r"使用该功能可导出密码，可导出以下几类文件：
⒈ 密码的文本文件（支持导出日文版/英文版密码）
⒉ 密码的内存转储文件
⒊ 密码的金手指文本文件，可以在《黄金太阳 失落的时代》中使用（支持日/美/德/西/法/意版本）
⒋ 密码数据的文本文件，可结合 Dyrati 的“Golden Sun Password Generator”表格使用

注：Mobile&elffinal/SOMA 的《黄金太阳 失落的时代》汉化版基于美版制作，因此金手指版本选择美版即可。";

impl App {
  pub fn show_c1(&mut self, ui: &mut Ui) {
    Label::new(DESC_C1).selectable(false).ui(ui);
    ui.separator();
    ui.columns_const(
      |columns: &mut [egui::Ui; SubCommandEnum::SUB_COMMANDS.len()]| {
        for i in 0..SubCommandEnum::SUB_COMMANDS.len() {
          columns[i].selectable_value(
            &mut self.subcommand,
            SubCommandEnum::SUB_COMMANDS[i],
            SubCommandEnum::SUB_COMMANDS[i].get_name(),
          );
        }
      },
    );

    match self.subcommand {
      SubCommandEnum::First => self.show_c1s1(ui),
      SubCommandEnum::Second => self.show_c1s2(ui),
      SubCommandEnum::Third => self.show_c1s3(ui),
    }
  }

  pub fn run_c1(&mut self) {
    match self.subcommand {
      SubCommandEnum::First => self.run_c1s1(),
      SubCommandEnum::Second => self.run_c1s2(),
      SubCommandEnum::Third => self.run_c1s3(),
    }
  }

  pub fn show_log_c1(&mut self, ui: &mut Ui) {
    match self.subcommand {
      SubCommandEnum::First => Label::new(&self.out_log_c1s1).selectable(false).ui(ui),
      SubCommandEnum::Second => Label::new(&self.out_log_c1s2).selectable(false).ui(ui),
      SubCommandEnum::Third => Label::new(&self.out_log_c1s3).selectable(false).ui(ui),
    };
  }
}
