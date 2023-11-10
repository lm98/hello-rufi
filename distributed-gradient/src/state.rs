use std::collections::HashMap;
use rf_core::export::Export;
use crate::message::message_queue::MessageQueueIterator;

pub mod states_manager;

/// This type alias represent the states of the device inside an aggregate computation.
pub type States = HashMap<i32, Export>;

/// Creates a [States] from a [MessageQueueIterator].
/// This trait is needed in order to implement a functionality for a type alias that aliases a type
/// outside of this crate.
trait FromMessageQueue<T: MessageQueueIterator> {
    fn from_messages(queue: T) -> Self;
}

impl<T: MessageQueueIterator> FromMessageQueue<T> for States {
    fn from_messages(queue: T) -> Self {
        let mut states = States::new();

        for msg in queue {
            let source = msg.source;
            let export = msg.export;

            states.insert(source, export);
        }

        states
    }
}

#[cfg(test)]
mod test {
    use std::any::Any;
    use rf_core::{export, path};
    use super::*;
    use crate::message::message_queue::deque::MessageDeque;
    use crate::message::Message;
    use rf_core::export::Export;
    use rf_core::path::Path;
    use rf_core::slot::Slot::{Rep, Nbr};
    use crate::message::message_queue::MessageQueue;

    #[test]
    fn test_from_messages() {
        let mut queue = MessageDeque::new();
        queue.push(Message::new(1, export!((Path::new(), 1),(path!(Rep(0)), 1), (path!(Rep(0), Nbr(0)), 1))));
        queue.push(Message::new(2, export!((Path::new(), 2),(path!(Rep(0)), 2), (path!(Rep(0), Nbr(0)), 2))));
        queue.push(Message::new(3, export!((Path::new(), 3),(path!(Rep(0)), 3), (path!(Rep(0), Nbr(0)), 3))));

        let states = States::from_messages(queue);

        let mut expected = States::new();
        expected.insert(1, export!((Path::new(), 1),(path!(Rep(0)), 1), (path!(Rep(0), Nbr(0)), 1)));
        expected.insert(2, export!((Path::new(), 2),(path!(Rep(0)), 2), (path!(Rep(0), Nbr(0)), 2)));
        expected.insert(3, export!((Path::new(), 3),(path!(Rep(0)), 3), (path!(Rep(0), Nbr(0)), 3)));

        assert_eq!(states, expected)
    }
}