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

            buffers: vec![Buffer::from_str(text), Buffer::new()],
            
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
                    'h' => { },
                    'l' => { },
                    'k' => {
                        if self.scroll > 0 { 
                            self.scroll -= 1;
                            self.dirty = true;
                        }
                    },
                    'j' => {
                        if self.scroll+self.height < self.buffers[0].line_count() {
                            self.scroll += 1;
                            self.dirty = true;
                        }
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
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Cursor {
    pub line: usize,
    pub column: usize,
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
