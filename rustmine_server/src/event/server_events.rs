use crate::{RustmineServer, Shared};

pub struct ServerConfigurationStartEvent {
    pub server: Shared<RustmineServer>,
}

impl super::Event<()> for ServerConfigurationStartEvent {}