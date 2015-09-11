use std::fs::File;
use std::io;
use std::io::{
    Read,
    BufReader,
};
use std::collections::VecDeque;

use buffer::Buffer;
use cursor::Cursor;
use operation::Operation;

#[derive(Copy, Clone, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Command,
}

pub enum Event {
    Backspace,
    Enter,
    Esc,
    Char(char),
}

pub struct Ted {
    mode: Mode,
    pub scroll: u64,
    pub height: u64,
    pub cursor: Cursor,

    buffers: Vec<Buffer>,

    pub log: Vec<Operation>,
    log_index: usize, // Current position in the log from undoing/redoing

    pub dirty: bool,
    running: bool,
    file_path: Option<String>,
}

impl Ted {
    pub fn new(height: u64) -> Ted {
        Ted {
            mode: Mode::Normal,
            scroll: 0,
            height: height,
            cursor: Cursor { line: 0, column: 0, buf_index: 0 },

            buffers: vec![Buffer::new(), Buffer::new()],
            
            log: Vec::new(),
            log_index: 0,

            dirty: true,
            running: true,
            file_path: None,
        }
    }

    pub fn from_string(height: u64, text: String) -> Ted {
        Ted {
            mode: Mode::Normal,
            scroll: 0,
            height: height,
            cursor: Cursor { line: 0, column: 0, buf_index: 0 },

            buffers: vec![Buffer::from_string(text), Buffer::new()],
            
            log: Vec::new(),
            log_index: 0,

            dirty: true,
            running: true,
            file_path: None,
        }
    }

    pub fn from_file(height: u64, path: String) -> io::Result<Ted> {
        // Read the file into file_contents
        let mut file = BufReader::new(try!(File::open(path.as_str())));
        let mut file_contents = String::new();
        try!(file.read_to_string(&mut file_contents));

        Ok(Ted {
            mode: Mode::Normal,
            scroll: 0,
            height: height,
            cursor: Cursor { line: 0, column: 0, buf_index: 0 },

            buffers: vec![Buffer::from_string(file_contents), Buffer::new()],
            
            log: Vec::new(),
            log_index: 0,

            dirty: true,
            running: true,
            file_path: Some(path),
        })
    }

    pub fn handle_event(&mut self, e: Event) {
        match self.mode {
            Mode::Normal => { self.normal_handle_event(e); },
            Mode::Insert => { self.insert_handle_event(e); },
            Mode::Command => { self.command_handle_event(e); },
        }
    }

    pub fn execute_command(&mut self, command: String) {
        if command == "q" {
            self.running = false;
        }
    }

    // Normal mode handle event
    fn normal_handle_event(&mut self, e: Event) {
        match e {
            Event::Backspace => { },
            Event::Char(c) => {
                match c {
                    'i' => {
                        self.mode = Mode::Insert;
                        self.dirty = true;
                    }
                    ':' => {
                        self.mode = Mode::Command;
                        self.dirty = true;
                    },
                    'h' => {
                        self.cursor_left();
                    },
                    'l' => {
                        self.cursor_right();
                    },
                    'k' => {
                        self.cursor_up();
                    },
                    'j' => {
                        self.cursor_down();
                    },
                    _ => { },
                }
            },
            _ => { },
        }
    }

    // Insert mode handle event
    fn insert_handle_event(&mut self, e: Event) {
        match e {
            Event::Esc => {
                self.mode = Mode::Normal;
                self.dirty = true;
            },
            Event::Backspace => {
                if self.cursor.buf_index > 0 {
                    self.cursor.buf_index -= 1;
                    if self.cursor.column == 0 {
                        // Handle special newline case
                        self.cursor.line -= 1;
                        self.cursor.column =
                            self.buffers[0].line_info()[self.cursor.line as usize].length as u64;
                    } else {
                        self.cursor.move_left(&self.buffers[0]);
                    }

                    let index = self.cursor.buf_index;

                    let op = self.remove_char(index);
                    self.log(op);
                    
                    self.dirty = true;
                }
            },
            Event::Enter => {
                let index = self.cursor.buf_index;
                let op = self.insert_char(index, '\n');
                self.log(op);
                self.cursor.column = 0;
                self.cursor.line += 1;
                self.cursor.buf_index += 1;
                self.dirty = true;
            },
            Event::Char(c) => {
                let index = self.cursor.buf_index;
                let op = self.insert_char(index, c);
                self.log(op);
                self.cursor.column += 1;
                self.cursor.buf_index += 1;
                self.dirty = true;
            },
        }
    }

    // Operation mode handle event
    fn command_handle_event(&mut self, e: Event) {
        match e {
            Event::Backspace => {
                if self.buffers[1].len() > 0 {
                    let end = self.buffers[1].len()-1;
                    self.buffers[1].remove(end, end);
                    self.dirty = true;
                }
            },
            Event::Char(c) => {
                let end = self.buffers[1].len();
                self.buffers[1].insert(end, format!("{}", c).as_str());
                self.dirty = true;
            },
            Event::Enter => {
                let command = self.buffers[1].buffer().clone();
                self.execute_command(command);
                self.buffers[1].clear();
                self.mode = Mode::Normal;
                self.dirty = true;
            },
            Event::Esc => {
                self.mode = Mode::Normal;
                self.dirty = true;
            }
        }
    }

