use crate::core::{
  data::{ALL_DJINN_NAMES, DYRATI_ITEM_NAMES, Language},
  enums::PasswordGrade,
  password::{
    PasswordData, PasswordInfo, bits_from_bytes, info_from_data, info_from_raw_save,
    items::STACKABLE_ITEMS,
    read_from_bits,
    stats::{DEFAULT_CLASS_MULTIPLIERS, MAX_VALID_IN_GAME_STATS_VALUES},
  },
};

use std::fmt::Write as _;

fn render(info: &PasswordInfo) -> String {
  let mut result = String::with_capacity(8192);

  // https://doc.rust-lang.org/rust-by-example/std/str.html
  // Text start
  let _ = writeln!(
    result,
    r"用于 Dyrati 的“Golden Sun Password Generator”表格的密码数据文本文件
来源出处：https://www.reddit.com/r/GoldenSun/comments/jon3h7/golden_sun_password_tools/
在线版表格：https://docs.google.com/spreadsheets/d/1jQ2Zj2F57Fb4hs0pDYLCaZL-qry7gcVxNbhpq0BJDUs/
对于 Windows 系统用户，推荐直接使用系统自带的记事本打开该文本文件。
复制“===开始===”和“===结束===”之间的所有内容，并粘贴到表格对应的单元格范围。



等级
罗宾，杰拉德，伊万，梅雅莉
表格中的范围 -> C8:F8
============开始============"
  );

  // Levels
  for (i, level) in info.levels.iter().enumerate() {
    let ch = if i == info.levels.len() - 1 {
      '\n'
    } else {
      '\t'
    };

    let _ = write!(result, "{level}{ch}");
  }

  writeln!(
    result,
    r"============结束============



精灵
维纳斯（地/金星），玛尔斯（火/火星），朱庇特（风/木星），墨丘利（水/水星）
表格中的范围 -> C12:F18
====================开始===================="
  )
  .unwrap();

  // Djinn
  for i in 0..7 {
    for j in 0..4 {
      let djinn_name = if info.djinn[j][i] {
        ALL_DJINN_NAMES[j][i].get_name(Language::English)
      } else {
        ""
      };

      let ch = if j == 3 { '\n' } else { '\t' };

      let _ = write!(result, "{djinn_name}{ch}");
    }
  }

  let _ = writeln!(
    result,
    r"====================结束====================



剧情事件
从上至下的顺序为：
（1）营救哈梅特，（2）竞技大赛获胜（最终决战获胜），（3）帮忙救助乌鲁木齐，（4）打败塔罗斯（宝岛最终Boss），（5）重返海迪亚（获悉罗宾母亲生病），（6）重返库普阿普（获悉小偷越狱逃跑）。

注：在 Paulygon 开发的“黄金太阳密码生成器”中，“帮忙救助乌鲁木齐”（事件 #3）的值与表格中的相反，
　　即在“黄金太阳密码生成器”中，勾选了“事件 #3”反而表示未完成该事件，而不勾选“事件 #3”才表示完成了该事件，不过这与游戏使用的实际数值对应。
　　表格版密码生成器为了方便理解，与其他事件进行了同步，即用1表示完成了该事件，用0表示未完成该事件。

表格中的范围 -> E21:E26
====开始===="
  );

  // Events
  for event in info.events {
    let _ = writeln!(result, "{}", if event { "1" } else { "0" });
  }

  writeln!(result, r#"====结束====



基础能力状态属性
罗宾，杰拉德，伊万，梅雅莉
从上至下的顺序为：
HP, EP, 攻击, 防御, 敏捷, 幸运

注：“基础能力状态属性”即"Base stats"，是不装备任何物品，没有附身任何精灵，也没有乘上角色默认职业的各属性倍率进行修正的角色能力属性值。
　　每个角色都拥有默认的基础职业，并且在游戏中基础属性值会乘上职业对应的属性倍率,
　　也就是说，这些是角色最本质的基础值，是无法在游戏里查看确认到的。
　　如果要使用“基础能力状态属性”来生成密码，记得把 J3 单元格选为"Base stats"。
　　（“基础能力状态属性”使用起来可能不太方便，不过不会出现数值误差）

表格中的范围 -> J5:M10
============开始============"#).unwrap();

  // Base Stats
  // Stats cap in dyrati sheet.
  let mut dyrati_base_stats = info.base_stats;
  let mut dyrati_stats = info.stats;
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
    for j in 0..4 {
      let stats = dyrati_base_stats[j][i];
      let ch = if j == 3 { '\n' } else { '\t' };
      let _ = write!(result, "{stats}{ch}");
    }
  }

  let _ = writeln!(
    result,
    r#"============结束============



能力状态属性
罗宾，杰拉德，伊万，梅雅莉
从上至下的顺序为：
HP, EP, 攻击, 防御, 敏捷, 幸运

注：“能力状态属性”即"Stats"，是不装备任何物品，没有附身任何精灵，但是有乘上角色默认职业的各属性倍率进行修正的角色能力属性值。
　　换句话说，这些属性值是可以在游戏里亲自确认到的，只要卸下所有装备，待命所有精灵就能看到。
　　如果要使用“能力状态属性”来生成密码，记得把 J3 单元格选为"Stats"。
　　（使用起来方便，但是在一些情况下会产生极其微小的数值误差，导致密码发生变化。这是因为在游戏中，几个不同的值有可能在经过计算后得到相同的结果，所以表格在逆向除以职业属性倍率获得真实的基础属性值时会假定该数值为多个可能的数值中的最大值。如果你非常在意这个误差问题的话，请使用“基础能力状态属性”吧）

表格中的范围 -> J5:M10
============开始============"#
  );

  // Stats, there may be some problems in this.
  for i in 0..6 {
    for j in 0..4 {
      let stats = dyrati_stats[j][i];
      let ch = if j == 3 { '\n' } else { '\t' };
      let _ = write!(result, "{stats}{ch}");
    }
  }

  let _ = writeln!(
    result,
    r"============结束============



物品
罗宾，杰拉德，伊万，梅雅莉
表格中的范围 -> J14:M28
========================================开始========================================"
  );

  // Items
  for i in 0..15 {
    for j in 0..4 {
      let (item_id, item_count) = info.items[j][i];

      let item_name = if item_id != 0 {
        let item_index = (item_id - 1) as usize;
        DYRATI_ITEM_NAMES[item_index]
      } else {
        ""
      };

      let count_str = if STACKABLE_ITEMS.contains(&item_id) {
        &format!("*{item_count}")
      } else {
        ""
      };

      let ch = if j == 3 { '\n' } else { '\t' };

      let _ = write!(result, "{item_name}{count_str}{ch}");
    }
  }

  let _ = writeln!(
    result,
    r"========================================结束========================================



金币
表格中的范围 -> D28
====开始===="
  );

  // Coins
  let _ = writeln!(result, "{}\n====结束====", info.coins);

  result
}

pub fn sheet_from_save_slot(raw_save: &[u8]) -> String {
  let info = info_from_raw_save(raw_save);
  render(&info)
}

pub fn sheet_from_password_data(data: &PasswordData) -> String {
  let info = info_from_data(data);
  render(&info)
}

pub fn sheet_from_password_bytes(password_bytes: &[u8], target_grade: PasswordGrade) -> String {
  let mut bits = bits_from_bytes(password_bytes, false).unwrap();
  let data = read_from_bits(&mut bits, target_grade);
  let info = info_from_data(&data);
  render(&info)
}
