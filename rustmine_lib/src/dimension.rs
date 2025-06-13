pub struct DimensionTypeRegistry {
    pub dimensions: Vec<DimensionType>,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionType {
    pub piglin_safe: bool,
    pub natural: bool,
    pub ambient_light: f32,
    pub fixed_time: Option<u64>,
    pub infiniburn: String,
    pub respawn_anchor_works: bool,
    pub bed_works: bool,
    pub effects: String,
    pub has_skylight: bool,
    pub has_ceiling: bool,
    pub ultrawarm: bool,
    pub has_raids: bool,
    pub logical_height: i32,
    pub coordinate_scale: f64,
    pub min_y: i32,
    pub height: i32,
    pub monster_spawn_light_level: Option<u8>,
    pub monster_spawn_block_light_limit: u8,
}

impl DimensionType {
    pub fn validate(&self) -> Result<(), String> {
        if self.min_y % 16 != 0 {
            return Err("min_y must be multiple of 16".to_string());
        }
        if self.height % 16 != 0 {
            return Err("height must be multiple of 16".to_string());
        }
        if self.logical_height > self.height || self.logical_height < 0 {
            return Err("logical_height must be between 0 and height".to_string());
        }
        Ok(())
    }
}

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Default)]
pub struct DimensionTypeManager {
    pub(crate) registered_types: RwLock<HashMap<String, Arc<DimensionType>>>,
}

impl DimensionTypeManager {
    pub fn register(&self, key: &str, dim: DimensionType) -> Result<(), String> {
        dim.validate()?;

        let mut map = self.registered_types.write().unwrap();
        if map.contains_key(key) {
            return Err(format!("Dimension type '{}' already registered", key));
        }

        map.insert(key.to_string(), Arc::new(dim));
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<Arc<DimensionType>> {
        self.registered_types.read().unwrap().get(key).cloned()
    }
}

#[macro_export]
macro_rules! dimension_type {
    (
        $(
            {
                $($field:ident : $value:expr),* $(,)?
            }
        )*
    ) => {{
        let dim = DimensionType {
            $(
                $field: $value,
            )*
        };
        dim.validate().expect(concat!("Invalid dimension type: ", $name));
        dim
    }};
}

#[macro_export]
macro_rules! register_default_dimension_types {
    ($manager:expr) => {{
        use rustmine_lib::dimension::DimensionType;

        $manager.register("minecraft:overworld", DimensionType {
            piglin_safe: false,
            natural: true,
            ambient_light: 0.0,
            fixed_time: None,
            infiniburn: "#minecraft:infiniburn_overworld".to_string(),
            respawn_anchor_works: false,
            bed_works: true,
            effects: "minecraft:overworld".to_string(),
            has_skylight: true,
            has_ceiling: false,
            ultrawarm: false,
            has_raids: true,
            logical_height: 384,
            coordinate_scale: 1.0,
            min_y: -64,
            height: 384,
            monster_spawn_light_level: Some(7),
            monster_spawn_block_light_limit: 0,
        }).unwrap();

        $manager.register("minecraft:the_nether", DimensionType {
            piglin_safe: true,
            natural: false,
            ambient_light: 0.1,
            fixed_time: None,
            infiniburn: "#minecraft:infiniburn_nether".to_string(),
            respawn_anchor_works: true,
            bed_works: false,
            effects: "minecraft:the_nether".to_string(),
            has_skylight: false,
            has_ceiling: true,
            ultrawarm: true,
            has_raids: false,
            logical_height: 128,
            coordinate_scale: 8.0,
            min_y: 0,
            height: 128,
            monster_spawn_light_level: Some(0),
            monster_spawn_block_light_limit: 15,
        }).unwrap();

        $manager.register("minecraft:the_end", DimensionType {
            piglin_safe: false,
            natural: false,
            ambient_light: 0.0,
            fixed_time: Some(6000),
            infiniburn: "#minecraft:infiniburn_end".to_string(),
            respawn_anchor_works: false,
            bed_works: false,
            effects: "minecraft:the_end".to_string(),
            has_skylight: false,
            has_ceiling: true,
            ultrawarm: false,
            has_raids: true,
            logical_height: 256,
            coordinate_scale: 1.0,
            min_y: 0,
            height: 256,
            monster_spawn_light_level: Some(0),
            monster_spawn_block_light_limit: 15,
        }).unwrap();
    }};
}