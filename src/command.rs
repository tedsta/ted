use buffer::Cursor;

#[derive(Clone, PartialEq, Eq)]
pub enum Command {
    Insert(usize, String),
    Remove(usize, usize, String),
}

impl Command {
    pub fn inverse(self) -> Command {
        use self::Command::*;

        match self {
            Insert(index, text) => Remove(index, index+text.len()-1, text),
            Remove(start, _, text) => Insert(start, text),
        }
    }
}
