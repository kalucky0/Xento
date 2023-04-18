pub enum FilterMethod {
    Adaptive,
}

impl FilterMethod {
    pub fn new(value: u8) -> Option<FilterMethod> {
        match value {
            0 => Some(FilterMethod::Adaptive),
            _ => None,
        }
    }
}