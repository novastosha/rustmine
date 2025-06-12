use std::io;

use rustmine_server::{
    RustmineServer,
    config::ServerConfig,
    event::player_events::{PlayerJoinedServer, PlayerSentPacket},
    packet::{
        clientbound::status::StatusResponsePacket,
        serverbound::{handshake::HandshakePacket, status::StatusRequestPacket},
    },
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let server = RustmineServer::new(ServerConfig::default());
    {
        let event_bus = server.lock().await.event_bus.clone();

        event_bus
            .listen::<PlayerJoinedServer, _, _, _>(false, |event| async move { None })
            .await;

        event_bus
            .listen::<PlayerSentPacket<StatusRequestPacket>, _, _, _>(false, |event| async move {
                // Send a response back!
                // Now should that be through the event bus or directly to the player?
                println!("Recieved StatusRequestPacket from player!");

                event
                    .player_connection
                    .lock()
                    .await
                    .write_packet(&StatusResponsePacket {
                        response: StatusResponse {
                            version: StatusVersion::default(),
                            players: StatusPlayers::default(),
                            description: ChatComponent {
                                text: "Welcome to Rustmine Server!".to_string(),
                                color: None,
                                bold: None,
                                italic: None,
                                underlined: None,
                                strikethrough: None,
                                obfuscated: None,
                            },
                            favicon: None,
                        }
                    })
                    .await
                    .unwrap();
                None
            })
            .await;

        event_bus
            .listen::<PlayerSentPacket<HandshakePacket>, _, _, _>(false, |event| async move {
                let packet = &event.packet;

                println!(
                    "Recieved HandshakePacket with the following intent: {:?}",
                    packet.next_state
                );
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
