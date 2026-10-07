use crate::core::{
  data::{ALL_DJINN_NAMES, DYRATI_ITEM_NAMES, Language},
  enums::PasswordGrade,
  password::{
    decode::{bits_from_bytes, info_from_data, info_from_raw_save, read_from_bits},
    stats::{DEFAULT_CLASS_MULTIPLIERS, MAX_VALID_IN_GAME_STATS_VALUES},
    structs::{PasswordData, PasswordInfo},
  },
  save_game::format::STACKABLE_ITEMS,
};

use std::fmt::Write as _;

fn render(pwd_info: &PasswordInfo) -> String {
  let mut exported_text = String::new();
  // Text start
  exported_text.push_str("用于 Dyrati 的“Golden Sun Password Generator”表格的密码数据文本文件\n");
  exported_text.push_str(
    "来源出处：https://www.reddit.com/r/GoldenSun/comments/jon3h7/golden_sun_password_tools/\n",
  );
  exported_text.push_str("在线版表格：https://docs.google.com/spreadsheets/d/1jQ2Zj2F57Fb4hs0pDYLCaZL-qry7gcVxNbhpq0BJDUs/\n");
  exported_text.push_str("对于 Windows 系统用户，推荐直接使用系统自带的记事本打开该文本文件。\n");
  exported_text.push_str(
    "复制“===开始===”和“===结束===”之间的所有内容，并粘贴到表格对应的单元格范围。\n\n\n\n",
  );

  // Levels
  exported_text.push_str("等级\n罗宾，杰拉德，伊万，梅雅莉\n表格中的范围 -> C8:F8\n");
  exported_text.push_str("============开始============\n");
  for (i, level) in pwd_info.levels.iter().enumerate() {
    if i < pwd_info.levels.len() - 1 {
      let _ = write!(exported_text, "{level}\t");
    } else {
      let _ = write!(exported_text, "{level}\n");
    }
  }
  exported_text.push_str("============结束============\n\n\n\n");

  // Djinn
  exported_text.push_str("精灵\n维纳斯（地/金星），玛尔斯（火/火星），朱庇特（风/木星），墨丘利（水/水星）\n表格中的范围 -> C12:F18\n");
  exported_text.push_str("====================开始====================\n");

  for i in 0..7 {
    for j in 0..4 {
      if pwd_info.djinn[0][i] {
        exported_text.push_str(ALL_DJINN_NAMES[j][i].get_name(Language::English));
      }
      if j == 3 {
        exported_text.push('\n');
      } else {
        exported_text.push('\t');
      }
    }
  }
  exported_text.push_str("====================结束====================\n\n\n\n");

  // Events
  exported_text.push_str("剧情事件\n从上至下的顺序为：\n（1）营救哈梅特，（2）竞技大赛获胜（最终决战获胜），（3）帮忙救助乌鲁木齐，（4）打败塔罗斯（宝岛最终Boss），（5）重返海迪亚（获悉罗宾母亲生病），（6）重返库普阿普（获悉小偷越狱逃跑）。\n\n");
  exported_text.push_str("注：在 Paulygon 开发的“黄金太阳密码生成器”中，“帮忙救助乌鲁木齐”（事件 #3）的值与表格中的相反，\n");
  exported_text.push_str(
    "　　即在“黄金太阳密码生成器”中，勾选了“事件 #3”反而表示未完成该事件，而不勾选“事件 #3”才表示完成了该事件，不过这与游戏使用的实际数值对应。\n",
  );
  exported_text.push_str("　　表格版密码生成器为了方便理解，与其他事件进行了同步，即用1表示完成了该事件，用0表示未完成该事件。\n\n");
  exported_text.push_str("表格中的范围 -> E21:E26\n");
  exported_text.push_str("====开始====\n");
  for event in pwd_info.events {
    let _ = writeln!(exported_text, "{}", if event { "1" } else { "0" });
  }
  exported_text.push_str("====结束====\n\n\n\n");

  // Base Stats
  exported_text.push_str("基础能力状态属性\n罗宾，杰拉德，伊万，梅雅莉\n");
  exported_text.push_str("从上至下的顺序为：\nHP, EP, 攻击, 防御, 敏捷, 幸运\n\n");
  exported_text.push_str(
    "注：“基础能力状态属性”即\"Base stats\"，是不装备任何物品，没有附身任何精灵，也没有乘上角色默认职业的各属性倍率进行修正的角色能力属性值。\n",
  );
  exported_text.push_str(
    "　　每个角色都拥有默认的基础职业，并且在游戏中基础属性值会乘上职业对应的属性倍率,\n",
  );
  exported_text.push_str("　　也就是说，这些是角色最本质的基础值，是无法在游戏里查看确认到的。\n");
  exported_text
    .push_str("　　如果要使用“基础能力状态属性”来生成密码，记得把 J3 单元格选为\"Base stats\"。\n");
  exported_text
    .push_str("　　（“基础能力状态属性”使用起来可能不太方便，不过不会出现数值误差）\n\n");
  exported_text.push_str("表格中的范围 -> J5:M10\n");
  exported_text.push_str("============开始============\n");

  // Stats cap in dyrati sheet.
  let mut dyrati_base_stats = pwd_info.base_stats;
  let mut dyrati_stats = pwd_info.stats;
  for i in 0..6 {
    for j in 0..4 {
      if dyrati_base_stats[j][i] > MAX_VALID_IN_GAME_STATS_VALUES[i] {
        dyrati_base_stats[j][i] = MAX_VALID_IN_GAME_STATS_VALUES[i];
        dyrati_stats[j][i] = (MAX_VALID_IN_GAME_STATS_VALUES[i] as f64
          * DEFAULT_CLASS_MULTIPLIERS[j][i])
          .floor() as u16;
      }
    }
  }

  for i in 0..6 {
    let _ = write!(exported_text, "{}\t", dyrati_base_stats[0][i]);
    let _ = write!(exported_text, "{}\t", dyrati_base_stats[1][i]);
    let _ = write!(exported_text, "{}\t", dyrati_base_stats[2][i]);
    let _ = writeln!(exported_text, "{}", dyrati_base_stats[3][i]);
  }
  exported_text.push_str("============结束============\n\n\n\n");

  // Stats, there may be some problems in this.
  exported_text.push_str("能力状态属性\n罗宾，杰拉德，伊万，梅雅莉\n");
  exported_text.push_str("从上至下的顺序为：\nHP, EP, 攻击, 防御, 敏捷, 幸运\n\n");
  exported_text
    .push_str("注：“能力状态属性”即\"Stats\"，是不装备任何物品，没有附身任何精灵，但是有乘上角色默认职业的各属性倍率进行修正的角色能力属性值。\n");
  exported_text.push_str("　　换句话说，这些属性值是可以在游戏里亲自确认到的，只要卸下所有装备，待命所有精灵就能看到。\n");
  exported_text
    .push_str("　　如果要使用“能力状态属性”来生成密码，记得把 J3 单元格选为\"Stats\"。\n");
  exported_text
    .push_str("　　（使用起来方便，但是在一些情况下会产生极其微小的数值误差，导致密码发生变化。");
  exported_text.push_str("这是因为在游戏中，几个不同的值有可能在经过计算后得到相同的结果，所以表格在逆向除以职业属性倍率获得真实的基础属性值时会假定该数值为多个可能的数值中的最大值。");
  exported_text.push_str("如果你非常在意这个误差问题的话，请使用“基础能力状态属性”吧）\n\n");
  exported_text.push_str("表格中的范围 -> J5:M10\n");
  exported_text.push_str("============开始============\n");
  for i in 0..6 {
    let _ = write!(exported_text, "{}\t", dyrati_stats[0][i]);
    let _ = write!(exported_text, "{}\t", dyrati_stats[1][i]);
    let _ = write!(exported_text, "{}\t", dyrati_stats[2][i]);
    let _ = writeln!(exported_text, "{}", dyrati_stats[3][i]);
  }
  exported_text.push_str("============结束============\n\n\n\n");

  // Items
  exported_text.push_str("物品\n罗宾，杰拉德，伊万，梅雅莉\n表格中的范围 -> J14:M28\n");
  exported_text.push_str(
    "========================================开始========================================\n",
  );
  for i in 0..15 {
    let (isaac_item_id, item_count) = pwd_info.items[0][i];
    if isaac_item_id != 0 {
      let isaac_item_index = (isaac_item_id - 1) as usize;
      exported_text.push_str(DYRATI_ITEM_NAMES[isaac_item_index]);
      if STACKABLE_ITEMS.contains(&isaac_item_id) {
        let _ = write!(exported_text, "*{}", item_count);
      }
    }
    exported_text.push('\t');

    let (garet_item_id, item_count) = pwd_info.items[1][i];
    if garet_item_id != 0 {
      let garet_item_index = (garet_item_id - 1) as usize;
      exported_text.push_str(DYRATI_ITEM_NAMES[garet_item_index]);
      if STACKABLE_ITEMS.contains(&garet_item_id) {
        let _ = write!(exported_text, "*{}", item_count);
      }
    }
    exported_text.push('\t');

    let (ivan_item_id, item_count) = pwd_info.items[2][i];
    if ivan_item_id != 0 {
      let ivan_item_index = (ivan_item_id - 1) as usize;
      exported_text.push_str(DYRATI_ITEM_NAMES[ivan_item_index]);
      if STACKABLE_ITEMS.contains(&ivan_item_id) {
        let _ = write!(exported_text, "*{}", item_count);
      }
    }
    exported_text.push('\t');

    let (mia_item_id, item_count) = pwd_info.items[3][i];
    if mia_item_id != 0 {
      let mia_item_index = (mia_item_id - 1) as usize;
      exported_text.push_str(DYRATI_ITEM_NAMES[mia_item_index]);
      if STACKABLE_ITEMS.contains(&mia_item_id) {
        let _ = write!(exported_text, "*{}", item_count);
      }
    }
    exported_text.push('\n');
  }
  exported_text.push_str(
    "========================================结束========================================\n\n\n\n",
  );
  // Coins
  exported_text.push_str("金币\n表格中的范围 -> D28\n");
  exported_text.push_str("====开始====\n");
  let _ = writeln!(exported_text, "{}", pwd_info.coins);
  exported_text.push_str("====结束====\n");

  exported_text
}

pub fn from_raw_save(raw_save: &[u8]) -> String {
  let pwd_info = info_from_raw_save(raw_save);
  render(&pwd_info)
}

pub fn from_data(data: &PasswordData) -> String {
  let pwd_info = info_from_data(data);
  render(&pwd_info)
}

pub fn from_password_bytes(password_bytes: &[u8], target_grade: PasswordGrade) -> String {
  let mut password_bits = bits_from_bytes(password_bytes, false).unwrap();
  let save_data = read_from_bits(&mut password_bits, target_grade);
  let pwd_info = info_from_data(&save_data);
  render(&pwd_info)
}
