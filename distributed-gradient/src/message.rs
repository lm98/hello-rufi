use rf_core::export::Export;
use serde::{Deserialize, Serialize};

/// This struct represent a message that will be sent between nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub source: i32,
    pub export: Export,
}
