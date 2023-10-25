use std::collections::HashMap;
use bytes::Bytes;
use rumqttc::{AsyncClient, EventLoop, MqttOptions, QoS};
use crate::message::Message;
use crate::network::Network;

pub struct MQTTNetwork {
    client: AsyncClient,
    eventloop: EventLoop,
}

impl MQTTNetwork {
    const MQTT_BROKER: &'static str = "test.mosquitto.org";
    const MQTT_PORT: u16 = 1883;
    const TOPIC_PREFIX: &'static str = "hello-rufi/gradient";

    pub fn new(mqtt_options: MqttOptions) -> MQTTNetwork {
        let (client, eventloop) = AsyncClient::new(mqtt_options, 20);
        MQTTNetwork {
            client,
            eventloop,
        }
    }
}

impl Network for MQTTNetwork {
    fn send(&self, self_id: i32, data: Message) {
        let _ = self.client.publish(format!("{}/{}", MQTTNetwork::TOPIC_PREFIX, self_id), QoS::AtMostOnce, false, Bytes::new());
    }

    fn receive(&self) -> HashMap<i32, Message> {
        todo!()
    }
}