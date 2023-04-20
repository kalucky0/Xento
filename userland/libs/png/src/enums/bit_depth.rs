#[derive(Clone, Copy)]
pub enum BitDepth {
    One,
    Two,
    Four,
    Eight,
    Sixteen,
}

impl BitDepth {
    pub fn new(value: u8) -> Option<BitDepth> {
        match value {
            0x1 => Some(BitDepth::One),
            0x2 => Some(BitDepth::Two),
            0x4 => Some(BitDepth::Four),
            0x8 => Some(BitDepth::Eight),
            0x10 => Some(BitDepth::Sixteen),
            _ => None,
        }
    }
}