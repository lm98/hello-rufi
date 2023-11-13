use std::collections::HashMap;
use crate::mailbox::{Mailbox, Messages};
use crate::message::Message;

pub enum ProcessingPolicy {
    MostRecent,
    LeastRecent,
}

pub struct MailboxFactory;

impl MailboxFactory {
    pub fn from_policy(policy: ProcessingPolicy) -> Box<dyn Mailbox> {
        match policy {
            ProcessingPolicy::MostRecent => Box::new(HashMapMailbox { messages: HashMap::new() }),
            ProcessingPolicy::LeastRecent => Box::new(MultiMapMailbox { messages: HashMap::new() }),
        }
    }
}

struct HashMapMailbox {
    messages: HashMap<i32, Message>,
}

impl Mailbox for HashMapMailbox {
    fn enqueue(&mut self, msg: Message) {
        self.messages.insert(msg.source, msg);
    }

    fn messages(&mut self) -> Messages {
        self.messages.clone()
    }
}

struct MultiMapMailbox {
    messages: HashMap<i32, Vec<Message>>,
}

impl Mailbox for MultiMapMailbox {
    fn enqueue(&mut self, msg: Message) {
        let entry = self.messages.entry(msg.source).or_insert(Vec::new());
        entry.push(msg);
    }

    fn messages(&mut self) -> Messages {
        let mut messages = HashMap::new();
        for (source, msgs) in self.messages.iter_mut() {
            let msg = msgs.pop().unwrap();
            messages.insert(*source, msg);
        }
        messages
    }
}