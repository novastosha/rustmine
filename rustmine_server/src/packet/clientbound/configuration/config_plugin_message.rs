use rustmine_lib::data;

use crate::{clientbound_packet, packet::Packet, packet_id};

pub struct ConfigurationPluginMessagePacket {
    pub channel: String,
    pub data: Vec<u8>,
}
impl ConfigurationPluginMessagePacket {
    pub fn brand_packet(brand: String) -> ConfigurationPluginMessagePacket {
        ConfigurationPluginMessagePacket { channel: "minecraft:brand".to_string(), data: brand.into_bytes() }
    }
}

impl Packet for ConfigurationPluginMessagePacket {
    packet_id!(0x01);
    clientbound_packet!();
    
    fn write_to(self: &Self, buffer: &mut Vec<u8>) {
        data::write_string(buffer, &self.channel);
        data::write_varint(buffer, self.data.len().try_into().unwrap());
        data::write_bytes(buffer, &self.data);
    }
}
