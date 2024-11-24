// Import all the packets from their modules
pub mod packets;

pub use packets::{
    connect::ConnectPacket,
    connack::ConnAckPacket,
    /*
    publish::{PublishPacket, PubAckPacket, PubRecPacket, PubRelPacket, PubCompPacket},
    subscribe::{SubscribePacket, UnsubscribePacket},
    suback::{SubAckPacket, UnsubAckPacket},
    ping::{PingReqPacket, PingRespPacket},
    disconnect::DisconnectPacket,
    auth::AuthPacket, */
};

// Represent all MQTT packet types
#[derive(Debug, PartialEq)]
pub enum MqttPacket {
    Connect(ConnectPacket),         // Packet ID: 1
    ConnAck(ConnAckPacket),         // Packet ID: 2
    /*Publish(PublishPacket),         // Packet ID: 3
    PubAck(PubAckPacket),           // Packet ID: 4
    PubRec(PubRecPacket),           // Packet ID: 5
    PubRel(PubRelPacket),           // Packet ID: 6
    PubComp(PubCompPacket),         // Packet ID: 7
    Subscribe(SubscribePacket),     // Packet ID: 8
    SubAck(SubAckPacket),           // Packet ID: 9
    Unsubscribe(UnsubscribePacket), // Packet ID: 10
    UnsubAck(UnsubAckPacket),       // Packet ID: 11
    PingReq(PingReqPacket),         // Packet ID: 12
    PingResp(PingRespPacket),       // Packet ID: 13
    Disconnect(DisconnectPacket),   // Packet ID: 14
    Auth(AuthPacket),  */             // Packet ID: 15
}
