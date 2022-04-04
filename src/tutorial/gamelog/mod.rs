use serde::{Deserialize, Serialize};

mod logstore;
use logstore::*;

mod events;
pub use events::*;

mod builder;
pub use builder::*;

pub use logstore::{clear_log, clone_log, log_display, restore_log};

use rltk::RGB;

#[derive(Serialize, Deserialize, Clone)]
pub struct LogFragment {
    pub color: RGB,
    pub text: String,
}
