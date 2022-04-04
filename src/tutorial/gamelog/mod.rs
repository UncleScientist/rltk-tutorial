mod logstore;
use logstore::*;

mod builder;
pub use builder::*;

pub use logstore::{clear_log, log_display};

use rltk::RGB;

pub struct LogFragment {
    pub color: RGB,
    pub text: String,
}
