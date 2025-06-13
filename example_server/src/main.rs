use std::io;

use once_cell::sync::Lazy;
use rustmine_lib::{register_default_dimension_types, styled, text};
use rustmine_server::{
    RustmineServer,
    config::ServerConfig,
    event::{player_events::PlayerSentPacket, server_events::ServerConfigurationStartEvent},
    packet::{
        clientbound::status::{StatusPlayers, StatusResponse, StatusResponsePacket, StatusVersion},
        serverbound::status::StatusRequestPacket,
    },
};

static MOTD: Lazy<StatusResponse> = Lazy::new(|| StatusResponse {
    version: StatusVersion::default(),
    players: StatusPlayers::default(),
    description: styled!(
        text!("Rustmine Server."),
        {
            color: "#a50000".to_string(),
            bold: true,
        }
    )
    .append(styled!(
        text!(" Now with components!\n"),
        {
            color: "yellow".to_string(),
            italic: true,
            bold: false
        }
    ))
    .append(styled!(text!("Made with <3 in Rust"), { color: "#fa0000".to_string() })),
    favicon: None,
    enforces_secure_chat: false,
});

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let server = RustmineServer::new(ServerConfig::default());
    {
        let event_bus = &server.lock().await.event_bus;

        event_bus
            .listen::<ServerConfigurationStartEvent, _, _, _>(true, |event| async move {
                let server = event.server.lock().await;

                register_default_dimension_types!(&server.dimension_type_manager);
                None
            })
            .await;

        event_bus
            .listen::<PlayerSentPacket<StatusRequestPacket>, _, _, _>(false, |event| async move {
                event
                    .player_connection
                    .lock()
                    .await
                    .write_packet(&StatusResponsePacket {
                        response: MOTD.clone(),
                    })
                    .await
                    .unwrap();
                None
            })
            .await;
    }

    RustmineServer::run(server).await.map_err(|err| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Caught an unhandled error: {}", err),
        )
    })
}
