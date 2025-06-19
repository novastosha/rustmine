use crate::blocks::Block;
use std::sync::{Arc, RwLock};
use fastnbt::to_bytes;
use fastnbt::value::Value;


pub trait ChunkGenerator: Send + Sync {
    fn generate_chunk(&self, chunk: Arc<Chunk>) -> ();
}

/// Represents a palette of unique blocks in a chunk section.
#[derive(Debug, Clone)]
pub struct Palette {
    blocks: Vec<Block>,
}

impl Palette {
    pub fn new() -> Self {
        Self { blocks: vec![Block::Air] }
    }

    /// Returns the palette index for the given block, adding it if not present.
    pub fn get_or_insert(&mut self, block: Block) -> u8 {
        if let Some(idx) = self.blocks.iter().position(|&b| b == block) {
            idx as u8
        } else {
            self.blocks.push(block);
            (self.blocks.len() - 1) as u8
        }
    }

    pub fn get(&self, idx: u8) -> Block {
        *self.blocks.get(idx as usize).unwrap_or(&Block::Air)
    }

    pub fn as_vec(&self) -> &Vec<Block> {
        &self.blocks
    }
}

/// Represents a 16x16x16 chunk section.
pub struct ChunkSection {
    palette: RwLock<Palette>,
    blocks: RwLock<Box<[[[u8; 16]; 16]; 16]>>, // palette indices
}

impl ChunkSection {
    pub fn new() -> Self {
        Self {
            palette: RwLock::new(Palette::new()),
            blocks: RwLock::new(Box::new([[[0u8; 16]; 16]; 16])),
        }
    }

    pub fn set_block(&self, x: u8, y: u8, z: u8, block: Block) {
        if x < 16 && y < 16 && z < 16 {
            let mut palette = self.palette.write().unwrap();
            let idx = palette.get_or_insert(block);
            let mut blocks = self.blocks.write().unwrap();
            blocks[x as usize][y as usize][z as usize] = idx;
        }
    }

    pub fn get_block(&self, x: u8, y: u8, z: u8) -> Block {
        if x < 16 && y < 16 && z < 16 {
            let blocks = self.blocks.read().unwrap();
            let idx = blocks[x as usize][y as usize][z as usize];
            let palette = self.palette.read().unwrap();
            palette.get(idx)
        } else {
            Block::Air
        }
    }
    pub fn encode_nbt(&self, y_index: u8) -> Value {
        let palette = self.palette.read().unwrap();
        let blocks = self.blocks.read().unwrap();

        let mut block_indices = Vec::with_capacity(16 * 16 * 16);
        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    block_indices.push(blocks[x][y][z]);
                }
            }
        }

        let mut section = std::collections::HashMap::new();
        section.insert("Y".to_string(), Value::Byte(y_index as i8));
        section.insert(
            "Palette".to_string(),
            Value::List(
                palette
                    .as_vec()
                    .iter()
                    .map(|b| Value::Int(blocks::get_block_registry_entry(block).default_state as i32))
                    .collect(),
            ),
        );
        section.insert(
            "BlockStates".to_string(),
            Value::ByteArray(fastnbt::ByteArray::new(block_indices.into_iter().map(|b| b as i8).collect())),
        );
        Value::Compound(section)
    }
}

/// Represents a full 16x256x16 chunk, composed of 16 chunk sections.
pub struct Chunk {
    pub x: i32,
    pub z: i32,
    sections: Vec<Arc<ChunkSection>>, // 16 sections, each 16 blocks tall
}

impl Chunk {
    pub fn new(x: i32, z: i32) -> Self {
        let mut sections = Vec::with_capacity(16);
        for _ in 0..16 {
            sections.push(Arc::new(ChunkSection::new()));
        }
        Self { x, z, sections }
    }

    pub fn coordinates(&self) -> (i32, i32) {
        (self.x, self.z)
    }

    pub fn set_block(&self, x: u8, y: u8, z: u8, block: Block) {
        if x < 16 && y < 256 && z < 16 {
            let section_idx = (y / 16) as usize;
            let section_y = y % 16;
            self.sections[section_idx].set_block(x, section_y, z, block);
        }
    }

    pub fn get_block(&self, x: u8, y: u8, z: u8) -> Block {
        if x < 16 && y < 256 && z < 16 {
            let section_idx = (y / 16) as usize;
            let section_y = y % 16;
            self.sections[section_idx].get_block(x, section_y, z)
        } else {
            Block::Air
        }
    }

    /// Encodes the chunk into NBT format for sending to clients.
    /// Returns a Vec<u8> containing the NBT data.
    pub fn encode_nbt(&self) -> Vec<u8> {
        let mut root = CompoundTag::new();
        root.insert("xPos".to_string(), Value::Int(self.x));
        root.insert("zPos".to_string(), Value::Int(self.z));

        // Encode sections
        let mut sections_nbt = Vec::new();
        for (i, section) in self.sections.iter().enumerate() {
            // Only include non-empty sections
            // (for simplicity, always include for now)
            sections_nbt.push(Value::Compound(section.encode_nbt(i as u8)));
        }
        root.insert("Sections".to_string(), Value::List(sections_nbt));

        to_bytes(&root).unwrap_or_default()
    }
}

// Example module, delete later (MARK_DELETE)
pub mod example {
    use std::sync::Arc;

    use crate::{blocks::Block, chunk::{Chunk, ChunkGenerator}};

    pub struct SinewaveGenerator;

    impl ChunkGenerator for SinewaveGenerator {
        fn generate_chunk(&self, chunk: Arc<Chunk>) {
            let (chunk_x, chunk_z) = chunk.coordinates();

            for x in 0..16 {
                for z in 0..16 {
                    let world_x = chunk_x * 16 + x;
                    let world_z = chunk_z * 16 + z;

                    let height = (40.0 + (world_x as f64 * 0.1).sin() * (world_z as f64 * 0.1).cos() * 10.0) as u8;

                    for y in 0..=height {
                        chunk.set_block(x, y, z, Block::GrassBlock);
                    }
                }
            }
        }
    }
}