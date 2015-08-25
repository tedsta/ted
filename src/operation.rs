use cursor::Cursor;

#[derive(Clone, PartialEq, Eq)]
pub enum Operation {
    Insert(usize, String),
    Remove(usize, usize, String),
}

impl Operation {
    pub fn inverse(self) -> Operation {
        use self::Operation::*;

        match self {
            Insert(index, text) => Remove(index, index+text.len()-1, text),
            Remove(start, _, text) => Insert(start, text),
        }
    }
}
