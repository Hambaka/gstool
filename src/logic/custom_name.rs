//! “自定义角色名”功能状态：名字输入、运行日志、存档载入 / 写入、toast 队列。
//!
//! ui侧只能经`request()`追加待办的request；
//! 唯一进行IO操作的地方是`logic()`与`run()`。
//!
//! 写存档的字节级操作在`core/save/name_write.rs`，本模块只负责
//! 读文件、识别版本、决定“哪些名字写进哪个槽位”、以及输出路径与提示。

use std::path::{Path, PathBuf};

use crate::{
  core::{
    enums::{GameRegion, GameTitle, NameType},
    save::{
      convert_save,
      format::{MAX_SAVE_FILE_SIZE, SaveLayout},
      name_write::write_custom_names,
      pc_name::{EncodedName, encode_pc_name},
      raw::{SaveSlots, parse_convertible_save, parse_save_slots},
    },
  },
  io::{
    read_bytes, render::name_encode_log, resolve_output_dir_single, suffixed_file_name,
    write_named_bytes,
  },
};

/// 主角团 7 位成员的默认中文名。
/// 元素个数和`PC_NAME_RECORD_COUNT[0]`是一致的
pub const PC_NAMES: [&str; 7] = [
  "罗宾",
  "杰拉德",
  "伊万",
  "梅雅莉",
  "加西亚",
  "加斯敏",
  "西芭",
];

/// toast 的类型（由`App::logic()`消费）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
  Success = 0,
  Warning = 1,
}

/// 一条待显示的 toast；`logic`侧只生产纯数据，由`ui`侧接收后显示
#[derive(Debug, Clone)]
pub struct ToastMessage {
  pub kind: ToastKind,
  pub text: String,
}

/// “自定义角色名”功能状态
#[derive(Debug, Default)]
pub struct CustomNameState {
  /// 7个输入框的自定义名字内容
  pub names: [String; 7],
  /// 最近一次运行时每个角色的编码结果（空名字为`None`）
  encoded: [Option<EncodedName>; 7],
  /// 最近一次运行时的名字快照；用来判断“运行后有没有再改名字”
  ran_names: Option<[String; 7]>,
  /// 已载入的存档文件路径（空 = 未载入）
  pub input_file_path: String,
  /// 已载入存档的原始字节（非日版在载入时已转换为新汉化版）
  save_bytes: Vec<u8>,
  /// 载入时是否做过“非日版 → 新汉化版”的自动转换
  pub auto_converted: bool,
  /// 已载入存档的槽位信息
  pub save_slots: SaveSlots,
  /// 1/2/3号槽位是否勾选
  pub selected_slots: [bool; 3],
  /// 指定输出目录（空 = 与输入文件同级）
  pub out_dir: String,
  /// 已暂存过至少一次名字
  pub has_stored_name: bool,
  /// 最近一次运行的时间戳（空 = 尚未运行）
  time: String,
  /// 最近一次运行的日志（空 = 尚未运行）
  log: String,
  /// 本页待办request（私有：ui侧使用`request()`追加）
  pending: Vec<CustomNameRequest>,
  /// 待显示的toast（私有：由`take_toasts()`交给`App`）
  pending_toasts: Vec<ToastMessage>,
}

/// 本页的待办request；
/// 只用本页自己的状态就能完成的动作
#[derive(Debug, Clone)]
pub enum CustomNameRequest {
  /// 读取并识别用户选中的存档文件
  LoadSave(PathBuf),
  /// 把名字暂存到准备输出的存档中
  StoreNames,
  /// 将存档输出到新文件
  WriteSave,
}

impl CustomNameState {
  /// ui侧使用，用于登记一条待办request，由下一帧的`logic()`处理
  pub fn request(&mut self, request: CustomNameRequest) {
    self.pending.push(request);
  }

  /// 本页是否可运行（至少有一个名字输入框填写了内容）
  pub fn can_run(&self) -> bool {
    self.names.iter().any(|name| !name.is_empty())
  }

  /// 是否有输出日志（判断是否可复制日志内容到剪贴板）
  pub fn has_output(&self) -> bool {
    !self.log.is_empty()
  }

