use std::io;

use buffer::Buffer;
use buffer_operator::BufferOperator;
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

    buf_op: BufferOperator,

    pub log: Vec<Operation>,
    log_index: usize, // Current position in the log from undoing/redoing

    pub cmd_log: Vec<String>,

    pub dirty: bool,
    running: bool,
}

impl Ted {
    pub fn new(height: u64) -> Ted {
        Ted {
            mode: Mode::Normal,
            scroll: 0,
            height: height,
            cursor: Cursor { line: 0, column: 0, buf_index: 0 },

            buf_op: BufferOperator::new(),
            
            log: Vec::new(),
            log_index: 0,

            cmd_log: Vec::new(),

            dirty: true,
            running: true,
        }
    }

    pub fn from_string(height: u64, text: String) -> Ted {
        Ted {
            mode: Mode::Normal,
            scroll: 0,
            height: height,
            cursor: Cursor { line: 0, column: 0, buf_index: 0 },

            buf_op: BufferOperator::from_string(text),
            
            log: Vec::new(),
            log_index: 0,

            cmd_log: Vec::new(),

            dirty: true,
            running: true,
        }
    }

    pub fn from_file(height: u64, path: String) -> io::Result<Ted> {
        Ok(Ted {
            mode: Mode::Normal,
            scroll: 0,
            height: height,
            cursor: Cursor { line: 0, column: 0, buf_index: 0 },

            buf_op: try!(BufferOperator::from_file(path)),
            
            log: Vec::new(),
            log_index: 0,

            cmd_log: Vec::new(),

            dirty: true,
            running: true,
        })
    }

    pub fn handle_event(&mut self, e: Event) {
        match self.mode {
            Mode::Normal => { self.normal_handle_event(e); },
            Mode::Insert => { self.insert_handle_event(e); },
            Mode::Command => { self.command_handle_event(e); },
        }
    }

    pub fn execute_command(&mut self, cmd: String) {
        if cmd == "q" {
            self.running = false;
        }

        self.cmd_log.push(cmd);
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
                    if self.cursor.buf_index == self.buffer(0)
                                                    .unwrap()
                                                    .line_info()[self.cursor.line as usize]
                                                    .buf_index as u64 {
                        // Handle special newline case
                        self.cursor.buf_index -= 1;
                        self.cursor.line -= 1;
                        self.cursor.column =
                            self.buffer(0).unwrap().line_info()[self.cursor.line as usize].length as u64;
                    } else {
                        self.cursor.move_left(self.buf_op.buffer(0).unwrap());
                    }

                    let index = self.cursor.buf_index;

                    let op = self.buf_op.remove_char(index);
                    self.log(op);
                }
            },
            Event::Enter => {
                let index = self.cursor.buf_index;
                let op = self.buf_op.insert_char(index, '\n');
                self.log(op);
                self.cursor.column = 0;
                self.cursor.line += 1;
                self.cursor.buf_index += 1;
            },
            Event::Char(c) => {
                let index = self.cursor.buf_index;
                let op = self.buf_op.insert_char(index, c);
                self.log(op);
                self.cursor.column += 1;
                self.cursor.buf_index += 1;
            },
        }
    }

    // Operation mode handle event
    fn command_handle_event(&mut self, e: Event) {
        match e {
            Event::Backspace => {
                if self.buffer(1).unwrap().len() > 0 {
                    let end = self.buffer(1).unwrap().len()-1;
                    self.buffer_mut(1).unwrap().remove(end, end);
                    self.dirty = true;
                }
            },
            Event::Char(c) => {
                let end = self.buffer(1).unwrap().len();
                self.buffer_mut(1).unwrap().insert(end, format!("{}", c).as_str());
                self.dirty = true;
            },
            Event::Enter => {
                let command = self.buffer(1).unwrap().buffer().clone();
                self.execute_command(command);
                self.buffer_mut(1).unwrap().clear();
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
        self.buf_op.buffer(index)
    }
    
    pub fn buffer_mut(&mut self, index: usize) -> Option<&mut Buffer> {
        self.buf_op.buffer_mut(index)
    }

    pub fn running(&self) -> bool {
        self.running
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty || self.buf_op.dirty
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////
    // Operation stuff

    pub fn do_operation(&mut self, operation: &Operation) {
        self.buf_op.do_operation(operation);
        self.cursor.op_adjust_cursor(self.buf_op.buffer(0).unwrap(), operation);
    }


    ////////////////////////////////////////////////////////////////////////////////////////////////
    // Cursor movement

    fn cursor_up(&mut self) {
        self.cursor.move_up(self.buf_op.buffer(0).unwrap());
        if self.scroll > self.cursor.line {
            self.scroll = self.cursor.line;
        }
        self.dirty = true;
    }

    fn cursor_down(&mut self) {
        self.cursor.move_down(self.buf_op.buffer(0).unwrap());
        if self.scroll+self.height <= self.cursor.line {
            self.scroll = self.cursor.line - (self.height-1);
        }
        self.dirty = true;
    }

    fn cursor_left(&mut self) {
        self.cursor.move_left(self.buf_op.buffer(0).unwrap());
        if self.scroll > self.cursor.line {
            self.scroll = self.cursor.line;
        }
        self.dirty = true;
    }

    fn cursor_right(&mut self) {
        self.cursor.move_right(self.buf_op.buffer(0).unwrap());
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
