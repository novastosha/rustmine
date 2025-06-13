use crate::{
    packet::{self, Packet},
    player::PlayerConnection,
};

// Because configuration has too many packets, each packet will have its own file.
mod client_information;
pub use client_information::*;

mod config_plugin_message;
pub use config_plugin_message::*;

pub(crate) async fn handle_configuration(
    cnx: &mut PlayerConnection,
) -> Result<(), Box<std::io::Error>> {
    let packet = cnx.read_packet().await?;

    let config_plugin_message = packet::downcast_packet::<ConfigurationPluginMessagePacket>(packet)
        .map_err(|_| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Expected a ConfigurationPluginMessagePacket",
            ))
        })
        .unwrap();

    let packet = cnx.read_packet().await?;
    let client_info_packet = packet::downcast_packet::<ClientInformationConfigPacket>(packet)
        .map_err(|_| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Expected a ClientInformationConfigPacket",
            ))
        })
        .unwrap();

    cnx.update_client_info(client_info_packet, config_plugin_message)
        .await?;

    // Send known packs response

    Ok(())
}

pub(crate) async fn read_packet(
    id: u32,
    buffer: Vec<u8>,
) -> Result<Box<dyn Packet + 'static>, Box<std::io::Error>> {
    match id {
        0x00 => packet::upcast_packet(ClientInformationConfigPacket::read_from(id, buffer).await),
        0x02 => {
            packet::upcast_packet(ConfigurationPluginMessagePacket::read_from(id, buffer).await)
        }
        _ => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Unknown packet id for Configuration: {}", id),
        ))),
    }
}