  /// 是否运行过（不保证名字全部有效）
  pub fn has_run(&self) -> bool {
    self.ran_names.is_some()
  }

  /// 上次运行是否至少有一个名字，且所有非空名字都编码成功
  pub fn has_valid_run(&self) -> bool {
    self.encoded.iter().any(Option::is_some)
      && self
        .encoded
        .iter()
        .all(|entry| entry.as_ref().is_none_or(EncodedName::is_valid))
  }

  /// 上次运行之后名字是否又被改过
  pub fn names_changed_since_run(&self) -> bool {
    self
      .ran_names
      .as_ref()
      .is_none_or(|snapshot| snapshot != &self.names)
  }

  /// 是否已载入存档（有路径且至少一个槽位）
  pub fn has_save(&self) -> bool {
    !self.input_file_path.is_empty() && self.save_slots.slots.iter().any(Option::is_some)
  }

  /// 是否勾选了至少一个“确实存在”的槽位
  pub fn any_slot_selected(&self) -> bool {
    self
      .selected_slots
      .iter()
      .enumerate()
      .any(|(index, &selected)| selected && self.save_slots.slots[index].is_some())
  }

  /// 当前是否可以点【修改存档】
  pub fn can_store_names(&self) -> bool {
    self.has_valid_run()
      && !self.names_changed_since_run()
      && self.has_save()
      && self.any_slot_selected()
  }

  /// 时间戳文本
  pub fn time(&self) -> &str {
    &self.time
  }

  /// 日志文本
  pub fn log(&self) -> &str {
    &self.log
  }

  /// 重置本页的名字输入（纯内存改动，视图里直接调）
  pub fn reset_names(&mut self) {
    for name in &mut self.names {
      name.clear();
    }
  }

  /// 将本页的空名字填入默认名
  pub fn set_default_name(&mut self) {
    for (name, default_name) in self.names.iter_mut().zip(PC_NAMES) {
      if name.is_empty() {
        default_name.clone_into(name);
      }
    }
  }

  /// 重置已载入的存档（纯内存改动，视图里直接调）
  pub fn reset_save(&mut self) {
    self.input_file_path.clear();
    self.save_bytes.clear();
    self.save_slots = SaveSlots::default();
    self.selected_slots = [false; 3];
    self.auto_converted = false;
    self.has_stored_name = false;
  }

  /// 逻辑侧，处理上一帧ui侧提交的本功能的待办请求
  pub fn logic(&mut self) -> Result<(), String> {
    let pending = std::mem::take(&mut self.pending);
    for request in pending {
      match request {
        CustomNameRequest::LoadSave(path) => self.load_save(&path)?,
        CustomNameRequest::StoreNames => self.store_names()?,
        CustomNameRequest::WriteSave => self.write_save()?,
      }
    }

    Ok(())
  }

  /// 运行本功能（由`AppRequest::Run`调用）：编码所有非空名字并生成输入方式日志
  pub fn run(&mut self) -> Result<(), String> {
    let layout = GameTitle::TheBrokenSeal.save_layout();
    let mut encoded = std::array::from_fn(|_| None);

    for (index, name) in self.names.iter().enumerate() {
      if name.is_empty() {
        continue;
      }
      encoded[index] = Some(encode_pc_name(name));
    }

    let entries = encoded
      .iter()
      .enumerate()
      .filter_map(|(index, encoded_name)| {
        encoded_name.as_ref().map(|encoded_name| {
          (
            PC_NAMES[index],
            layout.pc_name_ram_address(index),
            encoded_name,
          )
        })
      })
      .collect::<Vec<_>>();

    self.time = crate::util::time::now();
    self.log = name_encode_log(&entries);
    self.encoded = encoded;
    self.ran_names = Some(self.names.clone());

    Ok(())
  }

  /// 取走待显示的toast；由`App::logic()`在`logic()`之后调用
  pub fn take_toasts(&mut self) -> Vec<ToastMessage> {
    std::mem::take(&mut self.pending_toasts)
  }