    pub fn log(&mut self, operation: Operation) {
        self.log.truncate(self.log_index+1);
        self.log.push(operation);
        self.log_index = self.log.len()-1;
    }

    pub fn log_entry_mut(&mut self, index: usize) -> &mut Operation {
        &mut self.log[index]
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn buffer(&self, index: usize) -> Option<&Buffer> {
        self.buffers.get(index)
    }
    
    pub fn buffer_mut(&mut self, index: usize) -> Option<&mut Buffer> {
        self.buffers.get_mut(index)
    }

    pub fn running(&self) -> bool {
        self.running
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////
    // Operation stuff

    pub fn do_operation(&mut self, operation: &Operation) {
        self.dirty = true;
        self.cursor.op_adjust_cursor(&self.buffers[0], operation);
        match *operation {
            Operation::InsertChar(index, c) => { self.buffers[0].insert_char(index as usize, c); },
            Operation::Insert(index, ref text) => { self.buffers[0].insert(index as usize, text.as_str()); },
            Operation::RemoveChar(index, _) => { self.buffers[0].remove_char(index as usize); },
            Operation::Remove(start, end, _) => { self.buffers[0].remove(start as usize, end as usize); },
        }
    }

    pub fn insert_char(&mut self, index: u64, c: char) -> Operation {
        self.dirty = true;
        self.buffers[0].insert_char(index as usize, c);
        Operation::InsertChar(index, c)
    }

    pub fn insert(&mut self, index: u64, text: String) -> Operation {
        self.dirty = true;
        self.buffers[0].insert(index as usize, text.as_str());
        Operation::Insert(index, text)
    }

    pub fn remove_char(&mut self, index: u64) -> Operation {
        self.dirty = true;
        let c = self.buffers[0].remove_char(index as usize);
        Operation::RemoveChar(index, c)
    }

    pub fn remove(&mut self, from: u64, to: u64) -> Operation {
        self.dirty = true;
        let text = self.buffers[0].remove(from as usize, to as usize);
        Operation::Remove(from, to, text)
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////
    // Cursor movement

    fn cursor_up(&mut self) {
        self.cursor.move_up(&self.buffers[0]);
        if self.scroll > self.cursor.line {
            self.scroll = self.cursor.line;
        }
        self.dirty = true;
    }

    fn cursor_down(&mut self) {
        self.cursor.move_down(&self.buffers[0]);
        if self.scroll+self.height <= self.cursor.line {
            self.scroll = self.cursor.line - (self.height-1);
        }
        self.dirty = true;
    }

    fn cursor_left(&mut self) {
        self.cursor.move_left(&self.buffers[0]);
        if self.scroll > self.cursor.line {
            self.scroll = self.cursor.line;
        }
        self.dirty = true;
    }

    fn cursor_right(&mut self) {
        self.cursor.move_right(&self.buffers[0]);
        if self.scroll+self.height <= self.cursor.line {
            self.scroll = self.cursor.line - (self.height-1);
        }
        self.dirty = true;
    }
}


#[test]
fn ted_log_empty() {
    let mut ted = Ted::new(0);
    ted.log(Operation::Insert(0, "asdf".to_string()));

    assert!(ted.log == vec![Operation::Insert(0, "asdf".to_string())]);
    assert!(ted.log_index == 0);
}

#[test]
fn ted_log_beginning() {
    let mut ted = Ted::new(0);
    ted.log(Operation::Insert(0, "asdf".to_string()));
    ted.log_index = 0;
    ted.log(Operation::Insert(0, "hi".to_string()));

    assert!(ted.log == vec![Operation::Insert(0, "asdf".to_string()),
                            Operation::Insert(0, "hi".to_string())]);
    assert!(ted.log_index == 1);
}

#[test]
fn ted_log_middle() {
    let mut ted = Ted::new(0);
    ted.log(Operation::Insert(0, "asdf".to_string()));
    ted.log(Operation::Insert(0, "asdf".to_string()));
    ted.log(Operation::Insert(0, "asdf".to_string()));
    ted.log_index = 1;
    ted.log(Operation::Insert(0, "hi".to_string()));

    assert!(ted.log == vec![Operation::Insert(0, "asdf".to_string()),
                            Operation::Insert(0, "asdf".to_string()),
                            Operation::Insert(0, "hi".to_string())]);
    assert!(ted.log_index == 2);
}

#[test]
fn ted_log_end() {
    let mut ted = Ted::new(0);
    ted.log(Operation::Insert(0, "asdf".to_string()));
    ted.log(Operation::Insert(0, "hi".to_string()));

    assert!(ted.log == vec![Operation::Insert(0, "asdf".to_string()),
                            Operation::Insert(0, "hi".to_string())]);
    assert!(ted.log_index == 1);
}
