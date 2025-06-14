use std::io::Error;

use rustmine_lib::game_profile::GameProfile;
use uuid::Uuid;

use crate::{
    id_match,
    packet::{self, Packet, clientbound::login::LoginSuccessPacket, data},
    packet_id, serverbound_packet,
};

pub struct LoginStartPacket {
    pub username: String,
    pub uuid: Uuid,
}

impl Packet for LoginStartPacket {
    packet_id!(0x00);
    serverbound_packet!();

    async fn read_from(id: u32, buffer: Vec<u8>) -> Result<Box<Self>, Box<std::io::Error>> {
        id_match!(id, Self::id());

        let mut position = 0 as usize;
        Ok(Box::new(LoginStartPacket {
            username: data::read_string(&buffer, &mut position).unwrap(),
            uuid: data::read_uuid(&buffer, &mut position).unwrap(),
        }))
    }
}

pub struct LoginAcknowledgedPacket {}

impl Packet for LoginAcknowledgedPacket {
    packet_id!(0x03);

    async fn read_from(
        id: u32,
        _buffer: Vec<u8>,
    ) -> Result<Box<LoginAcknowledgedPacket>, Box<std::io::Error>> {
        id_match!(id, Self::id());
        Ok(Box::new(LoginAcknowledgedPacket {}))
    }

    fn write_to(&self, _buffer: &mut Vec<u8>) {}
}

pub(crate) async fn read_packet(id: u32, buffer: Vec<u8>) -> Result<Box<dyn Packet>, Box<Error>> {
    return match id {
        0x00 => packet::upcast_packet(LoginStartPacket::read_from(id, buffer).await),
        0x03 => packet::upcast_packet(LoginAcknowledgedPacket::read_from(id, buffer).await),
        _ => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Unknown packet id for Status: {}", id),
        ))),
    };
}

pub(crate) async fn handle_login(
    arg: &mut crate::player::PlayerConnection,
) -> Result<(), Box<std::io::Error>> {
    let packet = arg.read_packet().await?;

    let login_start_packet = packet::downcast_packet::<LoginStartPacket>(packet)
        .map_err(|_| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Expected a LoginStartPacket",
            ))
        })
        .unwrap();

    arg.write_packet(&LoginSuccessPacket {
        username: login_start_packet.username.clone(),
        uuid: login_start_packet.uuid,
        // TODO READ TEXTURES AND SIGNATURE
    })
    .await?;

    arg.set_game_profile(GameProfile {
        username: login_start_packet.username.clone(),
        uuid: login_start_packet.uuid
    }).await;

    let packet = arg.read_packet().await?;
    if packet.packet_id() != LoginAcknowledgedPacket::id() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Expected login acknowledgement",
        )));
    }

    Ok(())
}
