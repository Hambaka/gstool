pub mod convert;
pub mod format;
pub mod raw;

pub use convert::convert_save_file;
pub use raw::{GameInfo, SaveSlot, collect_slots, find_first_slot_index, identify_game};
