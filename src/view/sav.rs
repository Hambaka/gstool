use core::f32;
use std::{
  fs::{File, create_dir_all},
  io::{Read, Write},
  path::{Path, PathBuf},
};

use egui::{ComboBox, Label, Ui, Widget};
use rfd::FileDialog;

use crate::{
  App,
  core::{
    enums::{GameType, GameVersion, NameType},
    save::{convert_save_file, get_game_type_with_loop_start_index_option},
  },
};

impl App {
  pub fn show_sav(&mut self, ui: &mut Ui) {
    // 正式界面
    Label::new(
      "使用该功能可转换《黄金太阳 开启的封印》或《黄金太阳 失落的时代》存档的版本：\n\
      （1）修改主角团所有成员的名字为指定语言版本的默认名\n\
      （2）通过修改存档中的游戏构建日期来转换存档版本\n\n\
      注：\n\
      （1）如果存档中的构建日期与游戏 ROM 中的构建日期不匹配，游戏将强制玩家从神殿开始游戏。\n\
      （2）Mobile&Elffinal/SOMA 的《黄金太阳 失落的时代》汉化版基于美版制作，所以 ROM 中的构建日期同美版。\n\
      （3）《黄金太阳 开启的封印》新汉化版基于日版制作，所以 ROM 中的构建日期同日版。"
    ).selectable(false).ui(ui);

    egui::Grid::new("（2）存档转换功能").num_columns(2).spacing([40.0, 4.0]).striped(true).show(ui, |ui|{
      // 第1行 -> 输入文件路径  【选择文件……】【清空】
      Label::new("输入文件路径").selectable(false).ui(ui).on_hover_text("《黄金太阳 开启的封印》或《黄金太阳 失落的时代》的存档文件");
      ui.horizontal(|ui|{
        if ui.button("选择文件……").clicked(){
          if let Some(file) = FileDialog::new().pick_file(){
            self.save_file.clear();

            self.build_date_ver = None;
            self.default_name_ver = None;

            self.input_file_path_c2  = file.to_string_lossy().into_owned();
            // 验证存档文件
            let result = validate_save_conv(&self.input_file_path_c2.as_ref());
            match result {
                Ok(o) => {
                  self.save_file = o.0;
                  self.for_run_c2 = Some((o.1,o.2));
                },
                Err(e) => {
                  self.for_run_c2 = None;
                  self.err_msg = e.to_owned();
                  self.modal_open = true;
                },
            }
          }
        };
        if ui.button("清空").clicked(){
          self.input_file_path_c2.clear();
          self.save_file.clear();
          self.for_run_c2 = None;

          self.build_date_ver = None;
          self.default_name_ver = None;
        }
        ui.add(egui::TextEdit::singleline(&mut self.input_file_path_c2).interactive(false).hint_text("存档文件路径").desired_width(f32::INFINITY));
      });
      ui.end_row();

      // 第2行 -> 名字版本
      Label::new("名字版本").selectable(false).ui(ui).on_hover_text("若选择了值，则将存档中的主角团所有成员的名字转换至指定版本的默认名；若留空则不修改。");
      ComboBox::from_id_salt("名字版本").width(100.0).selected_text(match &self.default_name_ver {
          Some(name_type) => {name_type.get_name()},
          None => {"无"},
      }).show_ui(ui, |ui|{
        ui.selectable_value(&mut self.default_name_ver, None, "无");
        if let Some(val_result) = self.for_run_c2 {
            ui.selectable_value(&mut self.default_name_ver, Some(NameType::Japanese), NameType::Japanese.get_name());
            ui.selectable_value(&mut self.default_name_ver, Some(NameType::English), NameType::English.get_name());
            ui.selectable_value(&mut self.default_name_ver, Some(NameType::German), NameType::German.get_name());
            ui.selectable_value(&mut self.default_name_ver, Some(NameType::Spanish), NameType::Spanish.get_name());
            ui.selectable_value(&mut self.default_name_ver, Some(NameType::French), NameType::French.get_name());
            ui.selectable_value(&mut self.default_name_ver, Some(NameType::Italian), NameType::Italian.get_name());
            match val_result.0{
              GameType::TheBrokenSeal => {
                ui.selectable_value(&mut self.default_name_ver, Some(NameType::ChineseFanTranslation2023Team), NameType::ChineseFanTranslation2023Team.get_name());
              },
              GameType::TheLostAge => {
                ui.selectable_value(&mut self.default_name_ver, Some(NameType::ChineseFanTranslationMobileTeam), NameType::ChineseFanTranslationMobileTeam.get_name());
              },
          }
        }
      });
      ui.end_row();

      // 第3行 -> 构建日期版本
      Label::new("构建日期版本").selectable(false).ui(ui).on_hover_text("若选择了值，则将存档中的构建日期转换至指定版本的构建日期；若留空则不修改。");
      ComboBox::from_id_salt("构建日期版本").width(100.0).height(800.0).selected_text(match &self.build_date_ver{
        Some(version_enum) => {version_enum.get_name()},
        None => {"无"},
      }).show_ui(ui, |ui|{
        ui.selectable_value(&mut self.build_date_ver, None, "无");
        if self.for_run_c2.is_some(){
          ui.selectable_value(&mut self.build_date_ver, Some(GameVersion::Japan), GameVersion::Japan.get_name());
          ui.selectable_value(&mut self.build_date_ver, Some(GameVersion::USA), GameVersion::USA.get_name());
          ui.selectable_value(&mut self.build_date_ver, Some(GameVersion::Germany), GameVersion::Germany.get_name());
          ui.selectable_value(&mut self.build_date_ver, Some(GameVersion::Spain), GameVersion::Spain.get_name());
          ui.selectable_value(&mut self.build_date_ver, Some(GameVersion::France), GameVersion::France.get_name());
          ui.selectable_value(&mut self.build_date_ver, Some(GameVersion::Italy), GameVersion::Italy.get_name());
        }
      });
      ui.end_row();

      // 第4行 -> 指定输出文件保存目录  【选择目录……】【清空】
      Label::new("指定输出文件保存目录").selectable(false).ui(ui).on_hover_text("自定义输出的存档文件路径（推荐留空），若留空则将转换后的存档文件保存到输入文件的同目录下。");
      ui.horizontal(|ui|{
          if ui.button("选择目录……").clicked(){
            if let Some(file) = FileDialog::new().pick_folder(){
              self.out_dir_c2 = file.to_string_lossy().into_owned();
            }
          };
          if ui.button("清空").clicked(){
            self.out_dir_c2.clear();
          }
        ui.add(egui::TextEdit::singleline(&mut self.out_dir_c2).interactive(false).hint_text("存档文件输出路径").desired_width(f32::INFINITY));
      });
      ui.end_row();
    });
  }

