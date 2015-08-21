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
    Char(char),
}

pub struct Ted {
    mode: Mode,

    buffers: Vec<Buffer>,

    log: Vec<Operation>,
    log_index: usize, // Current position in the log from undoing/redoing

    pub dirty: bool,
    running: bool,
}

impl Ted {
    pub fn new() -> Ted {
        Ted {
            mode: Mode::Normal,

            buffers: vec![Buffer::new(), Buffer::new()],
            
            log: Vec::new(),
            log_index: 0,

            dirty: true,
            running: true,
        }
    }

    pub fn from_str(text: &str) -> Ted {
        Ted {
            mode: Mode::Normal,

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
        match e {
            Event::Backspace => { },
            Event::Char(c) => {
                match c {
                    ':' => {
                        self.mode = Mode::Command;
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
            Event::Backspace => { },
            Event::Char(c) => { },
            _ => { },
        }
    }

    // Operation mode handle event
    fn command_handle_event(&mut self, e: Event) {
        match e {
            Event::Backspace => { },
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

#[test]
fn ted_log_command_empty() {
    let mut ted = Ted::new();
    ted.log_command(Operation::Insert(0, "asdf".to_string()));

    assert!(ted.log == vec![Operation::Insert(0, "asdf".to_string())]);
    assert!(ted.log_index == 0);
}

#[test]
fn ted_log_command_beginning() {
    let mut ted = Ted::new();
    ted.log_command(Operation::Insert(0, "asdf".to_string()));
    ted.log_index = 0;
    ted.log_command(Operation::Insert(0, "hi".to_string()));

    assert!(ted.log == vec![Operation::Insert(0, "asdf".to_string()),
                            Operation::Insert(0, "hi".to_string())]);
    assert!(ted.log_index == 1);
}

#[test]
fn ted_log_command_middle() {
    let mut ted = Ted::new();
    ted.log_command(Operation::Insert(0, "asdf".to_string()));
    ted.log_command(Operation::Insert(0, "asdf".to_string()));
    ted.log_command(Operation::Insert(0, "asdf".to_string()));
    ted.log_index = 1;
    ted.log_command(Operation::Insert(0, "hi".to_string()));

    assert!(ted.log == vec![Operation::Insert(0, "asdf".to_string()),
                            Operation::Insert(0, "asdf".to_string()),
                            Operation::Insert(0, "hi".to_string())]);
    assert!(ted.log_index == 2);
}

#[test]
fn ted_log_command_end() {
    let mut ted = Ted::new();
    ted.log_command(Operation::Insert(0, "asdf".to_string()));
    ted.log_command(Operation::Insert(0, "hi".to_string()));

    assert!(ted.log == vec![Operation::Insert(0, "asdf".to_string()),
                            Operation::Insert(0, "hi".to_string())]);
    assert!(ted.log_index == 1);
}
