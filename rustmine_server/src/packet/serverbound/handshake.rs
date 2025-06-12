use std::{io::Error, io::ErrorKind};

use crate::{
    packet::{Packet, data},
    player::State,
};

pub struct HandshakePacket {
    pub protocol: u32,
    pub server_address: String,
    pub port: u16,
    pub next_state: State,
}

impl HandshakePacket {}

impl Packet for HandshakePacket {
    fn id() -> u32 {
        0x00
    }

    fn packet_id(&self) -> u32 {
        Self::id()
    }

    async fn read_from(id: u32, buffer: Vec<u8>) -> Result<Box<HandshakePacket>, Box<Error>> {
        if id != Self::id() {
            return Err(Box::new(std::io::Error::new(
                ErrorKind::Other,
                "id mismatch!",
            )));
        }

        let mut position = 0 as usize;

        Ok(Box::new(HandshakePacket {
            protocol: data::read_varint(&buffer, &mut position).unwrap(),
            server_address: data::read_string(&buffer, &mut position).unwrap(),
            port: data::read_ushort(&buffer, &mut position).unwrap(),
            next_state: data::read_varint(&buffer, &mut position).and_then(|id| match id {
                1 => Ok(State::Status),
                2 => Ok(State::Login),
                3 => Ok(State::Transfer),
                _ => Err(std::io::Error::new(
                    ErrorKind::InvalidData,
                    "Invalid next state id",
                )),
            })?,
        }))
    }

    fn write_to(&self, buffer: &mut Vec<u8>) {
        data::write_varint(buffer, self.protocol);
        data::write_string(buffer, &self.server_address);
        data::write_ushort(buffer, self.port);
        data::write_varint(buffer, self.next_state.id());
    }
}
