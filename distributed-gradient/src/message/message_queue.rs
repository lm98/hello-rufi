use crate::message::Message;

pub mod deque;

/// This trait represent a message queue.
pub trait MessageQueue {
    /// Push a message to the queue.
    ///
    /// # Arguments
    ///
    /// * `msg` - The message to push.
    fn push(&mut self, msg: Message);

    /// Pop a message from the queue.
    ///
    /// # Returns
    ///
    /// The message that was popped.
    fn pop(&mut self) -> Option<Message>;

    /// Check if the queue is empty.
    ///
    /// # Returns
    ///
    /// True if the queue is empty, false otherwise.
    fn empty(&self) -> bool;
}

/// This trait represent a message queue that can be iterated.
pub trait MessageQueueIterator: MessageQueue + Iterator<Item = Message> {}