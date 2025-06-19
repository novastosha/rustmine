use rustmine_lib::component::Component;
use serde::{Deserialize, Serialize};

use crate::{
    clientbound_packet,
    packet::{Packet, data},
    packet_id,
};

pub struct StatusResponsePacket {
    pub response: StatusResponse,
}

impl Packet for StatusResponsePacket {
    packet_id!(0x00);
    clientbound_packet!();

    fn write_to(&self, buffer: &mut Vec<u8>) {
        data::write_string(buffer, &serde_json::to_string(&self.response).unwrap());
    }
}

pub struct StatusPongPacket {
    pub payload: u64,
}
impl Packet for StatusPongPacket {
    packet_id!(0x01);
    clientbound_packet!();

    fn write_to(&self, buffer: &mut Vec<u8>) {
        data::write_long(buffer, self.payload);
    }
}

impl Default for StatusVersion {
    fn default() -> Self {
        StatusVersion {
            name: "Rustmine".to_string(),
            protocol: crate::PROTOCOL_VERSION,
        }
    }
}

impl Default for StatusPlayers {
    fn default() -> Self {
        StatusPlayers {
            max: 0,
            online: 0,
            sample: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StatusResponse {
    pub version: StatusVersion,
    pub players: StatusPlayers,
    pub description: Component,
    // prefix the favicon with "data:image/png;base64," if it's a base64 encoded image
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,

    #[serde(rename = "enforcesSecureChat")]
    pub enforces_secure_chat: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StatusVersion {
    pub name: String,
    pub protocol: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StatusPlayers {
    pub max: i32,
    pub online: i32,
    pub sample: Vec<StatusPlayerEntry>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StatusPlayerEntry {
    pub name: String,
    pub id: String,
}
