use rustmine_lib::data;

use crate::{id_match, packet::Packet, packet_id, serverbound_packet};

pub struct ClientInformationConfigPacket {
    pub locale: String,
    pub view_distance: i8,
    pub chat_mode: i32,
    pub chat_colors: bool,
    pub skin_parts: u8,
    pub main_hand: i32,
    pub text_filtering: bool,
    pub server_listing: bool,
}

impl Packet for ClientInformationConfigPacket {
    packet_id!(0x00);
    serverbound_packet!();

    async fn read_from(
        id: u32,
        buffer: Vec<u8>,
    ) -> Result<Box<ClientInformationConfigPacket>, Box<std::io::Error>> {
        id_match!(id, Self::id());

        let mut position = 0;
        let locale = data::read_string(&buffer, &mut position)?;
        let view_distance = data::read_varint(&buffer, &mut position)?;
        let chat_mode = data::read_varint(&buffer, &mut position)?;
        let chat_colors = data::read_bool(&buffer, &mut position)?;
        let skin_parts = data::read_byte(&buffer, &mut position)?;
        let main_hand = data::read_varint(&buffer, &mut position)?;
        let text_filtering = data::read_bool(&buffer, &mut position)?;
        let server_listing = data::read_bool(&buffer, &mut position)?;

        Ok(Box::new(ClientInformationConfigPacket {
            locale,
            view_distance: view_distance as i8,
            chat_mode: chat_mode as i32,
            chat_colors,
            skin_parts,
            main_hand: main_hand as i32,
            text_filtering,
            server_listing,
        }))
    }

}
