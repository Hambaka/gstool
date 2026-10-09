pub mod convert;
pub mod format;
pub mod raw;

pub use convert::convert_save;
pub use raw::{GameInfo, SaveSlot, collect_slots, find_first_slot_index, identify_game};
