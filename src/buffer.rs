#[derive(Clone)]
pub struct Buffer {
    buf: String,
    line_info: Vec<LineInfo>, // [(index, line size)], note '\n' not included in line size
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            buf: "".to_string(),
            line_info: build_line_info(""),
        }
    }

    pub fn from_string(text: String) -> Buffer {
        let line_info = build_line_info(text.as_str());
        Buffer {
            buf: text,
            line_info: line_info,
        }
    }

    pub fn insert(&mut self, buf_index: usize, text: &str) {
        self.buf = self.buf[..buf_index].to_string() + text + &self.buf[buf_index..].to_string();

        // TODO: optimize, we don't have to rebuild whole line_info array
        self.line_info = build_line_info(self.buf.as_str());
    }

    pub fn insert_char(&mut self, buf_index: usize, c: char) {
        self.buf.insert(buf_index, c);

        // TODO: optimize, we don't have to rebuild whole line_info array
        self.line_info = build_line_info(self.buf.as_str());
    }

    /// Remove all characters between the from and to cursors inclusively
    /// Order of from and to cursors does not matter
    pub fn remove(&mut self, from: usize, to: usize) -> String {
        use std::cmp::{min, max};
        
        let (from, to) = (min(from, to), max(from, to));
        
        let removed = self.buf[from..to+1].to_string();
        self.buf = self.buf[..from].to_string() + &self.buf[to+1..].to_string();

        // TODO: optimize, we don't have to rebuild whole line_info array
        self.line_info = build_line_info(self.buf.as_str());

        removed
    }

    /// Remove a single character from the buffer and return it
    pub fn remove_char(&mut self, index: usize) -> char {
        let c = self.buf.remove(index);

        // TODO: optimize, we don't have to rebuild whole line_info array
        self.line_info = build_line_info(self.buf.as_str());

        c
    }

    /// Read-only access to line_info
    pub fn line_info(&self) -> &Vec<LineInfo> {
        &self.line_info
    }

    pub fn line(&self, index: usize) -> &str {
        let LineInfo { buf_index, length } = self.line_info[index];
        &self.buf[buf_index..buf_index+length]
    }

    pub fn line_count(&self) -> usize {
        self.line_info.len()
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn buffer(&self) -> &String {
        &self.buf
    }

    pub fn clear(&mut self) {
        self.buf.clear();
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct LineInfo {
    pub buf_index: usize,
    pub length: usize,
}

impl LineInfo {
    pub fn last_column(&self) -> usize {
        if self.length > 0 {
            self.length - 1
        } else {
            0
        }
    }
}

fn build_line_info(text: &str) -> Vec<LineInfo> {
    let mut line_info = Vec::new();
    let mut next_line_start = 0;
    for (i, c) in text.chars().enumerate() {
        if c == '\n' {
            // Note '\n' not included in line_size
            let length = i - next_line_start;
            line_info.push(LineInfo { buf_index: next_line_start, length: length });
            next_line_start = i+1;
        }
    }
    let length = text.len() - next_line_start;
    line_info.push(LineInfo { buf_index: next_line_start, length: length });
    line_info
}

#[test]
fn buffer_insert_no_lines() {
    let mut buf = Buffer::from_string("helloworld!\nbye".to_string());
    buf.insert(5, ", ");

    assert!(buf.buf.as_str() == "hello, world!\nbye");
    assert!(buf.line_info == vec![LineInfo { buf_index: 0, length: 13 },
                                  LineInfo { buf_index: 14, length: 3 }]);
}

#[test]
fn buffer_insert_two_lines() {
    let mut buf = Buffer::from_string("hello\nbye".to_string());
    buf.insert(5, "s\nworld");

    assert!(buf.buf.as_str() == "hellos\nworld\nbye");
    assert!(buf.line_info == vec![LineInfo { buf_index: 0, length: 6 },
                                  LineInfo { buf_index: 7, length: 5 },
                                  LineInfo { buf_index: 13, length: 3 }]);
}

#[test]
fn buffer_remove_same_line() {
    let mut buf = Buffer::from_string("hello, world!\nbye".to_string());
    let removed = buf.remove(5, 6);

    assert!(buf.buf.as_str() == "helloworld!\nbye");
    assert!(removed.as_str() == ", ");
    assert!(buf.line_info == vec![LineInfo { buf_index: 0, length: 11 },
                                  LineInfo { buf_index: 12, length: 3 }]);
}

#[test]
fn buffer_remove_two_lines() {
    let mut buf = Buffer::from_string("hello, world!\nbye".to_string());
    let removed = buf.remove(5, 15);

    assert!(buf.buf.as_str() == "helloe");
    assert!(removed.as_str() == ", world!\nby");
    assert!(buf.line_info == vec![LineInfo { buf_index: 0, length: 6 }]);
}

#[test]
fn buffer_remove_multi_line() {
    let mut buf = Buffer::from_string("hello, world!\nbye\nhola".to_string());
    let removed = buf.remove(5, 19);

    assert!(buf.buf.as_str() == "hellola");
    assert!(removed.as_str() == ", world!\nbye\nho");
    assert!(buf.line_info == vec![LineInfo { buf_index: 0, length: 7 }]);
}

#[test]
fn build_line_info_empty() {
    let line_info = build_line_info("");

    assert!(line_info == vec![LineInfo { buf_index: 0, length: 0 }]);
}

#[test]
fn build_line_info_one() {
    let line_info = build_line_info("hello");

    assert!(line_info == vec![LineInfo { buf_index: 0, length: 5 }]);
}

#[test]
fn build_line_info_two() {
    let line_info = build_line_info("hello\nfoo");

    assert!(line_info == vec![LineInfo { buf_index: 0, length: 5 },
                              LineInfo { buf_index: 6, length: 3 }]);
}
