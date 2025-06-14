use rustmine_lib::{common::configuration_state::ConfigKnownPackEntry, data};

use crate::{clientbound_packet, packet::Packet, packet_id};

pub struct ConfigSelectKnownPacksPacket {
    pub entries: Vec<ConfigKnownPackEntry>
}

impl Packet for ConfigSelectKnownPacksPacket {
    packet_id!(0x0E);
    clientbound_packet!();
    
    fn write_to(self: &Self, buffer: &mut Vec<u8>) {
        let entries_length = self.entries.len();

        data::write_varint(buffer, entries_length.try_into().unwrap());
        for entry in &self.entries {
            let split_entry_name = entry.name.split_once(":").unwrap();

            data::write_string(buffer, split_entry_name.0);
            data::write_string(buffer, split_entry_name.1);
            data::write_string(buffer, &entry.version);
        }

    }
}

