use rustmine_lib::data;

use crate::{id_match, packet::Packet, packet_id, serverbound_packet};

pub struct ConfigurationPluginMessagePacket {
    pub channel: String,
    pub data: Vec<u8>,
}

impl Packet for ConfigurationPluginMessagePacket {
    packet_id!(0x02);
    serverbound_packet!();

    async fn read_from(
        id: u32,
        buffer: Vec<u8>,
    ) -> Result<Box<ConfigurationPluginMessagePacket>, Box<std::io::Error>> {
        id_match!(id, Self::id());

        let mut position = 0;
        let channel = data::read_string(&buffer, &mut position)?;
        let data_length = data::read_varint(&buffer, &mut position)? as usize;
        let data = data::read_bytes(&buffer, &mut position, data_length)?;

        Ok(Box::new(ConfigurationPluginMessagePacket { channel, data }))
    }
}
