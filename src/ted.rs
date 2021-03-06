use std::io::{self, Write};
use std::fs::File;
use std::path::Path;

use buffer::Buffer;
use buffer_operator::BufferOperator;
use cursor::Cursor;
use operation::Operation;

#[derive(Copy, Clone, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Command,
    VisualChar { start: u64 },
    VisualLine { start: u64 },
    VisualBlock { start: u64 },
}

pub enum Event {
    Backspace,
    Enter,
    Esc,
    Char(char),
}

pub struct TedOperation {
    args: Vec<ParameterType>,
}

pub enum ParameterType {
    Position,
}

pub struct Ted {
    path: Option<String>,
    mode: Mode,
    pub scroll: u64,
    pub height: u64,
    pub cursor: Cursor,

    buf_op: BufferOperator,
    cmd_buffer: BufferOperator,

    pub log: Vec<Operation>,
    log_index: usize, // Current position in the log from undoing/redoing

    pub cmd_log: Vec<String>,

    pub dirty: bool,
    running: bool,
}

impl Ted {
    pub fn new(height: u64) -> Ted {
        Ted {
            path: None,
            
            mode: Mode::Normal,
            scroll: 0,
            height: height,
            cursor: Cursor { line: 0, column: 0, buf_index: 0 },

            buf_op: BufferOperator::new(),
            cmd_buffer: BufferOperator::new(),
            
            log: Vec::new(),
            log_index: 0,

            cmd_log: Vec::new(),

            dirty: true,
            running: true,
        }
    }

    pub fn from_string(height: u64, text: String) -> Ted {
        Ted {
            path: None,

            mode: Mode::Normal,
            scroll: 0,
            height: height,
            cursor: Cursor { line: 0, column: 0, buf_index: 0 },

            buf_op: BufferOperator::from_string(text),
            cmd_buffer: BufferOperator::new(),
            
            log: Vec::new(),
            log_index: 0,

            cmd_log: Vec::new(),

            dirty: true,
            running: true,
        }
    }

