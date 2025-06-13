use uuid::Uuid;

use crate::{
    clientbound_packet,
    packet::{Packet, data},
    packet_id,
};

pub struct LoginSuccessPacket {
    pub username: String,
    pub uuid: Uuid,
}

impl Packet for LoginSuccessPacket {
    packet_id!(0x02);
    clientbound_packet!();

    fn write_to(self: &Self, buffer: &mut Vec<u8>) {
        data::write_uuid(buffer, &self.uuid);
        data::write_string(buffer, &self.username);
        data::write_varint(buffer, 0);
    }
}
