#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Cursor {
    pub line: usize,
    pub column: usize,
}

#[derive(Clone)]
pub struct Buffer {
    buf: String,
    lines: Vec<(usize, usize)>, // [(index, line size)], note '\n' not included in line size
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            buf: "".to_string(),
            lines: Vec::new(),
        }
    }

    pub fn from_str(text: &str) -> Buffer {
        let lines = build_lines(text);
        Buffer {
            buf: text.to_string(),
            lines: lines,
        }
    }

    pub fn insert(&mut self, buf_index: usize, text: &str) {
        self.buf = self.buf[..buf_index].to_string() + text + &self.buf[buf_index..].to_string();

        // TODO: optimize, we don't have to rebuild whole lines array
        self.lines = build_lines(self.buf.as_str());
    }

    /// Remove all characters between the from and to cursors inclusively
    /// Order of from and to cursors does not matter
    pub fn remove(&mut self, from: usize, to: usize) -> String {
        use std::cmp::{min, max};
        
        let (from, to) = (min(from, to), max(from, to));
        
        let removed = self.buf[from..to+1].to_string();
        self.buf = self.buf[..from].to_string() + &self.buf[to+1..].to_string();

        // TODO: optimize, we don't have to rebuild whole lines array
        self.lines = build_lines(self.buf.as_str());

        removed
    }
}

fn build_lines(text: &str) -> Vec<(usize, usize)> {
    let mut lines = Vec::new();
    let mut last_index = 0;
    for (i, c) in text.chars().enumerate() {
        if c == '\n' {
            // Note '\n' not included in line_size
            let line_size = i - last_index;
            lines.push((last_index, line_size));
            last_index = i+1;
        }
    }
    let line_size = text.len() - last_index;
    lines.push((last_index, line_size));
    lines
}

#[test]
fn buffer_insert_no_lines() {
    let mut buf = Buffer::from_str("helloworld!\nbye");
    buf.insert(5, ", ");

    assert!(buf.buf.as_str() == "hello, world!\nbye");
    assert!(buf.lines == vec![(0, 13), (14, 3)]);
}

#[test]
fn buffer_insert_two_lines() {
    let mut buf = Buffer::from_str("hello\nbye");
    buf.insert(5, "s\nworld");

    assert!(buf.buf.as_str() == "hellos\nworld\nbye");
    assert!(buf.lines == vec![(0, 6), (7, 5), (13, 3)]);
}

#[test]
fn buffer_remove_same_line() {
    let mut buf = Buffer::from_str("hello, world!\nbye");
    let removed = buf.remove(5, 6);

    assert!(buf.buf.as_str() == "helloworld!\nbye");
    assert!(removed.as_str() == ", ");
    assert!(buf.lines == vec![(0, 11), (12, 3)]);
}

#[test]
fn buffer_remove_two_lines() {
    let mut buf = Buffer::from_str("hello, world!\nbye");
    let removed = buf.remove(5, 15);

    assert!(buf.buf.as_str() == "helloe");
    assert!(removed.as_str() == ", world!\nby");
    assert!(buf.lines == vec![(0, 6)]);
}

#[test]
fn buffer_remove_multi_line() {
    let mut buf = Buffer::from_str("hello, world!\nbye\nhola");
    let removed = buf.remove(5, 19);

    assert!(buf.buf.as_str() == "hellola");
    assert!(removed.as_str() == ", world!\nbye\nho");
    assert!(buf.lines == vec![(0, 7)]);
}

#[test]
fn build_lines_empty() {
    let lines = build_lines("");

    assert!(lines == vec![(0, 0)]);
}

#[test]
fn build_lines_one() {
    let lines = build_lines("hello");

    assert!(lines == vec![(0, 5)]);
}

#[test]
fn build_lines_two() {
    let lines = build_lines("hello\nfoo");

    assert!(lines == vec![(0, 5), (6, 3)]);
}
