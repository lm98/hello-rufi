use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;
use std::time::Duration;
use bytes::Bytes;
use rf_core::context::Context;
use rf_core::export::Export;
use rf_core::lang::execution::round;
use rf_core::sensor_id::{sensor, SensorId};
use rf_core::vm::round_vm::RoundVM;
use rufi_gradient::gradient;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use distributed_gradient::message::Message;

#[tokio::main]
async fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    let self_id: i32 = i32::from_str(&args[1]).unwrap();

    // Setup the MQTT client
    let mut mqttoptions = MqttOptions::new(format!("device#{}", self_id), "test.mosquitto.org", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 20);
    let mut states: HashMap<i32, Export> = HashMap::new();

    /* Set up a simple topology that will be used for these tests.
    *  Topology: [1] -- [2] -- [3] -- [4] -- [5].
    */
    let nbrs: Vec<i32> = vec![self_id.clone()-1, self_id.clone(), self_id.clone()+1].into_iter().filter(|n| (n > &0 && n < &6)).collect();
    client.subscribe(format!("hello-rufi/1/subscriptions"), QoS::AtMostOnce).await.unwrap();
    client.subscribe(format!("hello-rufi/2/subscriptions"), QoS::AtMostOnce).await.unwrap();
    let local_sensor: HashMap<SensorId, Rc<Box<dyn Any>>> = vec![(sensor("source"), Rc::new(Box::new(self_id == 2) as Box<dyn Any>))].into_iter().collect();
    let nbr_sensor: HashMap<SensorId, HashMap<i32, Rc<Box<dyn Any>>>> = HashMap::from([(sensor("nbr_range"), nbrs.iter().map(|n| (n.clone(), Rc::new(Box::new(i32::abs(self_id - n)) as Box<dyn Any>))).collect())]);


    loop {
        // poll the eventloop for new messages
        let notification = eventloop.poll().await.unwrap();
        if let rumqttc::Event::Incoming(rumqttc::Packet::Publish(msg)) = notification {
            let msg_string = String::from_utf8(msg.payload.to_vec()).unwrap();
            let msg: Message = serde_json::from_str(&msg_string).unwrap();
            println!("MSG: {:?}", msg);
            states.insert(msg.source, msg.export);
        }
        let context = Context::new(self_id, local_sensor.clone(), nbr_sensor.clone(), states.clone());
        println!("CONTEXT: {:?}", context);
        let mut vm = RoundVM::new(context);
        vm.export_stack.push(Export::new());
        let (mut vm, result) = round(vm, gradient);
        let self_export: Export = vm.export_data().clone();
        states.insert(self_id, self_export.clone());
        println!("OUTPUT: {}\nEXPORT: {}\n", result, self_export);
        let msg = Message {  source: self_id, export: self_export.clone() };
        let msg_ser = serde_json::to_string(&msg).unwrap();
        client.publish(format!("hello-rufi/{self_id}/subscriptions"), QoS::AtMostOnce, false, Bytes::from(msg_ser)).await.unwrap();
    }
}
