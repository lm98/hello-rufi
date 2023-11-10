use std::collections::VecDeque;
use crate::message::Message;
use crate::message::message_queue::{MessageQueue, MessageQueueIterator};

#[derive(Clone, Debug)]
pub struct MessageDeque {
    deque: VecDeque<Message>,
}

impl MessageDeque {
    pub fn new() -> Self {
        Self {
            deque: VecDeque::new(),
        }
    }
}

impl MessageQueue for MessageDeque {
    fn push(&mut self, msg: Message) {
        self.deque.push_back(msg);
    }

    fn pop(&mut self) -> Option<Message> {
        self.deque.pop_front()
    }

    fn empty(&self) -> bool {
        self.deque.is_empty()
    }
}

impl Iterator for MessageDeque {
    type Item = Message;

    fn next(&mut self) -> Option<Self::Item> {
        self.pop()
    }
}

impl MessageQueueIterator for MessageDeque {}

#[cfg(test)]
mod test {
    use std::any::Any;
    use std::collections::HashMap;
    use rf_core::{export, path};
    use rf_core::slot::Slot::{Nbr, Rep};
    use rf_core::export::Export;
    use rf_core::path::Path;
    use super::*;

    #[test]
    fn test_push_and_pop() {
        let export = export!((path!(), 2), (path!(Rep(0)), 2), (path!(Rep(0), Nbr(0)), 2));
        let msg = Message::new(1, export);
        let mut deque = MessageDeque::new();
        deque.push(msg.clone());
        assert_eq!(deque.pop(), Some(msg));
    }

    #[test]
    fn test_pop_removes_the_item() {
        let export = export!((path!(), 2), (path!(Rep(0)), 2), (path!(Rep(0), Nbr(0)), 2));
        let msg = Message::new(1, export);
        let mut deque = MessageDeque::new();
        deque.push(msg.clone());
        deque.pop();
        assert!(deque.empty());
    }
}