    pub fn from_file(height: u64, path: String) -> io::Result<Ted> {
        Ok(Ted {
            path: Some(path.clone()),

            mode: Mode::Normal,
            scroll: 0,
            height: height,
            cursor: Cursor { line: 0, column: 0, buf_index: 0 },

            buf_op: try!(BufferOperator::from_file(path)),
            cmd_buffer: BufferOperator::new(),
            
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
            Mode::VisualChar { start: _ } => { self.visual_char_handle_event(e); },
            Mode::VisualLine { start: _ } => { self.visual_line_handle_event(e); },
            Mode::VisualBlock { start: _ } => { self.visual_block_handle_event(e); },
        }
    }

    pub fn execute_command(&mut self, cmd: String) {
        let cmd_split: Vec<String> = cmd.split(" ").map(|s| s.to_owned()).collect();

        if cmd == "q" {
            self.running = false;
        }

        if cmd_split[0] == "w" {
            // Write command
            if cmd_split.len() >= 2 {
                // User supplied a file
                self.path = Some(cmd_split[1].clone());
                self.save(&cmd_split[1]);
            } else {
                // User didn't supply a file to write to
                if let Some(ref path) = self.path {
                    // TODO: Display error message rather than panicing if save fails
                    self.save(path).unwrap();
                }
            }
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
                    },
                    ':' => {
                        self.mode = Mode::Command;
                        self.dirty = true;
                    },
                    'v' => {
                        self.mode = Mode::VisualChar { start: self.cursor.buf_index };
                        self.dirty = true;
                    },
                    'V' => {
                        self.mode = Mode::VisualLine { start: self.cursor.buf_index };
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
                    'b' => {
                        if self.cursor.buf_index > 0 &&
                            (self.buf_op.buffer_bytes()[self.cursor.buf_index as usize - 1] == b' ' ||
                            self.buf_op.buffer_bytes()[self.cursor.buf_index as usize - 1] == b'\n')
                        {
                            self.cursor.buf_index -= 1;
                        }
                        while self.cursor.buf_index > 0 &&
                              self.buf_op.buffer_bytes()[self.cursor.buf_index as usize - 1] != b' ' &&
                              self.buf_op.buffer_bytes()[self.cursor.buf_index as usize - 1] != b'\n' {
                            self.cursor.buf_index -= 1;
                        }
                        self.cursor.calculate_pos(&self.buf_op.buffer());
                        self.dirty = true;
                    },
                    'w' => {
                        if (self.cursor.buf_index as usize) < self.buf_op.buffer_bytes().len() - 1 &&
                            (self.cursor.buf_index == 0 ||
                             ((self.cursor.buf_index as usize) > 0 &&
                              (self.buf_op.buffer_bytes()[self.cursor.buf_index as usize - 1] == b' ' ||
                              self.buf_op.buffer_bytes()[self.cursor.buf_index as usize - 1] == b'\n')))
                        {
                            self.cursor.buf_index += 1;
                        }
                        while (self.cursor.buf_index as usize) < self.buf_op.buffer_bytes().len() - 1 &&
                              (self.cursor.buf_index as usize) > 0 &&
                              self.buf_op.buffer_bytes()[self.cursor.buf_index as usize - 1] != b' ' &&
                              self.buf_op.buffer_bytes()[self.cursor.buf_index as usize - 1] != b'\n' {
                            self.cursor.buf_index += 1;
                        }
                        self.cursor.calculate_pos(&self.buf_op.buffer());
                        self.dirty = true;
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
                    if self.cursor.buf_index == self.buffer()
                                                    .line_info()[self.cursor.line as usize]
                                                    .buf_index as u64 {
                        // Handle special newline case
                        self.cursor.buf_index -= 1;
                        self.cursor.line -= 1;
                        self.cursor.column =
                            self.buffer().line_info()[self.cursor.line as usize].length as u64;
                    } else {
                        self.cursor.move_left(self.buf_op.buffer());
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
                self.cursor_down();
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

    // Command mode handle event
    fn command_handle_event(&mut self, e: Event) {
        match e {
            Event::Backspace => {
                if self.cmd_buffer.buffer().len() > 0 {
                    let end = self.cmd_buffer.buffer().len()-1;
                    self.cmd_buffer.buffer_mut().remove(end, end);
                    self.dirty = true;
                }
            },
            Event::Char(c) => {
                let end = self.cmd_buffer.buffer().len();
                self.cmd_buffer.buffer_mut().insert(end, format!("{}", c).as_str());
                self.dirty = true;
            },
            Event::Enter => {
                let command = self.cmd_buffer.buffer().buffer().clone();
                self.execute_command(command);
                self.cmd_buffer.buffer_mut().clear();
                self.mode = Mode::Normal;
                self.dirty = true;
            },
            Event::Esc => {
                self.mode = Mode::Normal;
                self.dirty = true;
            }
        }
    }

    // Visual character mode
    fn visual_char_handle_event(&mut self, e: Event) {
        match e {
            Event::Esc => {
                self.mode = Mode::Normal;
                self.dirty = true;
            },
            Event::Backspace => { },
            Event::Char(_) => {
            },
            _ => { },
        }
    }

    // Visual line mode
    fn visual_line_handle_event(&mut self, e: Event) {
        match e {
            Event::Esc => {
                self.mode = Mode::Normal;
                self.dirty = true;
            },
            Event::Backspace => { },
            Event::Char(_) => {
            },
            _ => { },
        }
    }

    // Visual block mode
    fn visual_block_handle_event(&mut self, e: Event) {
        match e {
            Event::Esc => {
                self.mode = Mode::Normal;
                self.dirty = true;
            },
            Event::Backspace => { },
            Event::Char(_) => {
            },
            _ => { },
        }
    }

    pub fn save<P: AsRef<Path>>(&self, path: &P) -> io::Result<()> {
        // Open the path in read-only mode, returns `io::Result<File>`
        let mut file = try!(File::create(path));

        file.write_all(&self.buf_op.buffer().buffer().as_bytes())
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

    pub fn buffer(&self) -> &Buffer {
        self.buf_op.buffer()
    }

    pub fn command_buffer(&self) -> &Buffer {
        self.cmd_buffer.buffer()
    }

    pub fn running(&self) -> bool {
        self.running
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty || self.buf_op.dirty
    }

    pub fn clean(&mut self) {
        self.dirty = false;
        self.buf_op.dirty = false;
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////
    // Operation stuff

    pub fn do_operation(&mut self, operation: &Operation) {
        self.buf_op.do_operation(operation);
        self.cursor.op_adjust_cursor(self.buf_op.buffer(), operation);
    }


    ////////////////////////////////////////////////////////////////////////////////////////////////
    // Cursor movement

    fn cursor_up(&mut self) {
        self.cursor.move_up(self.buf_op.buffer());
        if self.scroll > self.cursor.line {
            self.scroll = self.cursor.line;
        }
        self.dirty = true;
    }

    fn cursor_down(&mut self) {
        self.cursor.move_down(self.buf_op.buffer());
        if self.scroll+self.height <= self.cursor.line {
            self.scroll = self.cursor.line - (self.height-1);
        }
        self.dirty = true;
    }

    fn cursor_left(&mut self) {
        self.cursor.move_left(self.buf_op.buffer());
        if self.scroll > self.cursor.line {
            self.scroll = self.cursor.line;
        }
        self.dirty = true;
    }

    fn cursor_right(&mut self) {
        self.cursor.move_right(self.buf_op.buffer());
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
