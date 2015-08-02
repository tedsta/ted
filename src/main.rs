pub struct Cursor {
    pub line: usize,
    pub column: usize,
}

pub struct Buffer {
    lines: Vec<String>,
    cursor: Cursor,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            lines: Vec::new(),
            cursor: Cursor { line: 0, column: 0 },
        }
    }

    pub fn insert(&mut self, text: &str) {
        let ref mut line = self.lines[self.cursor.line];
        *line = line[..self.cursor.column].to_string() + text + &line[self.cursor.column..];
    }
}

fn main() {
    println!("Hello, world!");
}

#[test]
fn buffer_insert() {
    let mut buf = Buffer::new();
    buf.lines = vec!["helloworld!".to_string(), "bye".to_string()];
    buf.cursor.column = 5;
    buf.insert(", ");

    assert!(buf.lines == vec!["hello, world!".to_string(), "bye".to_string()]);
}
