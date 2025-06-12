pub const PROTOCOL_VERSION: i32 = 770;

pub type Shared<T> = Arc<Mutex<T>>; // Move this elsewhere maybe?

pub mod config;
pub mod event;
pub mod packet;
pub mod player;

use std::sync::Arc;
use tokio::{net::TcpListener, sync::Mutex, task};

use crate::{
    config::ServerConfig, event::EventBus, packet::serverbound::handshake::HandshakePacket,
    player::PlayerConnection,
};

pub struct RustmineServer {
    pub config: ServerConfig,
    pub event_bus: Arc<EventBus>,
}

impl RustmineServer {
    pub fn new(config: ServerConfig) -> Shared<RustmineServer> {
        Arc::new(Mutex::new(RustmineServer {
            config,
            event_bus: Arc::new(EventBus::default()),
        }))
    }

    /*pub async fn get_event_bus(server: &Shared<RustmineServer>) -> Result<Shared<EventBus>, std::io::Error> {
        let server = server.lock().await;
        let cloned_event_bus = server.event_bus.clone();
        drop(server);

        Ok(cloned_event_bus)
    }*/

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
