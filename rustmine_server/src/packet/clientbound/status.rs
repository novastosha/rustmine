use std::io::{Error, ErrorKind};

use crate::packet::{Packet, data};

pub struct StatusResponsePacket {
    pub response: StatusResponse
}

impl Packet for StatusResponsePacket {
    fn id() -> u32 {
        0x00
    }

    async fn read_from(
        id: u32,
        buffer: Vec<u8>,
    ) -> Result<Box<StatusResponsePacket>, Box<std::io::Error>> {
        if id != Self::id() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "id mismatch!",
            )));
        }
        let mut position = 0 as usize;
        let json_response = data::read_string(&buffer, &mut position).unwrap();

        Ok(Box::new(StatusResponsePacket { 
            response: serde_json::from_str(&json_response)
                .map_err(|e| Box::new(Error::new(ErrorKind::InvalidData, e)))?
        }))
    }

    fn write_to(&self, buffer: &mut Vec<u8>) {
        data::write_string(buffer, &self.response.serde_serialize());
    }

    fn packet_id(&self) -> u32 {
        Self::id()
    }
}

pub struct StatusPongPacket {
    pub payload: u64,
}
impl Packet for StatusPongPacket {
    fn id() -> u32 {
        0x01
    }

    async fn read_from(
        id: u32,
        buffer: Vec<u8>,
    ) -> Result<Box<StatusPongPacket>, Box<std::io::Error>> {
        if id != Self::id() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "id mismatch!",
            )));
        }
        let mut position = 0 as usize;
        let payload = data::read_long(&buffer, &mut position).unwrap();

        Ok(Box::new(StatusPongPacket { payload }))
    }

    fn write_to(&self, buffer: &mut Vec<u8>) {
        data::write_long(buffer, self.payload);
    }

    fn packet_id(&self) -> u32 {
        Self::id()
    }
}

impl Default for StatusVersion {
    fn default() -> Self {
        StatusVersion {
            name: "Rustmine Server".to_string(),
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


#[derive(Serialize)]
pub struct StatusResponse {
    pub version: StatusVersion,
    pub players: StatusPlayers,
    pub description: ChatComponent,
    pub favicon: Option<String>,
}

#[derive(Serialize)]
pub struct StatusVersion {
    pub name: String,
    pub protocol: i32,
}


#[derive(Serialize)]
pub struct StatusPlayers {
    pub max: i32,
    pub online: i32,
    pub sample: Vec<StatusPlayerEntry>,
}

#[derive(Serialize)]
pub struct StatusPlayerEntry {
    pub name: String,
    pub id: String,
}