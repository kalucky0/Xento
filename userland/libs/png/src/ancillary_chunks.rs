use crate::enums::TransparencyChunk;

pub struct AncillaryChunks<'a> {
    palette: Option<&'a [u8]>,
    transparency: Option<TransparencyChunk<'a>>,
    background: Option<&'a [u8]>,
}

impl AncillaryChunks<'_> {
    pub fn new() -> AncillaryChunks<'static> {
        AncillaryChunks {
            palette: None,
            transparency: None,
            background: None,
        }
    }

    pub fn palette(&self) -> Option<&[u8]> {
        self.palette
    }

    pub fn transparency(&self) -> &Option<TransparencyChunk> {
        &self.transparency
    }

    pub fn background(&self) -> Option<&[u8]> {
        self.background
    }
}