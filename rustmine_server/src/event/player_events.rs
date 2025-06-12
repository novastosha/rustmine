use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{packet::{serverbound::status::StatusRequestPacket, Packet}, player::{Player, PlayerConnection}, Shared};

pub struct PlayerJoinedServer {
    pub player: Shared<Player>,
}
impl super::Event<()> for PlayerJoinedServer {}

pub struct PlayerSentPacket<P> where P: Packet + Send + Sync + ?Sized {
    pub packet: Arc<P>,
    pub player_connection: Mutex<PlayerConnection>,
}

impl<P> super::Event<()> for PlayerSentPacket<P> where P: Packet + Send + Sync + ?Sized {}


