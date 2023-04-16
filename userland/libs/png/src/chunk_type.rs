pub enum ChunkType {
    Background,
    Gamma,
    ImageData,
    ImageEnd,
    ImageHeader,
    Palette,
    Srgb,
    Transparency,
    Unknown(u32),
}

impl ChunkType {
    pub fn new(data: u32) -> ChunkType {
        match data {
            0x62_4B_47_44 => ChunkType::Background,
            0x67_41_4D_41 => ChunkType::Gamma,
            0x49_44_41_54 => ChunkType::ImageData,
            0x49_45_4E_44 => ChunkType::ImageEnd,
            0x49_48_44_52 => ChunkType::ImageHeader,
            0x50_4C_54_45 => ChunkType::Palette,
            0x73_52_47_42 => ChunkType::Srgb,
            0x74_52_4E_53 => ChunkType::Transparency,
            t => ChunkType::Unknown(t),
        }
    }
}