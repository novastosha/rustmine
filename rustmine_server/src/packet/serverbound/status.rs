use std::io::Error;

use flate2::Status;
use serde::Serialize;

use crate::packet::{self, Packet, clientbound::status::StatusPongPacket};

pub struct StatusRequestPacket {}

impl Packet for StatusRequestPacket {
    fn id() -> u32 {
        0x00
    }

    async fn read_from(
        id: u32,
        _buffer: Vec<u8>,
    ) -> Result<Box<StatusRequestPacket>, Box<std::io::Error>> {
        if id != Self::id() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "id mismatch!",
            )));
        }

        // For StatusRequestPacket, no additional data is needed.
        Ok(Box::new(StatusRequestPacket {}))
    }

    fn write_to(&self, _buffer: &mut Vec<u8>) {
        // No data to write for StatusRequestPacket
    }

    fn packet_id(&self) -> u32 {
        Self::id()
    }
}

pub struct StatusPingPacket {
    pub payload: u64,
}

impl Packet for StatusPingPacket {
    fn id() -> u32 {
        0x01
    }

    async fn read_from(
        id: u32,
        buffer: Vec<u8>,
    ) -> Result<Box<StatusPingPacket>, Box<std::io::Error>> {
        if id != Self::id() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "id mismatch!",
            )));
        }

        let mut position = 0 as usize;
        let payload = crate::packet::data::read_long(&buffer, &mut position).unwrap();

        Ok(Box::new(StatusPingPacket { payload }))
    }

    fn write_to(&self, buffer: &mut Vec<u8>) {
        crate::packet::data::write_long(buffer, self.payload);
    }

    fn packet_id(&self) -> u32 {
        Self::id()
    }
}

pub(crate) async fn read_packet(id: u32, buffer: Vec<u8>) -> Result<Box<dyn Packet>, Box<Error>> {
    return match id {
        0x00 => packet::upcast_packet(StatusRequestPacket::read_from(id, buffer).await),
        0x01 => packet::upcast_packet(StatusPingPacket::read_from(id, buffer).await),
        _ => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Unknown packet id for Status: {}", id),
        ))),
    };
}

pub(crate) async fn handle_status_request(
    arg: &mut crate::player::PlayerConnection,
) -> Result<(), Box<std::io::Error>> {
    let packet = arg.read_packet().await?;
    if packet.packet_id() != StatusRequestPacket::id() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Expected StatusRequestPacket",
        )));
    }

    let packet = arg.read_packet().await?;
    let status_ping = packet::downcast_packet::<StatusPingPacket>(packet).map_err(|_| {
        Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Failed to downcast to StatusPingPacket",
        ))
    });

    if let Ok(ping_packet) = status_ping {
        // Send pong response
        let pong_response = StatusPongPacket {
            payload: ping_packet.payload,
        };
        arg.write_packet(&pong_response).await?;
    }

    Ok(())
}


