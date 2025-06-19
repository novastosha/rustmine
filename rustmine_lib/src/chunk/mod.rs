
pub const CHUNK_HEIGHT: usize = 256;
pub const CHUNK_WIDTH: usize = 16;

pub const SECTION_HEIGHT: usize = 16;
pub const SECTION_WIDTH: usize = CHUNK_WIDTH;
pub const SECTION_VOLUME: usize = (SECTION_HEIGHT * SECTION_WIDTH * SECTION_WIDTH) as usize;
pub const NUM_SECTIONS: usize = 16;

pub const GLOBAL_BITS_PER_BLOCK: u8 = 15;
pub const MIN_BITS_PER_BLOCK: u8 = 4;
pub const MAX_BITS_PER_BLOCK: u8 = 8;