// Common structs between the clientbound and the serverbound packets during the Configuration state

#[derive(PartialEq, Eq)]
pub struct ConfigKnownPackEntry {
    pub name: String, // Namespaced
    pub version: String,
}

impl ConfigKnownPackEntry {
    pub fn minecraft_core() -> ConfigKnownPackEntry {
        ConfigKnownPackEntry { name: "minecraft:core".to_string(), version: "1.21.6".to_string() }
    }
}
