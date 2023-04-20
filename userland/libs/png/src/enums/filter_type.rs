pub enum FilterType {
    None,
    Sub,
    Up,
    Average,
    Paeth,
}

impl FilterType {
    pub fn new(value: u8) -> Option<FilterType> {
        match value {
            0 => Some(FilterType::None),
            1 => Some(FilterType::Sub),
            2 => Some(FilterType::Up),
            3 => Some(FilterType::Average),
            4 => Some(FilterType::Paeth),
            _ => None,
        }
    }
}