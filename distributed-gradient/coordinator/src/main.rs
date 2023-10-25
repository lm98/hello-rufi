use std::time::Duration;
use rumqttc::{AsyncClient, MqttOptions, QoS};

#[tokio::main]
async fn main() {
    let mut mqttoptions = MqttOptions::new("pinger", "test.mosquitto.org", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 20);
    client.subscribe("hello-rufi/gradient/subscriptions", QoS::AtMostOnce).await.unwrap();

    let devices = vec![1, 2, 3, 4, 5];
    let mut subscriptions: Vec<i32> = vec![];

    loop {
        let notification = eventloop.poll().await.unwrap();
        match notification {
            rumqttc::Event::Incoming(rumqttc::Incoming::Publish(p)) => {
                let payload = p.payload;
                println!("Received: {:?}", payload);
                let id: i32 = String::from_utf8(payload.to_vec()).unwrap().parse().unwrap();
                subscriptions.push(id);
                println!("Subscriptions: {:?}", subscriptions);
                if subscriptions == devices {
                    println!("All devices have subscribed!");
                    //client.publish("hello-rufi/gradient/start", QoS::AtMostOnce, false, "Start".to_string()).await.unwrap();
                    break;
                }
            }
            _ => {}
        }
    }
}
