#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Cursor {
    pub line: usize,
    pub column: usize,
}

#[derive(Clone)]
pub struct Buffer {
    lines: Vec<String>,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            lines: Vec::new(),
        }
    }

    pub fn insert(&mut self, cursor: Cursor, text: &str) {
        let ref mut line = self.lines[cursor.line];
        *line = line[..cursor.column].to_string() + text + &line[cursor.column..];
    }

    pub fn insert_line(&mut self, line: usize, text: String) {
        self.lines.insert(line, text);
    }

    /// Remove all characters between the from and to cursors inclusively
    /// Order of from and to cursors does not matter
    pub fn remove_range(&mut self, from: &Cursor, to: &Cursor) {
        // Figure out which cursor is actually the starting point
        let (start, end) =
            if from.line < to.line {
                (from, to)
            } else if to.line < from.line {
                (to, from)
            } else {
                if from.column <= to.column {
                    (from, to)
                } else {
                    (to, from)
                }
            };

        if start.line == end.line {
            // Start and end are on same line, remove characters between them
            self.lines[start.line].drain(start.column..end.column+1);
        } else {
            // Start and end are on different lines
            // On start line, remove all characters after start column
            // On end line, remove all characters before end column
            self.lines[start.line].drain(start.column..);
            self.lines[end.line].drain(..end.column+1);

            // Delete lines between start and end cursors
            self.lines.drain(start.line+1..end.line);
        }
    }
}

#[test]
fn buffer_insert_line_empty() {
    let mut buf = Buffer::new();

    buf.lines = vec![];
    buf.insert_line(0, "asdf".to_string());

    assert!(buf.lines == vec!["asdf".to_string()]);
}

#[test]
fn buffer_insert_line_middle() {
    let mut buf = Buffer::new();

    buf.lines = vec!["helloworld!".to_string(), "bye".to_string()];
    buf.insert_line(1, "asdf".to_string());

    assert!(buf.lines == vec!["helloworld!".to_string(), "asdf".to_string(), "bye".to_string()]);
}

#[test]
fn buffer_insert_line_end() {
    let mut buf = Buffer::new();

    buf.lines = vec!["helloworld!".to_string(), "bye".to_string()];
    buf.insert_line(2, "asdf".to_string());

    assert!(buf.lines == vec!["helloworld!".to_string(), "bye".to_string(), "asdf".to_string()]);
}

#[test]
fn buffer_insert() {
    let mut buf = Buffer::new();
    let cursor = Cursor { line: 0, column: 5 };

    buf.lines = vec!["helloworld!".to_string(), "bye".to_string()];
    buf.insert(cursor, ", ");

    assert!(buf.lines == vec!["hello, world!".to_string(), "bye".to_string()]);
}

#[test]
fn buffer_remove_range_same_line() {
    let mut buf = Buffer::new();
    let from = Cursor { line: 0, column: 5 };
    let to = Cursor { line: 0, column: 6 };

    buf.lines = vec!["hello, world!".to_string(), "bye".to_string()];
    buf.remove_range(&from, &to);

    assert!(buf.lines == vec!["helloworld!".to_string(), "bye".to_string()]);
}

#[test]
fn buffer_remove_range_two_lines() {
    let mut buf = Buffer::new();
    let from = Cursor { line: 0, column: 5 };
    let to = Cursor { line: 1, column: 1 };

    buf.lines = vec!["hello, world!".to_string(), "bye".to_string()];
    buf.remove_range(&from, &to);

    assert!(buf.lines == vec!["hello".to_string(), "e".to_string()]);
}

#[test]
fn buffer_remove_range_multi_line() {
    let mut buf = Buffer::new();
    let from = Cursor { line: 0, column: 5 };
    let to = Cursor { line: 2, column: 1 };

    buf.lines = vec!["hello, world!".to_string(), "bye".to_string(), "hola".to_string()];
    buf.remove_range(&from, &to);

    assert!(buf.lines == vec!["hello".to_string(), "la".to_string()]);
}
