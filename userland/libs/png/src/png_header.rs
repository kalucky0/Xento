use crate::{
    enums::{BitDepth, ChunkType, ColorType, CompressionMethod, FilterMethod, InterlaceMethod},
    Chunk,
};

pub struct PngHeader {
    pub width: u32,
    pub height: u32,
    pub bit_depth: BitDepth,
    pub color_type: ColorType,
    pub compression_method: CompressionMethod,
    pub filter_method: FilterMethod,
    pub interlace_method: InterlaceMethod,
}

impl PngHeader {
    pub fn new(chunk: &Chunk) -> Option<PngHeader> {
        let data = chunk.data();

        if chunk.chunk_type() != &ChunkType::ImageHeader || data.len() < 13 {
            return None;
        }

        Some(PngHeader {
            width: u32::from_be_bytes([data[0], data[1], data[2], data[3]]),
            height: u32::from_be_bytes([data[4], data[5], data[6], data[7]]),
            bit_depth: BitDepth::new(data[8])?,
            color_type: ColorType::new(data[9])?,
            compression_method: CompressionMethod::new(data[10])?,
            filter_method: FilterMethod::new(data[11])?,
            interlace_method: InterlaceMethod::new(data[12])?,
        })
    }
}
