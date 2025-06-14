use std::{error::Error, io::ErrorKind, sync::Arc};

use rustmine_lib::game_profile::GameProfile;
use tokio::{net::TcpStream, sync::Mutex};

use crate::{
    dispatch_packet_event, packet::{
        self, serverbound::{
            self, configuration,
            handshake::HandshakePacket,
            login::{self, LoginAcknowledgedPacket, LoginStartPacket},
            status::{self, StatusRequestPacket},
        }, Packet, RawPacket
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
pub struct PlayerClientInfo {
    pub brand: String,
    pub locale: String,
    pub view_distance: i8,
    pub chat_mode: i32,
    pub chat_colors: bool,
    pub skin_parts: u8,
    pub main_hand: u8,
    pub text_filtering: bool,
    pub server_listing: bool,
}

impl Default for PlayerClientInfo {
    fn default() -> Self {
        Self {
            brand: "".to_string(),
            locale: "".to_string(),
            view_distance: 0,
            chat_mode: 0,
            chat_colors: false,
            skin_parts: 0,
            main_hand: 0,
            text_filtering: false,
            server_listing: false,
        }
    }
}

#[derive(Clone)]
pub struct PlayerConnection {
    pub server: Shared<RustmineServer>,
    pub game_profile: Shared<Option<GameProfile>>,

    info: Shared<PlayerClientInfo>,
    stream: Shared<TcpStream>, // Any I/O should be handled by the player connection implementation
    state: Shared<State>,
    compression_threshold: u32,
}

#[allow(dead_code)]
pub struct Player {
    pub server: Shared<RustmineServer>,
    pub connection: Shared<PlayerConnection>,
}

impl PlayerConnection {
    /// Creates a new [`PlayerConnection`].
    pub(crate) fn new(stream: TcpStream, server: &Shared<RustmineServer>) -> Self {
        Self {
            info: Arc::new(Mutex::new(PlayerClientInfo::default())),
            stream: Arc::new(Mutex::new(stream)),
            server: Arc::clone(server),
            state: Arc::new(Mutex::new(State::Handshake)),
            game_profile: Arc::new(Mutex::new(None)),
            compression_threshold: 0,
        }
    }

    pub async fn read_packet(&mut self) -> Result<Arc<dyn Packet>, Box<std::io::Error>> {
        let (_, id, buffer) = self.read_packet_raw().await.map_err(|e| {
            Box::new(std::io::Error::new(
                ErrorKind::InvalidData,
                format!("Error occured whilst reading data: {}", e),
            ))
        })?;

        let packet = match *self.state.lock().await {
            State::Handshake => match id {
                0 => packet::upcast_packet(HandshakePacket::read_from(id, buffer).await),
                _ => Err(Box::new(std::io::Error::new(
                    ErrorKind::InvalidData,
                    "Packet length cannot be zero",
                )))?,
            },
            State::Status => serverbound::status::read_packet(id, buffer).await,
            State::Login => serverbound::login::read_packet(id, buffer).await,
            State::Configuration => serverbound::configuration::read_packet(id, buffer).await,
            State::Play => todo!(),
            State::Transfer => todo!(),
        }
        .map(|boxed| Arc::from(boxed) as Arc<dyn Packet>);

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
                    State::Login => [LoginStartPacket, LoginAcknowledgedPacket],
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
                if login::handle_login(self).await.is_ok() {
                    *self.state.lock().await = State::Configuration;
                    if configuration::handle_configuration(self).await.is_ok() {
                        *self.state.lock().await = State::Play;

                        
                    }
                }
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
        packet::write_packet(
            packet,
            &mut *self.stream.lock().await,
            self.compression_threshold,
        )
        .await
        .map_err(|e| Box::new(e))
    }

    pub(crate) async fn update_client_info(
        &mut self,
        client_info_packet: Arc<configuration::ClientInformationConfigPacket>,
        config_plugin_message: Arc<configuration::ConfigurationPluginMessagePacket>,
    ) -> Result<(), Box<std::io::Error>> {
        if config_plugin_message.channel != "minecraft:brand" {
            return Err(Box::new(std::io::Error::new(
                ErrorKind::InvalidData,
                "Expected minecraft:brand channel",
            )));
        }

        let brand = String::from_utf8(config_plugin_message.data.clone())
            .map_err(|_| std::io::Error::new(ErrorKind::InvalidData, "Invalid brand data"))?;

        let mut info = self.info.lock().await;
        info.brand = brand;
        info.locale = client_info_packet.locale.clone();
        info.view_distance = client_info_packet.view_distance;
        info.chat_mode = client_info_packet.chat_mode;
        info.chat_colors = client_info_packet.chat_colors;
        info.skin_parts = client_info_packet.skin_parts;
        info.main_hand = client_info_packet.main_hand as u8;
        info.text_filtering = client_info_packet.text_filtering;
        info.server_listing = client_info_packet.server_listing;

        Ok(())
    }
    
    pub(crate) async fn set_game_profile(&mut self, profile: GameProfile) {
        let mut profile_player = self.game_profile.lock().await;
        *profile_player = Some(profile);
    }
}
