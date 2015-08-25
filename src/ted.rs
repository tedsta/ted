use buffer::Buffer;
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
    pub scroll: usize,
    pub height: usize,
    pub cursor: Cursor,

    buffers: Vec<Buffer>,

    log: Vec<Operation>,
    log_index: usize, // Current position in the log from undoing/redoing

    pub dirty: bool,
    running: bool,
}

impl Ted {
    pub fn new(height: usize) -> Ted {
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
        }
    }

    pub fn from_str(height: usize, text: &str) -> Ted {
        Ted {
            mode: Mode::Normal,
            scroll: 0,
            height: height,
            cursor: Cursor { line: 0, column: 0, buf_index: 0 },

            buffers: vec![Buffer::from_str(text), Buffer::new()],
            
            log: Vec::new(),
            log_index: 0,

            dirty: true,
            running: true,
        }
    }

    pub fn from_string(height: usize, text: String) -> Ted {
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
        }
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
        use std::cmp;
        
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
            Event::Backspace => { },
            Event::Char(c) => { },
            _ => { },
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

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Cursor {
    pub line: usize,
    pub column: usize,
    pub buf_index: usize,
}

impl Cursor {
    /// Moves the cursor up and returns the new index within the buffer
    pub fn move_up(&mut self, buffer: &Buffer) {
        if self.line > 0 {
            self.line -= 1;
        }

        self.calculate_index(buffer);
    }
    
    /// Moves the cursor down and returns the new index within the buffer
    pub fn move_down(&mut self, buffer: &Buffer) {
        if self.line < buffer.line_count()-1 {
            self.line += 1;
        }

        self.calculate_index(buffer);
    }

    /// Moves the cursor left and returns the new index within the buffer
    pub fn move_left(&mut self, buffer: &Buffer) {
        if self.column > 0 {
            // Cursor can move to the left, need to determine if it's past the current line, though.
            if self.column < buffer.line_info()[self.line].length {
                self.column -= 1;
            } else if buffer.line_info()[self.line].length >= 2 {
                self.column = buffer.line_info()[self.line].length - 2;
            }
        } else if self.line > 0 {
            // Cursor is at the beginning of the line, move to previous line
            self.line -= 1;
            if buffer.line_info()[self.line].length > 0 {
                self.column = buffer.line_info()[self.line].length - 1;
            } else {
                self.column = 0;
            }
        }

        self.calculate_index(buffer);
    }

    /// Moves the cursor right and returns the new index within the buffer
    pub fn move_right(&mut self, buffer: &Buffer) {
        if buffer.line_info()[self.line].length > 0 &&
           self.column < buffer.line_info()[self.line].length - 1 {
            // Cursor can move to the right
            self.column += 1;
        } else if self.line < buffer.line_count() - 1 {
            // Cursor can't move right, move to next line
            self.line += 1;
            self.column = 0;
        }

        self.calculate_index(buffer);
    }

    /// Calculates the position to display the cursor at
    pub fn get_display_xy(&self, buffer: &Buffer) -> (usize, usize) {
        use std::cmp;

        let line_info = buffer.line_info()[self.line];

        (self.buf_index - line_info.buf_index , self.line)
    }

    /// Calculates the cursor's index within the specified buffer
    fn calculate_index(&mut self, buffer: &Buffer) {
        use std::cmp;

        let line_info = buffer.line_info()[self.line];
        self.buf_index =
            if line_info.length > 0 {
                line_info.buf_index + cmp::min(line_info.length-1, self.column)
            } else {
                line_info.buf_index
            };
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
