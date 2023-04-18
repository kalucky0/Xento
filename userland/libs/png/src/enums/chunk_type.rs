#[derive(PartialEq)]
pub enum ChunkType {
    Background,
    Gamma,
    ImageData,
    ImageEnd,
    ImageHeader,
    Palette,
    Srgb,
    Transparency,
    Unknown([u8; 4]),
}

impl ChunkType {
    pub fn new(data: &[u8]) -> ChunkType {
        match data {
            [0x62, 0x4B, 0x47, 0x44] => ChunkType::Background,
            [0x67, 0x41, 0x4D, 0x41] => ChunkType::Gamma,
            [0x49, 0x44, 0x41, 0x54] => ChunkType::ImageData,
            [0x49, 0x45, 0x4E, 0x44] => ChunkType::ImageEnd,
            [0x49, 0x48, 0x44, 0x52] => ChunkType::ImageHeader,
            [0x50, 0x4C, 0x54, 0x45] => ChunkType::Palette,
            [0x73, 0x52, 0x47, 0x42] => ChunkType::Srgb,
            [0x74, 0x52, 0x4E, 0x53] => ChunkType::Transparency,
            t => ChunkType::Unknown([t[0], t[1], t[2], t[3]]),
        }
    }
}
