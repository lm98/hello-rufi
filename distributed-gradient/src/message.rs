use rf_core::export::Export;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub source: i32,
    pub export: Export,
}