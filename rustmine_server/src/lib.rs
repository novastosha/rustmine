pub const PROTOCOL_VERSION: i32 = 770;

pub type Shared<T> = Arc<Mutex<T>>; // Move this elsewhere maybe?

pub mod config;
pub mod event;
pub mod packet;
pub mod player;
pub mod world;

use std::sync::Arc;
use rustmine_lib::dimension;
use tokio::{net::TcpListener, sync::Mutex, task};

use crate::{
    config::ServerConfig, event::EventBus, packet::serverbound::handshake::HandshakePacket,
    player::PlayerConnection,
};

pub struct RustmineServer {
    pub config: ServerConfig,
    pub event_bus: EventBus,
    pub dimension_type_manager: dimension::DimensionTypeManager,
}

impl RustmineServer {
    pub fn new(config: ServerConfig) -> Shared<RustmineServer> {
        Arc::new(Mutex::new(RustmineServer {
            config,
            event_bus: EventBus::default(),
            dimension_type_manager: dimension::DimensionTypeManager::default(),
        }))
    }

    pub async fn run(server: Shared<RustmineServer>) -> Result<(), Box<std::io::Error>> {
        let server_lock = server.lock().await;

        let listener = TcpListener::bind(format!(
            "{}:{}",
            server_lock.config.bind_address, server_lock.config.port
        ))
        .await?;

        println!("Server listening on port: {:?}", server_lock.config.port);

        drop(server_lock);
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    let server = Arc::clone(&server);

                    task::spawn(async move {
                        let mut connection = PlayerConnection::new(stream, &server);
                        let packet = connection.read_packet().await.unwrap();

                        let handshake = packet::downcast_packet::<HandshakePacket>(packet).unwrap();

                        println!("New connection accepted from: {}", addr);
                        connection.handle_handshake(handshake).await.unwrap();

                        ()
                    });
                }
                Err(err) => {
                    eprintln!(
                        "There was an error accepting an incoming connection: {:?}",
                        err
                    )
                }
            }
        }
    }
}
