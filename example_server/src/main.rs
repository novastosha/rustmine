use std::{io, rc::Rc, sync::Arc};

use once_cell::sync::Lazy;
use rustmine_lib::{
    blocks::Block, dimension::DimensionType, register_default_dimension_types, styled, text,
};
use rustmine_server::{
    RustmineServer,
    config::ServerConfig,
    event::{
        player_events::{PlayerJoinedServer, PlayerSentPacket},
        server_events::ServerConfigurationStartEvent,
    },
    packet::{
        clientbound::status::{StatusPlayers, StatusResponse, StatusResponsePacket, StatusVersion},
        serverbound::status::StatusRequestPacket,
    },
    world::World,
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

        let mut spawning_world: Rc<Option<Arc<World>>> = Rc::new(None);

        event_bus
            .listen::<ServerConfigurationStartEvent, _, _, _>(true, |event| async move {
                let mut server = event.server.lock().await;
                register_default_dimension_types!(&server.dimension_type_manager);

                let dimension = &server
                    .dimension_type_manager
                    .get("minecraft:the_end")
                    .unwrap();

                let world = server.world_manager.create_world(dimension, SinewaveGenerator);

                spawning_world = Rc::new(Some(world));
                server.brand_name = "Cool Brandname".to_string();
                None
            })
            .await;

        event_bus.listen::<PlayerJoinedServer, _, _, _>(false, |event| async move {
            let spawning_world = spawning_world.unwrap();
            let player = event.player.lock().await;

            player.set_world(spawning_world);
        });

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
