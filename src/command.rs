use buffer::Cursor;

#[derive(Clone, PartialEq, Eq)]
pub enum Command {
    InsertLines(usize, Vec<String>),
    Insert(Cursor, String),
    Remove(Cursor, Cursor, String),
    RemoveLines(usize, usize, Vec<String>),
}

impl Command {
    pub fn inverse(self) -> Command {
        use self::Command::*;

        match self {
            InsertLines(line_num, lines) => RemoveLines(line_num, line_num+lines.len()-1, lines),
            Insert(cursor, text) => {
                let end = Cursor { line: cursor.line, column: cursor.column+text.len() };
                Remove(cursor, end, text)
            },
            Remove(start, _, text) => Insert(start, text),
            RemoveLines(start, _, lines) => InsertLines(start, lines),
        }
    }
}
