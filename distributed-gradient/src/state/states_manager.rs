use crate::message::message_queue::MessageQueueIterator;
use crate::state::{FromMessageQueue, States};

/// This enum represent the different policies that can be used to process incoming messages.
pub enum MessageProcessingPolicy {
    /// Process one message at a time.
    OneAtATime,
    /// Process all the messages at once.
    AllAtOnce,
}

pub struct StatesManager<T: MessageQueueIterator> {
    msg_queue: T,
    policy: MessageProcessingPolicy,
}

impl<T> StatesManager<T>
    where
        T: MessageQueueIterator + Clone
{
    pub fn new(msg_queue: T) -> Self {
        Self {
            msg_queue,
            policy: MessageProcessingPolicy::OneAtATime,
        }
    }

    /// Set the policy to use to process incoming messages.
    ///
    /// # Arguments
    ///
    /// * `policy` - The [MessageProcessingPolicy] to use.
    ///
    /// # Returns
    ///
    /// A reference to self.
    pub fn with_policy(mut self, policy: MessageProcessingPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Enqueue a message.
    ///
    /// # Arguments
    ///
    /// * `msg` - The message to enqueue.
    pub fn enqueue(&mut self, msg: T::Item) {
        self.msg_queue.push(msg);
    }

    /// Process the next set of states.
    ///
    /// # Returns
    ///
    /// The next set of states to be used inside a round execution. The result of this operation
    /// depends on the [MessageProcessingPolicy] that is being used.
    pub fn next_set_of_states(&mut self) -> States {
        match self.policy {
            MessageProcessingPolicy::OneAtATime => {
                let mut states = States::new();
                if let Some(msg) = self.msg_queue.pop() {
                    let source = msg.source;
                    let export = msg.export;
                    states.insert(source, export);
                };
                states
            },
            MessageProcessingPolicy::AllAtOnce => {
                States::from_messages(self.msg_queue.clone())
            }
        }
    }
}