  pub fn run_save(&mut self) {
    // Convert save data.
    convert_save_file(
      &mut self.save_file,
      Some(self.for_run_c2.unwrap().0),
      self.for_run_c2.unwrap().1,
      self.default_name_ver,
      self.build_date_ver,
    );

    // Start to create and write output save file.
    let output_path;
    let output_file;
    let input_path = Path::new(&self.input_file_path_c2);
    let file_stem = input_path.file_stem().unwrap().to_str().unwrap();
    let file_extension = if input_path.extension().is_none() {
      ""
    } else {
      input_path.extension().unwrap().to_str().unwrap()
    };
    let mut file_name_str = String::new();
    file_name_str.push_str(file_stem);
    file_name_str.push_str("_转换");

    if !file_extension.is_empty() {
      file_name_str.push('.');
      file_name_str.push_str(file_extension);
    }

    if self.out_dir_c2.is_empty() {
      let parent = input_path.parent().unwrap().to_str().unwrap();
      output_path = PathBuf::from(parent).join(file_name_str);
    } else {
      if create_dir_all(self.out_dir_c2.clone()).is_err() {
        self.err_msg = "创建目录失败！".to_owned();
        self.modal_open = true;
        return;
      }
      output_path = PathBuf::from(self.out_dir_c2.clone()).join(file_name_str);
    }

    output_file = File::create(output_path.clone());
    match output_file {
      Ok(mut file) => {
        if file.write_all(&self.save_file).is_err() {
          self.err_msg = format!("创建 “{}” 失败！", output_path.to_str().unwrap());
          self.modal_open = true;
          return;
        }
      }
      Err(_) => {
        self.err_msg = format!("创建 “{}” 失败！", output_path.to_str().unwrap());
        self.modal_open = true;
        return;
      }
    }

    // More detailed output logs, to improve user experience.
    // Sh*t code, but at least it works.
    self.out_log_c2.clear();
    self
      .out_log_c2
      .push_str(format!("输出文件：\n{}\n\n", output_path.to_str().unwrap()).as_str());
    self.out_log_c2.push_str("运行概况：\n");

    let mut result_index = 0;
    result_index += 1;
    let game_type_cn = match self.for_run_c2.unwrap().0 {
      GameType::TheBrokenSeal => "《黄金太阳 开启的封印》",
      GameType::TheLostAge => "《黄金太阳 失落的时代》",
    };
    self
      .out_log_c2
      .push_str(format!("{result_index}. 输入的源文件为{game_type_cn}的存档文件。\n").as_str());

    if let Some(name_type) = self.default_name_ver {
      result_index += 1;
      let name_type_cn = name_type.get_name();
      self.out_log_c2.push_str(
        format!(
          "{result_index}. 生成的存档文件已将主角团成员的名字转换为{name_type_cn}版的默认名。\n"
        )
        .as_str(),
      );

      if let Some(build_date_type) = self.build_date_ver {
        result_index += 1;
        let build_date_type_cn = build_date_type.get_name();
        self.out_log_c2.push_str(
          format!("{result_index}. 生成的存档文件已将构建日期转换为{build_date_type_cn}版。\n")
            .as_str(),
        );
      }
    }
  }
}

fn validate_save_conv(path_str: &str) -> Result<(Vec<u8>, GameType, usize), &'static str> {
  // Read save file.
  let input_file_result = File::open(path_str);
  let mut sav_file;
  if input_file_result.is_err() {
    return Err("打开存档文件时发生错误！");
  } else {
    sav_file = input_file_result.unwrap();
  }

  /* Check the size of save file.
  The size of save file should be 64KB,
  though the .SaveRAM file created by Bizhawk is 128KB.
  The size of the .srm file created by the libretro fork of VBA-M, VBA-Next and
  the .4gs file created by 44VBA is 136KB.
  Although the size of those files is over 64KB, seems it only use first 64KB space to store save data? */
  let file_size = sav_file.metadata().unwrap().len();
  if file_size != 0x10000 && file_size != 0x20000 && file_size != 0x22000 {
    return Err("存档文件的大小应为 64/128/136 KB！");
  }

  // Get raw save.
  let mut raw_save_file = Vec::new();
  sav_file.read_to_end(&mut raw_save_file).unwrap();

  // Detect game/save type, also get loop start index.
  let (game_type_option, loop_start_index_option) =
    get_game_type_with_loop_start_index_option(&raw_save_file);
  if game_type_option.is_none() {
    return Err(
      "输入的不是《黄金太阳 开启的封印》或《黄金太阳 失落的时代》的存档文件！或者存档文件中没有数据！",
    );
  }

  Ok((
    raw_save_file,
    game_type_option.unwrap(),
    loop_start_index_option.unwrap(),
  ))
}
