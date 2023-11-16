use distributed_gradient::message::Message;
use rf_core::context::Context;
use rf_core::export::Export;
use rf_core::lang::execution::round;
use rf_core::sensor_id::{sensor, SensorId};
use rf_core::vm::round_vm::RoundVM;
use rufi_gradient::gradient;
use rumqttc::MqttOptions;
use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;
use distributed_gradient::mailbox::AsStates;
use distributed_gradient::mailbox::factory::{MailboxFactory, ProcessingPolicy};
use distributed_gradient::network::NetworkUpdate;
use distributed_gradient::network::factory::NetworkFactory;

#[derive(Debug, Default)]
struct Arguments {
    pub id: i32,
    pub source: bool,
}

impl Arguments {
    pub fn parse<S: AsRef<str>>(args: impl IntoIterator<Item = S>) -> Result<Arguments, String> {
        let mut r = Arguments::default();

        for arg in args {
            match arg.as_ref() {
                "-t" => r.source = true,
                "-f" => r.source = false,
                x => r.id = x.parse::<i32>().unwrap(),
            }
        }

        Ok(r)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get arguments from the CLI
    let args = Arguments::parse(std::env::args().skip(1))?;
    let self_id = args.id;
    let is_source = args.source;

    /* Set up a simple topology that will be used for these tests.
     *  Topology: [1] -- [2] -- [3] -- [4] -- [5].
     */
    let nbrs: Vec<i32> = vec![self_id.clone() - 1, self_id.clone(), self_id.clone() + 1]
        .into_iter()
        .filter(|n| (n > &0 && n < &6))
        .collect();
    let local_sensor: HashMap<SensorId, Rc<Box<dyn Any>>> = vec![(
        sensor("source"),
        Rc::new(Box::new(is_source) as Box<dyn Any>),
    )]
    .into_iter()
    .collect();
    let nbr_sensor: HashMap<SensorId, HashMap<i32, Rc<Box<dyn Any>>>> = HashMap::from([(
        sensor("nbr_range"),
        nbrs.iter()
            .map(|n| {
                (
                    n.clone(),
                    Rc::new(Box::new(i32::abs(self_id - n)) as Box<dyn Any>),
                )
            })
            .collect(),
    )]);

    // Setup the MQTT client
    let mut mqttoptions =
        MqttOptions::new(format!("device#{}", self_id), "test.mosquitto.org", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    let mut network = NetworkFactory::async_mqtt_network(mqttoptions, nbrs).await;

    // Setup the mailbox
    let mut mailbox = MailboxFactory::from_policy(ProcessingPolicy::MemoryLess);

    loop {
        //STEP 1: Setup the aggregate program execution

        // Retrieve the neighbouring exports from the mailbox
        let states = mailbox.messages().as_states();

        //STEP 2: Execute a round
        let context = Context::new(
            self_id,
            local_sensor.clone(),
            nbr_sensor.clone(),
            states,
        );
        println!("CONTEXT: {:?}", context);
        let mut vm = RoundVM::new(context);
        vm.new_export_stack();
        let (mut vm_, result) = round(vm, gradient);
        let self_export: Export = vm_.export_data().clone();
        println!("OUTPUT: {}\nEXPORT: {}\n", result, self_export);

        //STEP 3: Publish the export
        let msg = Message::new(self_id, self_export, std::time::SystemTime::now());
        let msg_ser = serde_json::to_string(&msg).unwrap();
        network.send(self_id, msg_ser).await?;

        //STEP 4: Receive the neighbouring exports from the network
        if let Ok(update) = network.recv().await {
            match update {
                NetworkUpdate::Update { msg } => {
                    let msg: Message = serde_json::from_str(&msg).unwrap();
                    mailbox.enqueue(msg);
                }
                _ => {}
            }
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

