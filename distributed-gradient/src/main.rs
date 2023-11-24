use rf_core::context::Context;
use rf_core::sensor_id::{sensor, SensorId};
use rufi_gradient::gradient;
use rumqttc::MqttOptions;
use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;
use distributed_gradient::mailbox::factory::{MailboxFactory, ProcessingPolicy};
use distributed_gradient::network::factory::NetworkFactory;
use distributed_gradient::platform::Platform;

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
    let network = NetworkFactory::async_mqtt_network(mqttoptions, nbrs).await;

    // Setup the mailbox
    let mailbox = MailboxFactory::from_policy(ProcessingPolicy::MemoryLess);

    let context = Context::new(
        self_id,
        local_sensor.clone(),
        nbr_sensor.clone(),
        Default::default(),
    );

    // Setup the platform and run the program
    Platform::new(
        mailbox,
        network,
        context,
    ).run(gradient).await
}

