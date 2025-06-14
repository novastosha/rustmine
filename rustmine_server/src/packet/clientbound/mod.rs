use crate::{id_match, packet::Packet, packet_id};

pub mod configuration;
pub mod login;
pub mod status;

pub struct FinishConfigurationPacket;

impl Packet for FinishConfigurationPacket {
    packet_id!(0x03);
    fn write_to(self: &Self, _: &mut Vec<u8>) {}

    async fn read_from(id: u32, _: Vec<u8>) -> Result<Box<Self>, Box<std::io::Error>> {
        id_match!(id, Self::id());
        Ok(Box::new(FinishConfigurationPacket))
    }
}
