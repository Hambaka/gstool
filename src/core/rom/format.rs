//! GBA的《黄金太阳》系列游戏ROM的格式常量（两代共用）。

/// GBA的内存映射基址：文件的偏移 + 此值 = 运行期地址
pub const ROM_MEMORY_MAPPING: u32 = 0x0800_0000;
/// GBA游戏ROM大小上限：32MB
pub const MAX_ROM_SIZE: u64 = 0x200_0000;
/// GBA游戏ROM头部的“游戏标题”字段的偏移
pub const SOFTWARE_TITLE_OFFSET: usize = 0xA0;
/// GBA游戏ROM头部的“游戏标题”字段的长度
pub const SOFTWARE_TITLE_LEN: usize = 12;
/// 《失落的时代》日版的标题
pub const SOFTWARE_TITLE_TLA_JAPAN: &str = "OUGONTAIYO_B";
/// 《失落的时代》美/德/西/法/意版德标题（也包含汉化版）
pub const SOFTWARE_TITLE_TLA_WORLD: &str = "GOLDEN_SUN_B";
/// ROM内记录的《失落的时代》的构建日期字符串的偏移\
/// 用于区分美/德/西/法/意版本
pub const ROM_BUILD_DATE_OFFSET: usize = 0x0068_2000;
/// ROM内记录的《失落的时代》的构建日期字符串的长度\
/// 用于区分美/德/西/法/意版本
pub const ROM_BUILD_DATE_LEN: usize = 12;
/// 自动输入密码程序的入口所使用的地址字段的偏移（注入后指向新增的程序）
pub const TARGET_ADDRESS_OFFSET: usize = 0x38304;
/// 注入块的自定义头`GSPI`
pub const INJECT_HEADER: [u8; 4] = *b"GSPI";
/// 注入块的头部大小：`GSPI`(4) + 组合编号(4) + 空白(8)
pub const INJECT_BLOCK_HEADER_SIZE: usize = 0x10;
/// 注入的程序本体占用的空间上限（金+银+铜的最大组合，并16字节对齐）
pub const INJECT_PAYLOAD_MAX_SIZE: usize = 0x370;
/// 注入块总占用：0x10 + 0x370 = 0x380
pub const INJECT_BLOCK_MAX_SIZE: usize = INJECT_BLOCK_HEADER_SIZE + INJECT_PAYLOAD_MAX_SIZE;
/// 注入偏移按16字节对齐
pub const INJECT_ALIGNMENT: usize = 0x10;
