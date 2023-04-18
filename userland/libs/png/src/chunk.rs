use crate::enums::ChunkType;

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

    pub fn read(bytes: &[u8]) -> Option<Chunk> {
        if bytes.len() < 4 {
            return None;
        }

        let len = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
        let bytes = &bytes[4..];

        if bytes.len() < len + 8 {
            return None;
        }

        let chunk_type = ChunkType::new(&bytes[..4]);
        let crc = u32::from_be_bytes([
            bytes[len + 4],
            bytes[len + 5],
            bytes[len + 6],
            bytes[len + 7],
        ]);
        let bytes: &[u8] = &bytes[..len + 4];

        // TODO: Check CRC

        Some(Chunk {
            chunk_type,
            data: &bytes[4..],
            crc,
        })
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
