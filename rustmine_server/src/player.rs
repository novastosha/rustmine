use std::{error::Error, io::ErrorKind, sync::Arc};

use tokio::{net::TcpStream, sync::Mutex};

use crate::{packet::{self, serverbound::handshake::HandshakePacket, Packet, RawPacket}, RustmineServer, Shared};

#[derive(Clone, PartialEq, Eq)]
pub enum State {
    Handshake,
    Play,
    Login,
    Transfer,
    Status,
    Configuration,
}

impl State {
    /// Returns the id of this [`State`].
    fn id(&self) -> u64 {
        match self {
            State::Handshake => 0,
            State::Status => 1,
            State::Login => 2,
            State::Transfer => 3,
            State::Play => 4,
            State::Configuration => 5, // Both the Play and Configuration ids are not used in the specification
        }
    }
}

pub struct PlayerConnection {
    stream: TcpStream, // Any I/O should be handled by the player connection implementation
    state: State,
    compression_threshold: u32,
}

pub struct Player {
    pub server: Shared<RustmineServer>,
    connection: Shared<PlayerConnection>,
}

impl PlayerConnection {
    /// Creates a new [`PlayerConnection`].
    pub(crate) fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            state: State::Handshake,
            compression_threshold: 0
        }
    }
    
    pub async fn read_packet(&mut self) -> Result<Box<dyn Packet>, Box<dyn Error>> {
        let (_, id, buffer) = self.read_packet_raw().await?;

        match self.state {
            State::Handshake => {
                match id {
                    0 => HandshakePacket::read_from(self).await,
                    _ => Err(Box::new(std::io::Error::new(ErrorKind::InvalidData, "Packet length cannot be zero")))?,
                }
            },
            State::Play => todo!(),
            State::Login => todo!(),
            State::Transfer => todo!(),
            State::Status => todo!(),
            State::Configuration => todo!(),
        }.map(|value| value as Box<dyn Packet>)
    }
    
    pub async fn read_packet_raw(&mut self) -> Result<RawPacket,Box<dyn Error>> {
       packet::read_packet(&mut self.stream, self.compression_threshold).await
    }
}