  /// 读取、识别并（必要时）转换用户选中的存档文件
  fn load_save(&mut self, path: &Path) -> Result<(), String> {
    let mut bytes =
      read_bytes(path, Some(MAX_SAVE_FILE_SIZE)).map_err(|error| error.to_string())?;
    let game_info = parse_convertible_save(&bytes).map_err(|error| error.to_string())?;

    if game_info.title == GameTitle::TheLostAge {
      return Err(
        "本功能仅支持《黄金太阳 开启的封印》的存档，无法修改《黄金太阳 失落的时代》的存档！"
          .to_owned(),
      );
    }

    // 非日版《开启的封印》存档：先强制转成新汉化版（默认名 + 日版构建日期），
    // 否则把中文名字节写进去后游戏可能显示异常。
    let auto_converted = game_info.region != GameRegion::Japan;
    if auto_converted {
      convert_save(
        &mut bytes,
        GameTitle::TheBrokenSeal,
        game_info.first_slot_index,
        Some(NameType::ChineseFanTranslation2023Team),
        Some(GameRegion::Japan),
      );
    }

    let save_slots = parse_save_slots(&bytes, true).map_err(|error| error.to_string())?;

    self.input_file_path = path.to_string_lossy().into_owned();
    self.save_bytes = bytes;
    self.save_slots = save_slots;
    self.selected_slots = [false; 3];
    self.auto_converted = auto_converted;
    self.has_stored_name = false;

    if auto_converted {
      self.push_toast(
        ToastKind::Warning,
        format!(
          "已自动转换{}版为新汉化版。输出文件时会生效，原文件不会被修改。",
          game_info.region.short_name()
        ),
      );
    }

    Ok(())
  }

  fn store_names(&mut self) -> Result<(), String> {
    let layout: SaveLayout = GameTitle::TheBrokenSeal.save_layout();
    let mut output_bytes = self.save_bytes.clone();
    let mut changed_slots = Vec::new();

    let entries = self.name_entries();

    for (slot_index, slot_option) in self.save_slots.slots.iter().enumerate() {
      if !self.selected_slots[slot_index] {
        continue;
      }
      let Some(slot) = slot_option else {
        continue;
      };

      let leader = self.leader_bytes(&layout);
      write_custom_names(
        &mut output_bytes,
        &layout,
        slot.section_index,
        &entries,
        leader,
      )
      .map_err(|error| error.to_string())?;
      changed_slots.push(slot_index + 1);
    }

    let slots_str = changed_slots
      .iter()
      .map(|x| x.to_string())
      .collect::<Vec<_>>()
      .join("、");
    self.save_bytes = output_bytes;
    self.has_stored_name = true;
    self.push_toast(
      ToastKind::Success,
      format!("已暂存名字！（槽位：{slots_str}）"),
    );

    Ok(())
  }

  /// 把暂存的数据输出为`原名_自定义名`的新存档
  fn write_save(&mut self) -> Result<(), String> {
    let output_bytes = self.save_bytes.clone();

    let input_path = Path::new(&self.input_file_path);
    let file_name = suffixed_file_name(input_path, "自定义名");
    let output_dir = resolve_output_dir_single(
      input_path,
      (!self.out_dir.is_empty()).then(|| Path::new(&self.out_dir)),
    );
    write_named_bytes(&output_dir, &file_name, &output_bytes).map_err(|error| error.to_string())?;

    self.push_toast(ToastKind::Success, format!("已生成：{file_name}"));

    Ok(())
  }

  /// 生成“要写入哪些角色”的列表
  fn name_entries(&self) -> Vec<(usize, &[u8])> {
    let mut entries = Vec::new();

    for (index, encoded) in self.encoded.iter().enumerate() {
      if let Some(encoded) = encoded {
        entries.push((index, encoded.all_bytes().as_slice()));
      }
    }

    entries
  }

  /// 队长名的字节：
  /// 如果“罗宾”被改了，需要额外同步存档界面的队长名
  fn leader_bytes(&self, layout: &SaveLayout) -> Option<&[u8]> {
    let leader_index = layout.main_leader_pc_index;

    if let Some(encoded) = &self.encoded[leader_index] {
      Some(encoded.all_bytes().as_slice())
    } else {
      None
    }
  }

  /// 登记一条待显示的toast
  fn push_toast(&mut self, kind: ToastKind, text: String) {
    self.pending_toasts.push(ToastMessage { kind, text });
  }
}
