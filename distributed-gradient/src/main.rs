use bytes::Bytes;
use distributed_gradient::message::Message;
use rf_core::context::Context;
use rf_core::export::Export;
use rf_core::lang::execution::round;
use rf_core::sensor_id::{sensor, SensorId};
use rf_core::vm::round_vm::RoundVM;
use rufi_gradient::gradient;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;
use tokio::sync::mpsc;
use distributed_gradient::mailbox::AsStates;
use distributed_gradient::mailbox::factory::{MailboxFactory, ProcessingPolicy};

// This enum represent the different command we will send between channels
#[derive(Debug)]
enum Command {
    Empty,
    Send { msg: String },
}

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

    // Setup the MQTT client
    let mut mqttoptions =
        MqttOptions::new(format!("device#{}", self_id), "test.mosquitto.org", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 20);

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

    // Spawn a task to handle the messages sent from neighbors
    let client_clone = client.clone();
    let (tx, mut rx) = mpsc::channel::<Command>(32);
    tokio::spawn(async move {
        subscriptions(client_clone, nbrs).await.unwrap();
        // poll the eventloop for new messages
        loop {
            if let Ok(notification) = eventloop.poll().await {
                if let rumqttc::Event::Incoming(rumqttc::Packet::Publish(msg)) = notification {
                    let msg_string = String::from_utf8(msg.payload.to_vec()).unwrap();
                    tx.send(Command::Send { msg: msg_string }).await.unwrap();
                }
            } else {
                tx.send(Command::Empty).await.unwrap();
            }
        }
    });

    let mut mailbox = MailboxFactory::from_policy(ProcessingPolicy::MostRecent);

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
        let msg = Message {
            source: self_id,
            export: self_export.clone(),
        };
        let msg_ser = serde_json::to_string(&msg).unwrap();
        client
            .publish(
                format!("hello-rufi/{self_id}/subscriptions"),
                QoS::AtMostOnce,
                false,
                Bytes::from(msg_ser),
            )
            .await?;

        //STEP 4: Receive the neighbouring exports from the message task
        if let Some(cmd) = rx.recv().await {
            match cmd {
                Command::Send { msg } => {
                    let msg: Message = serde_json::from_str(&msg).unwrap();
                    mailbox.enqueue(msg);
                }
                _ => {}
            }
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

async fn subscriptions(
    client: AsyncClient,
    nbrs: Vec<i32>,
) -> Result<(), Box<dyn std::error::Error>> {
    for nbr in nbrs {
        client
            .subscribe(format!("hello-rufi/{nbr}/subscriptions"), QoS::AtMostOnce)
            .await?;
    }
    Ok(())
}
