use buffer::Cursor;

pub enum Command {
    InsertLine(usize, String),
    Insert(Cursor, String),
    Remove(Cursor, Cursor),
}
