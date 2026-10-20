//! 状态 + 工作流程层：承载每个功能的状态与共享数据。
//!
//! 即界面的状态
//! （选了哪个文件、勾了哪些选项、载入了哪些密码数据）与消费该状态的工作流程；
//! 而UI侧只负责画并报告动作，具体的工作流程在这各模块。
pub mod command;
pub mod custom_name;
pub mod inject_pwd;
pub mod password_export;
pub mod patch_rom;
pub mod preview;
pub mod save_convert;
pub mod save_slot;
pub mod sources;
