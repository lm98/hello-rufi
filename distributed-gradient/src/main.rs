use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;
use tokio::sync::mpsc;
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

#[derive(Debug)]
enum Command {
    Get,
    Send {
        msg: String,
    }
}

#[tokio::main]
async fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    let self_id: i32 = i32::from_str(&args[1]).unwrap();

    // Setup the MQTT client
    let mut mqttoptions = MqttOptions::new(format!("device#{}", self_id), "test.mosquitto.org", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 20);
    /* Set up a simple topology that will be used for these tests.
    *  Topology: [1] -- [2] -- [3] -- [4] -- [5].
    */
    let nbrs: Vec<i32> = vec![self_id.clone()-1, self_id.clone(), self_id.clone()+1].into_iter().filter(|n| (n > &0 && n < &6)).collect();
    let local_sensor: HashMap<SensorId, Rc<Box<dyn Any>>> = vec![(sensor("source"), Rc::new(Box::new(self_id == 2) as Box<dyn Any>))].into_iter().collect();
    let nbr_sensor: HashMap<SensorId, HashMap<i32, Rc<Box<dyn Any>>>> = HashMap::from([(sensor("nbr_range"), nbrs.iter().map(|n| (n.clone(), Rc::new(Box::new(i32::abs(self_id - n)) as Box<dyn Any>))).collect())]);

    let client_clone = client.clone();
    let (tx, mut rx) = mpsc::channel::<Command>(32);

    // Spawn a task to handle the messages sent from neighbors
    tokio::spawn(async move {
        subscriptions(client_clone, nbrs).await;
        // poll the eventloop for new messages
        loop {
            let notification = eventloop.poll().await.unwrap();
            if let rumqttc::Event::Incoming(rumqttc::Packet::Publish(msg)) = notification {
                let msg_string = String::from_utf8(msg.payload.to_vec()).unwrap();
                tx.send(Command::Send { msg: msg_string }).await.unwrap();
            }
        }
    });

    let mut states: HashMap<i32, Export> = HashMap::new();
    loop {
        //Execute a round
        let context = Context::new(self_id, local_sensor.clone(), nbr_sensor.clone(), states.clone());
        println!("CONTEXT: {:?}", context);
        let mut vm = RoundVM::new(context);
        vm.export_stack.push(Export::new());
        let (mut vm_, result) = round(vm, gradient);
        let self_export: Export = vm_.export_data().clone();
        states.insert(self_id, self_export.clone());
        println!("OUTPUT: {}\nEXPORT: {}\n", result, self_export);
        // Publish the export
        let msg = Message {  source: self_id, export: self_export.clone() };
        let msg_ser = serde_json::to_string(&msg).unwrap();
        client.publish(format!("hello-rufi/{self_id}/subscriptions"), QoS::AtMostOnce, false, Bytes::from(msg_ser)).await.unwrap();
        // Receive the export from the message task
        if let Some(message) = rx.recv().await {
            if let Command::Send { msg } = message {
                let msg: Message = serde_json::from_str(&msg).unwrap();
                states.insert(msg.source, msg.export);
            }
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

async fn subscriptions(client: AsyncClient, nbrs: Vec<i32>) {
    for nbr in nbrs {
        client.subscribe(format!("hello-rufi/{nbr}/subscriptions"), QoS::AtMostOnce).await.unwrap();
        println!("Subscribed to: {}", nbr)
    }
}