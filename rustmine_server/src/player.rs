use std::{error::Error, io::ErrorKind, sync::Arc};

use tokio::{net::TcpStream, sync::Mutex};

use crate::{
    dispatch_packet_event, packet::{
        self, serverbound::{self, handshake::HandshakePacket, status::{self, StatusRequestPacket}}, Packet, RawPacket
    }, RustmineServer, Shared
};



#[derive(Clone, PartialEq, Eq, Debug)]
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
    pub fn id(&self) -> u32 {
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


#[derive(Clone)]
pub struct PlayerConnection {
    server: Shared<RustmineServer>,
    stream: Shared<TcpStream>, // Any I/O should be handled by the player connection implementation
    state:  Shared<State>,
    compression_threshold: u32,
}

#[allow(dead_code)]
pub struct Player {
    pub server: Shared<RustmineServer>,
    connection: Shared<PlayerConnection>,
}

impl PlayerConnection {
    /// Creates a new [`PlayerConnection`].
    pub(crate) fn new(stream: TcpStream, server: &Shared<RustmineServer>) -> Self {
        Self {
            stream: Arc::new(Mutex::new(stream)),
            server: Arc::clone(server),
            state: Arc::new(Mutex::new(State::Handshake)),
            compression_threshold: 0,
        }
    }

    pub async fn read_packet(&mut self) -> Result<Arc<dyn Packet>, Box<std::io::Error>> {
        let (_, id, buffer) = self.read_packet_raw().await
            .map_err(|_| Box::new(std::io::Error::new(ErrorKind::InvalidData, "Error occured whilst reading data")))?;

        let packet = match *self.state.lock().await {
            State::Handshake => match id {
                0 => packet::upcast_packet(HandshakePacket::read_from(id, buffer).await),
                _ => Err(Box::new(std::io::Error::new(
                    ErrorKind::InvalidData,
                    "Packet length cannot be zero",
                )))?,
            },
            State::Status => serverbound::status::read_packet(id, buffer).await,
            State::Login => todo!(),
            State::Play => todo!(),
            State::Configuration => todo!(),
            State::Transfer => todo!(),
        }.map(|boxed| Arc::from(boxed) as Arc<dyn Packet>);

        if let Ok(ref p) = packet {
            if p.packet_id() != id {
                return Err(Box::new(std::io::Error::new(
                    ErrorKind::InvalidData,
                    format!("Packet id mismatch: expected {}, got {}", p.packet_id(), id),
                )));
            }

            let state = {
                let state_lock = self.state.lock().await;
                state_lock.clone()
            };

            dispatch_packet_event! {
                packet = p.clone(),
                server = self.server,
                state = state,
                connection = Mutex::new(self.clone()),
                table = {
                    State::Handshake => [HandshakePacket],
                    State::Status => [StatusRequestPacket],
                }
            } // Maybe centralize this into a packet registry instead of defining a table?
        }

        return packet.map_err(|op| {
            Box::new(std::io::Error::new(
                ErrorKind::InvalidData,
                format!("Failed to read packet: {}", op),
            ))
        });
    }

    pub async fn read_packet_raw(&mut self) -> Result<RawPacket, Box<dyn Error>> {
        packet::read_packet(&mut *self.stream.lock().await, self.compression_threshold).await
    }

    pub async fn handle_handshake(
        &mut self,
        handshake: Arc<HandshakePacket>,
    ) -> Result<(), Box<dyn Error>> {
        match handshake.next_state {
            State::Status => {
                *self.state.lock().await = State::Status;
                status::handle_status_request(self).await?;
            }
            State::Login => {
                *self.state.lock().await = State::Login;
            }
            _ => {
                return Err(Box::new(std::io::Error::new(
                    ErrorKind::InvalidData,
                    "Invalid next state",
                )));
            }
        }

        Ok(())
    }

    pub async fn write_packet(&mut self, packet: &dyn Packet) -> Result<(), Box<std::io::Error>> {
        packet::write_packet(packet, &mut *self.stream.lock().await, self.compression_threshold).await.map_err(|e| Box::new(e))
    }

}
