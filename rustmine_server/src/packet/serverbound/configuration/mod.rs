use std::vec;

use crate::{
    packet::{
        self, clientbound::{self, configuration::ConfigSelectKnownPacksPacket, FinishConfigurationPacket}, Packet
    },
    player::PlayerConnection,
};

// Because configuration has too many packets, each packet will have its own file.
mod client_information;
pub use client_information::*;

mod config_plugin_message;
pub use config_plugin_message::*;

mod client_known_packs;
pub use client_known_packs::*;
use rustmine_lib::common::configuration_state::ConfigKnownPackEntry;

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

    let brand_name = {
        let server = cnx.server.lock().await;
        server.brand_name.to_owned()
    };

    cnx.write_packet(
        &clientbound::configuration::ConfigurationPluginMessagePacket::brand_packet(brand_name),
    )
    .await?;

    // Send known packs to client
    cnx.write_packet(&ConfigSelectKnownPacksPacket {
        entries: vec![ConfigKnownPackEntry::minecraft_core()],
    })
    .await?;

    let packet = cnx.read_packet().await?; // Downcast this later
    let client_known_packs = packet::downcast_packet::<ClientKnownPacksPacket>(packet).unwrap();

    if !client_known_packs
        .known_packs
        .contains(&ConfigKnownPackEntry::minecraft_core())
    {
        println!("Client doesn't know minecraft:core!")
    }

    cnx.write_packet(&FinishConfigurationPacket).await?;
    let packet = cnx.read_packet().await?;

    if packet.packet_id() != FinishConfigurationPacket::id() {
        return Err(
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Expected client to acknowledge configuration end.",
            ))
        );
    }

    Ok(())
}

pub(crate) async fn read_packet(
    id: u32,
    buffer: Vec<u8>,
) -> Result<Box<dyn Packet + 'static>, Box<std::io::Error>> {
    match id {
        0x00 => packet::upcast_packet(ClientInformationConfigPacket::read_from(id, buffer).await),
        0x02 => packet::upcast_packet(ConfigurationPluginMessagePacket::read_from(id, buffer).await),
        0x03 => packet::upcast_packet(FinishConfigurationPacket::read_from(id, buffer).await),
        0x07 => packet::upcast_packet(ClientKnownPacksPacket::read_from(id, buffer).await), // Screams macro

        _ => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Unknown packet id for Configuration: {}", id),
        ))),
    }
}
