use rf_core::export::Export;
use serde::{Deserialize, Serialize};

pub mod message_queue;

/// This struct represent a message that will be sent between nodes.
#[derive(Debug, Clone, PartialEq,Serialize, Deserialize)]
pub struct Message {
    pub source: i32,
    pub export: Export,
}

impl Message {
    pub fn new(source: i32, p1: Export) -> Self {
        Self {
            source,
            export: p1,
        }
    }
}

#[cfg(test)]
mod test {
    use std::any::Any;
    use std::collections::HashMap;
    use rf_core::path::Path;
    use rf_core::export;
    use super::*;

    #[test]
    fn test_new() {
        let export = export!((Path::new(), 1));
        let msg = super::Message::new(1, export.clone());
        assert_eq!(msg.source, 1);
        assert_eq!(msg.export, export);
    }
}
