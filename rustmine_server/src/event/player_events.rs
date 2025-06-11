use crate::{player::Player, Shared};

#[derive(Clone)]
pub struct PlayerJoinedServer {
    pub player: Shared<Player>,
}
impl super::Event<()> for PlayerJoinedServer {}

#[derive(Clone)]
pub struct PlayerSentPacket<P> where P: Send + Sync + Clone + 'static {
    pub player: Shared<Player>,
    pub packet: P
}
impl<P> super::Event<()> for PlayerSentPacket<P> where P: Send + Sync + Clone + 'static {}
