use std::sync::Arc;

use rustmine_lib::{
    chunk::ChunkGenerator,
    dimension::{self, DimensionType},
};

#[derive(Default)]
pub struct WorldManager {
    pub worlds: Vec<Arc<World>>,
}

impl WorldManager {
    pub fn create_world(
        &mut self,
        dimension: &Arc<DimensionType>,
        generator: Arc<dyn ChunkGenerator>,
    ) -> Arc<World> {
        let world = Arc::new(World {
            dimension_type: dimension.clone(),
            generator,
        });

        self.worlds.push(world.clone());
        world
    }
}

pub struct World {
    pub dimension_type: Arc<dimension::DimensionType>,
    pub generator: Arc<dyn ChunkGenerator>,
}
