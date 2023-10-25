use std::collections::HashMap;
use rufi_core::core::context::Context;
use rufi_core::core::export::Export;
use rufi_core::core::lang::execution::round;
use crate::network::Network;
use rufi_core::core::vm::round_vm::RoundVM;
use crate::message::Message;

pub fn run<A: Copy+ 'static, F>(network: impl Network, self_id: i32, program: F)
    where
        F: Fn(RoundVM) -> (RoundVM, A) + Copy
{
    loop {
        let messages = network.receive();
        let exports: HashMap<i32, Export> = messages.iter().map(|(id, message)| (*id, message.to_export())).collect();
        let context = Context::new(self_id, Default::default(), Default::default(), exports);
        let vm = RoundVM::new(context);
        let (vm, _result) = round(vm, program);
        let self_export: Export = vm.context.exports.get(&self_id).unwrap().clone();
        network.send(self_id, Message::from_export(self_export));
    }
}