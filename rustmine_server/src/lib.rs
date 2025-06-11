pub type Shared<T> = Arc<Mutex<T>>; // Move this elsewhere maybe?

pub mod config;
pub mod event;
pub mod packet;
pub mod player;

use std::{error::Error, sync::Arc};
use tokio::{net::TcpListener, sync::Mutex, task};

use crate::{config::ServerConfig, event::EventBus, packet::{serverbound::handshake::HandshakePacket, Packet}, player::PlayerConnection};

pub struct RustmineServer {
    pub config: ServerConfig,
    pub event_bus: EventBus,
}

impl RustmineServer {
    pub fn new(config: ServerConfig) -> Shared<RustmineServer> {
        Arc::new(Mutex::new(RustmineServer {
            config,
            event_bus: EventBus::default(),
        }))
    }

    /*pub async fn get_event_bus(server: &Shared<RustmineServer>) -> Result<Shared<EventBus>, std::io::Error> {
        let server = server.lock().await;
        let cloned_event_bus = server.event_bus.clone();
        drop(server);

        Ok(cloned_event_bus)
    }*/

    pub async fn run(server: Shared<RustmineServer>) -> Result<(), Box<dyn Error>> {
        let server = server.lock().await;

        let listener = TcpListener::bind(format!(
            "{}:{}",
            server.config.bind_address, server.config.port
        ))
        .await?;

        println!("Server listening on port: {:?}", server.config.port);
        drop(server);

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    task::spawn(async move {
                        let connection = Arc::new(Mutex::new(PlayerConnection::new(stream)));

                        let mut cnx = connection.lock().await;
                        let handshake = HandshakePacket::read_from(&mut cnx).await.unwrap();

                        println!("{:?}",handshake.server_address)
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
