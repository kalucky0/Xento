use crate::ChunkType;

pub struct Chunk<'a> {
    chunk_type: ChunkType,
    data: &'a [u8],
    crc: u32,
}

impl<'a> Chunk<'a> {
    pub fn new(chunk_type: ChunkType, data: &'a [u8], crc: u32) -> Chunk<'a> {
        Chunk {
            chunk_type,
            data,
            crc,
        }
    }

    pub fn byte_size(&self) -> usize {
        12 + self.data.len()
    }

    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    pub fn data(&self) -> &'a [u8] {
        self.data
    }

    pub fn crc(&self) -> u32 {
        self.crc
    }
}
