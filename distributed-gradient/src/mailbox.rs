use std::collections::HashMap;
use crate::message::Message;
use crate::state::States;

pub mod factory;

pub trait Mailbox {
    fn enqueue(&mut self, msg: Message);
    fn messages(&mut self) -> Messages;
}

pub type Messages = HashMap<i32, Message>;

pub trait AsStates {
    fn as_states(&self) -> States;
}

impl AsStates for Messages {
    fn as_states(&self) -> States {
        let mut states = States::new();
        for (id, msg) in self.iter() {
            states.insert(*id, msg.export.clone());
        }
        states
    }

}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    #[test]
    fn test_as_states() {
        use crate::message::Message;
        use crate::state::States;
        use crate::mailbox::AsStates;
        use rf_core::export::Export;
        use rf_core::path::Path;
        use std::any::Any;
        use rf_core::export;

        let mut messages = HashMap::new();
        let export_1 = export!((Path::new(), 1));
        let export_2 = export!((Path::new(), 2));
        let export_3 = export!((Path::new(), 3));
        messages.insert(1, Message::new(1, export_1.clone()));
        messages.insert(2, Message::new(2, export_2.clone()));
        messages.insert(3, Message::new(3, export_3.clone()));

        let states: States = messages.as_states();
        assert_eq!(states.len(), 3);
        assert_eq!(states.get(&1).unwrap(), &export_1);
        assert_eq!(states.get(&2).unwrap(), &export_2);
        assert_eq!(states.get(&3).unwrap(), &export_3);
    }
}