use egui::{Label, Ui, Widget};

use crate::{App, logic::command::SubCommandEnum, widgets::atoms::sub_command_atoms};

// 1.、2.、3.、4.
const DESC_C1: &str = r"使用该功能可导出密码，可导出以下几类文件：
1. 密码的文本文件（支持导出日文版/英文版密码）
2. 密码的内存转储文件
3. 密码的金手指文本文件，可以在《黄金太阳 失落的时代》中使用（支持日/美/德/西/法/意版本）
4. 密码数据的文本文件，可结合 Dyrati 的“Golden Sun Password Generator”表格使用

注：Mobile&elffinal/SOMA 的《黄金太阳 失落的时代》汉化版基于美版制作，因此金手指版本选择美版即可。";

impl App {
  pub fn show_c1(&mut self, ui: &mut Ui, svg_icon_size: f32, is_dark: bool) {
    Label::new(DESC_C1).selectable(false).ui(ui);
    ui.separator();
    ui.columns_const(|columns: &mut [egui::Ui; SubCommandEnum::ALL.len()]| {
      for i in 0..SubCommandEnum::ALL.len() {
        columns[i].selectable_value(
          &mut self.subcommand,
          SubCommandEnum::ALL[i],
          sub_command_atoms(SubCommandEnum::ALL[i], svg_icon_size, is_dark),
        );
      }
    });

    match self.subcommand {
      SubCommandEnum::First => self.show_c1s1(ui, svg_icon_size),
      SubCommandEnum::Second => self.show_c1s2(ui, svg_icon_size),
      SubCommandEnum::Third => self.show_c1s3(ui, svg_icon_size),
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
