//! 由构建脚本在编译期生成的数据表（见 `build/gen_db.rs`）。
//!
//! 生成的内容：
//! - `Language`：四语言枚举（日/英/1代新汉化/2代汉化），用于ITEM_NAMES
//! - `PreviewObject`：带name(Language)的四语言名称结构
//! - `ITEM_NAMES: [PreviewObject; 460]`：460 个道具的四种语言/汉化版本的名字
//! - `DYRATI_ITEM_NAMES`：460 个道具的 Dyrati 表格专用的英文名
//! - `ALL_DJINN_NAMES：[[PreviewObject; 7]; 4]`：按游戏内主角团的默认队伍排列顺序来重新排列对应的元素顺序的精灵名
//! - `get_icon_id`：获取道具ID（同时处理同图标道具的ID），以获取对应的道具图标
include!(concat!(env!("OUT_DIR"), "/codegen_db.rs"));
