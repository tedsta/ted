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

    pub fn insert(&mut self, cursor: &Cursor, text: &str) {
        let lines = text.split("\n");
        let mut line_index = cursor.line;

        // Since we reversed it, first line is actually last, so insert this line in place
        {
            let ref mut line = self.lines[line_index];
            *line = line[..cursor.column].to_string() + lines.clone().take(1).next().unwrap() +
                    &line[cursor.column..];
            line_index += 1;
        }

        let count = lines.clone().count();
        for (i, line) in lines.enumerate().skip(1) {
            if i < count-1 {
                // Not at the last line yet
                self.lines.insert(line_index, line.to_string());
                line_index += 1;
            } else {
                // For last line, prepend text to line
                self.lines[line_index] = line.to_string() + &self.lines[line_index];
            }
        }
    }

    pub fn insert_lines(&mut self, line_num: usize, lines: &Vec<String>) {
        for (i, line) in lines.iter().enumerate() {
            self.lines.insert(line_num+i, line.clone());
        }
    }

    /// Remove all characters between the from and to cursors inclusively
    /// Order of from and to cursors does not matter
    pub fn remove_range(&mut self, from: &Cursor, to: &Cursor) -> Vec<String> {
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
            vec![self.lines[start.line].drain(start.column..end.column+1).collect()]
        } else {
            // Start and end are on different lines
            // On start line, remove all characters after start column
            // On end line, remove all characters before end column
            let start_line: String = self.lines[start.line].drain(start.column..).collect();
            let end_line: String = self.lines[end.line].drain(..end.column+1).collect();

            // Delete lines between start and end cursors
            let mut lines: Vec<String> = vec![start_line];
            lines.push_all(self.lines.drain(start.line+1..end.line).collect::<Vec<String>>().as_slice());
            lines.push(end_line);
            lines
        }
    }

    /// Remove a range of lines inclusively
    pub fn remove_lines(&mut self, start_line: usize, end_line: usize) -> Vec<String> {
            self.lines.drain(start_line..end_line+1).collect()
    }
}

#[test]
fn buffer_insert_lines_empty() {
    let mut buf = Buffer::new();

    buf.lines = vec![];
    buf.insert_lines(0, &vec!["asdf".to_string(), "boo".to_string()]);

    assert!(buf.lines == vec!["asdf".to_string(), "boo".to_string()]);
}

#[test]
fn buffer_insert_lines_middle() {
    let mut buf = Buffer::new();

    buf.lines = vec!["helloworld!".to_string(), "bye".to_string()];
    buf.insert_lines(1, &vec!["asdf".to_string(), "boo".to_string()]);

    assert!(buf.lines == vec!["helloworld!".to_string(), "asdf".to_string(), "boo".to_string(), "bye".to_string()]);
}

#[test]
fn buffer_insert_line_end() {
    let mut buf = Buffer::new();

    buf.lines = vec!["helloworld!".to_string(), "bye".to_string()];
    buf.insert_lines(2, &vec!["asdf".to_string()]);

    assert!(buf.lines == vec!["helloworld!".to_string(), "bye".to_string(), "asdf".to_string()]);
}

#[test]
fn buffer_insert_no_lines() {
    let mut buf = Buffer::new();
    let cursor = Cursor { line: 0, column: 5 };

    buf.lines = vec!["helloworld!".to_string(), "bye".to_string()];
    buf.insert(&cursor, ", ");

    assert!(buf.lines == vec!["hello, world!".to_string(), "bye".to_string()]);
}

#[test]
fn buffer_insert_two_lines() {
    let mut buf = Buffer::new();
    let cursor = Cursor { line: 0, column: 5 };

    buf.lines = vec!["hello".to_string(), "bye".to_string()];
    buf.insert(&cursor, "s\nworld\n");

    assert!(buf.lines == vec!["hellos".to_string(), "world".to_string(), "bye".to_string()]);
}

#[test]
fn buffer_remove_range_same_line() {
    let mut buf = Buffer::new();
    let from = Cursor { line: 0, column: 5 };
    let to = Cursor { line: 0, column: 6 };

    buf.lines = vec!["hello, world!".to_string(), "bye".to_string()];
    let removed = buf.remove_range(&from, &to);

    assert!(buf.lines == vec!["helloworld!".to_string(), "bye".to_string()]);
    assert!(removed == vec![", ".to_string()]);
}

#[test]
fn buffer_remove_range_two_lines() {
    let mut buf = Buffer::new();
    let from = Cursor { line: 0, column: 5 };
    let to = Cursor { line: 1, column: 1 };

    buf.lines = vec!["hello, world!".to_string(), "bye".to_string()];
    let removed = buf.remove_range(&from, &to);

    assert!(buf.lines == vec!["hello".to_string(), "e".to_string()]);
    assert!(removed == vec![", world!".to_string(), "by".to_string()]);
}

#[test]
fn buffer_remove_range_multi_line() {
    let mut buf = Buffer::new();
    let from = Cursor { line: 0, column: 5 };
    let to = Cursor { line: 2, column: 1 };

    buf.lines = vec!["hello, world!".to_string(), "bye".to_string(), "hola".to_string()];
    let removed = buf.remove_range(&from, &to);

    assert!(buf.lines == vec!["hello".to_string(), "la".to_string()]);
    assert!(removed == vec![", world!".to_string(), "bye".to_string(), "ho".to_string()]);
}

#[test]
fn buffer_remove_lines() {
    let mut buf = Buffer::new();

    buf.lines = vec!["hello".to_string(), "bye".to_string(), "hola".to_string(), "bob".to_string()];
    let removed = buf.remove_lines(1, 2);

    assert!(buf.lines == vec!["hello".to_string(), "bob".to_string()]);
    assert!(removed == vec!["bye".to_string(), "hola".to_string()]);
}
