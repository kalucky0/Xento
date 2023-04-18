pub enum CompressionMethod {
    Deflate,
}

impl CompressionMethod {
    pub fn new(value: u8) -> Option<CompressionMethod> {
        match value {
            0 => Some(CompressionMethod::Deflate),
            _ => None,
        }
    }
}