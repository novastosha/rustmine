use rustmine_lib::{common::configuration_state::ConfigKnownPackEntry, data};

use crate::{id_match, packet::Packet, packet_id, serverbound_packet};

pub struct ClientKnownPacksPacket {
    pub known_packs: Vec<ConfigKnownPackEntry>,
}

impl Packet for ClientKnownPacksPacket {
    packet_id!(0x07);
    serverbound_packet!();

    async fn read_from(id: u32, buffer: Vec<u8>) -> Result<Box<Self>, Box<std::io::Error>> {
        id_match!(id, Self::id());

        let mut known_packs: Vec<ConfigKnownPackEntry> = vec![];
        let mut position = 0 as usize;

        let length = data::read_varint(&buffer, &mut position).unwrap();
        for _ in 0..length {
            let namespace = data::read_string(&buffer, &mut position).unwrap();
            let name = data::read_string(&buffer, &mut position).unwrap();
            let version = data::read_string(&buffer, &mut position).unwrap();

            known_packs.push(ConfigKnownPackEntry {
                name: format!("{}:{}", namespace, name),
                version,
            });
        }

        Ok(Box::new(ClientKnownPacksPacket { known_packs }))
    }
}
