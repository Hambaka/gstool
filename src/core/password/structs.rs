/// 用于生成密码的密码数据
#[derive(Debug)]
pub struct PasswordData {
  /// 角色等级数据
  pub levels: [u8; 4],
  /// 精灵获取信息数据
  pub djinn: [u32; 4],
  /// 事件完成数据
  pub events: [u8; 6],
  /// 角色状态属性数据
  pub stats: [[u16; 6]; 4],
  /// 道具数据
  pub items: [[u16; 15]; 4],
  /// 金币数量数据
  pub coins: u32,
}

/// `由PasswordData而来`
/// 用于密码信息展示，方便玩家阅读
#[derive(Debug, Clone)]
pub struct PasswordView {
  /// 角色等级数据
  pub levels: [u8; 4],
  /// 精灵获取信息数据
  ///
  /// `与PasswordData的不同点`：
  /// 按照游戏的排列顺序进行重新排序，以符合玩家习惯（适配Dyrati标准）
  /// 并改为bool类型的二维数组以方便信息展示
  pub djinn: [[bool; 7]; 4],
  /// 事件完成数据
  ///
  /// `与PasswordData的不同点`：
  /// 将其中的搭救乌鲁木齐的事件的值进行取反，以符合玩家对事件是否已完成的直觉判断（适配Dyrati标准）
  /// 并改为bool类型的一维数组以方便展示
  pub events: [bool; 6],
  /// 角色状态属性数据
  ///
  /// `和PasswordData中的stats相同`
  /// 都是游戏内无法确认到的基础属性数据
  pub base_stats: [[u16; 6]; 4],
  /// 游戏内实际展示的角色状态属性数据
  ///
  /// 角色卸掉全部装备、待命全部精灵后即是此值
  pub stats: [[u16; 6]; 4],
  /// 精神力道具获取信息数据
  ///
  /// 银、铜密码只记录精神力道具的获取状态
  /// 单独记录以方便对应的密码级别进行预览
  pub psy_items: [bool; 8],
  /// 道具数据
  ///
  /// `与PasswordData的不同点`：
  /// 额外添加了道具数量以方便信息展示
  pub items: [[(u16, u8); 15]; 4],
  /// 金币数量数据
  pub coins: u32,
}
