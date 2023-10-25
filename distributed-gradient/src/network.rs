use std::collections::HashMap;
use crate::message::Message;

mod implements;

pub trait Network {
    fn send(&self, self_id: i32, data: Message);
    fn receive(&self) -> HashMap<i32, Message>;
}