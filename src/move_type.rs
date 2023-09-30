use std::fmt;

#[derive(Clone, Copy)]
pub struct Move {
    pub from: u8,
    pub to: u8,
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Move from {} to {}", self.from, self.to)?;
        Ok(())
    }
}
