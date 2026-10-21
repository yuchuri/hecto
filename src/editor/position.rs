pub type Row = usize;
pub type Col = usize;

#[derive(Clone, Copy, Default)]
pub struct Position {
    pub col: Col,
    pub row: Row,
}

impl Position {
    pub const fn saturating_sub(self, rhs: Self) -> Self {
        Self {
            col: self.col.saturating_sub(rhs.col),
            row: self.row.saturating_sub(rhs.row),
        }
    }
}
