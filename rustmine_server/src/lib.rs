pub const PROTOCOL_VERSION: i32 = 771;

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
    config::ServerConfig, event::{server_events::ServerConfigurationStartEvent, EventBus}, packet::serverbound::handshake::HandshakePacket,
    player::PlayerConnection,
};

pub struct RustmineServer {
    pub brand_name: String,
    pub config: ServerConfig,
    pub event_bus: Arc<EventBus>,
    pub dimension_type_manager: dimension::DimensionTypeManager,
    pub world_manager: world::WorldManager,
}

impl RustmineServer {
    pub fn new(config: ServerConfig) -> Shared<RustmineServer> {
        Arc::new(Mutex::new(RustmineServer {
            config,
            event_bus: Arc::new(EventBus::default()),
            dimension_type_manager: dimension::DimensionTypeManager::default(),
            world_manager: world::WorldManager::default(),
            brand_name: "Rustmine".to_owned(),
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

        let event_bus = server_lock.event_bus.clone();
        drop(server_lock);

        event_bus.dispatch(&Arc::new(ServerConfigurationStartEvent {
            server: server.clone()
        })).await;

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
