pub enum InterlaceMethod {
    None,
    Adam7,
}

impl InterlaceMethod {
    pub fn new(value: u8) -> Option<InterlaceMethod> {
        match value {
            0 => Some(InterlaceMethod::None),
            1 => Some(InterlaceMethod::Adam7),
            _ => None,
        }
    }
}