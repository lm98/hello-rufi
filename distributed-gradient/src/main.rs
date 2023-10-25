use std::str::FromStr;
use std::time::Duration;
use bytes::Bytes;
use rumqttc::{AsyncClient, MqttOptions, QoS};

#[tokio::main]
async fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    let self_id: i32 = i32::from_str(&args[1]).unwrap();

    // Setup the MQTT client
    let mut mqttoptions = MqttOptions::new(format!("device#{}", self_id), "test.mosquitto.org", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 20);

    client.subscribe("hello-rufi/gradient/start", QoS::AtMostOnce).await.unwrap();

    client.publish_bytes("hello-rufi/gradient/subscriptions", QoS::AtMostOnce, false, Bytes::from(self_id.to_string())).await.unwrap();
    loop {
        let notification = eventloop.poll().await.unwrap();
        match notification {
            rumqttc::Event::Incoming(rumqttc::Incoming::Publish(p)) => {
                println!("Received = {:?}", p.payload);
            }
            _ => {}
        }
    }
}
