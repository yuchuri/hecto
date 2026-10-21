use crate::editor::{GraphemeIdx, Row};

#[derive(Clone, Copy, Default)]
pub struct Location {
    pub grapheme_idx: GraphemeIdx,
    pub line_idx: Row,
}
