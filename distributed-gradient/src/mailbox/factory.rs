use std::collections::{BTreeMap, HashMap};
use std::time::SystemTime;
use crate::mailbox::{Mailbox, Messages};
use crate::message::Message;

/// This enum represent the different processing policies for the mailbox.
pub enum ProcessingPolicy {
    /// Only the last message received is kept. This policy, from the user's viewpoint, acts similarly to
    /// the [MostRecent] version, but it is more memory efficient since the other messages received are substituted
    /// with the last received.
    MemoryLess,
    /// Keeps every message received from each neighbor, but returns only the most recent one.
    MostRecent,
    /// Keeps every message received from each neighbor, but returns only the least recent one. This policy is
    /// useful if the user wants to make sure every message is processed.
    LeastRecent,
}

/// This struct is used as a factory for [Mailbox]es.
pub struct MailboxFactory;

impl MailboxFactory {
    /// Creates a new [Mailbox] with the given [ProcessingPolicy].
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