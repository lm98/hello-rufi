use std::collections::{BTreeMap, HashMap};
use std::time::SystemTime;
use crate::mailbox::{Mailbox, Messages};
use crate::message::Message;

pub enum ProcessingPolicy {
    MemoryLess,
    MostRecent,
    LeastRecent,
}

pub struct MailboxFactory;

impl MailboxFactory {
    pub fn from_policy(policy: ProcessingPolicy) -> Box<dyn Mailbox> {
        match policy {
            ProcessingPolicy::MemoryLess => Box::new(MemoryLessMailbox { messages: HashMap::new() }),
            ProcessingPolicy::MostRecent => Box::new(TimeOrderedMailbox { messages: HashMap::new(), pop_first: false }),
            ProcessingPolicy::LeastRecent => Box::new(TimeOrderedMailbox { messages: HashMap::new(), pop_first: true }),
        }
    }
}

struct MemoryLessMailbox {
    messages: HashMap<i32, Message>,
}

impl Mailbox for MemoryLessMailbox {
    fn enqueue(&mut self, msg: Message) {
        self.messages.insert(msg.source, msg);
    }

    fn messages(&mut self) -> Messages {
        self.messages.clone()
    }
}

struct TimeOrderedMailbox {
    messages: HashMap<i32, BTreeMap<SystemTime, Message>>,
    pop_first: bool,
}

impl Mailbox for TimeOrderedMailbox {
    fn enqueue(&mut self, msg: Message) {
        let msgs = self.messages.entry(msg.source).or_insert(BTreeMap::new());
        msgs.insert(msg.timestamp, msg);
    }

    fn messages(&mut self) -> Messages {
        let mut messages = HashMap::new();
        for (id, msgs) in self.messages.iter_mut() {
            if self.pop_first {
                //get the first entry of the BTreeMap
                if let Some((_, msg)) = msgs.pop_first() {
                    messages.insert(*id, msg.clone());
                }
            } else {
                //get the last entry of the BTreeMap
                if let Some((_, msg)) = msgs.pop_last() {
                    messages.insert(*id, msg.clone());
                }
            }
        }
        messages
    }
}