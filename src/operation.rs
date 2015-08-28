use cursor::Cursor;

#[derive(Clone, PartialEq, Eq)]
pub enum Operation {
    InsertChar(usize, char),
    Insert(usize, String),
    RemoveChar(usize, char),
    Remove(usize, usize, String),
}

impl Operation {
    pub fn inverse(self) -> Operation {
        use self::Operation::*;

        match self {
            InsertChar(index, c) => RemoveChar(index, c),
            Insert(index, text) => Remove(index, index+text.len()-1, text),
            RemoveChar(index, c) => InsertChar(index, c),
            Remove(start, _, text) => Insert(start, text),
        }
    }
}
