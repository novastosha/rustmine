#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub bind_address: String,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> ServerConfig {
        ServerConfig { bind_address: "0.0.0.0".to_string(),
                       port: 25565  }
    }
}