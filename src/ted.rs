use buffer::Buffer;
use command::Command;

pub enum Mode {
    Normal,
    Insert,
    Command,
}

pub enum Event {
    Backspace,
    Char(char),
}

pub struct Ted {
    mode: Mode,

    buffers: Vec<Buffer>,

    log: Vec<Command>,
    log_index: usize, // Current position in the log from undoing/redoing

    dirty: bool,
    running: bool,
}

impl Ted {
    pub fn new() -> Ted {
        Ted {
            mode: Mode::Normal,

            buffers: vec![Buffer::new()],
            
            log: Vec::new(),
            log_index: 0,

            dirty: true,
            running: true,
        }
    }

    pub fn handle_event(&mut self, e: Event) {
        match e {
            Event::Backspace => { },
            Event::Char(c) => { self.running = false; },
        }
    }

    fn log_command(&mut self, command: Command) {
        self.log.truncate(self.log_index+1);
        self.log.push(command);
        self.log_index = self.log.len()-1;
    }

    pub fn running(&self) -> bool {
        self.running
    }
}

#[test]
fn ted_log_command_empty() {
    let mut ted = Ted::new();
    ted.log_command(Command::Insert(0, "asdf".to_string()));

    assert!(ted.log == vec![Command::Insert(0, "asdf".to_string())]);
    assert!(ted.log_index == 0);
}

#[test]
fn ted_log_command_beginning() {
    let mut ted = Ted::new();
    ted.log_command(Command::Insert(0, "asdf".to_string()));
    ted.log_index = 0;
    ted.log_command(Command::Insert(0, "hi".to_string()));

    assert!(ted.log == vec![Command::Insert(0, "asdf".to_string()),
                               Command::Insert(0, "hi".to_string())]);
    assert!(ted.log_index == 1);
}

#[test]
fn ted_log_command_middle() {
    let mut ted = Ted::new();
    ted.log_command(Command::Insert(0, "asdf".to_string()));
    ted.log_command(Command::Insert(0, "asdf".to_string()));
    ted.log_command(Command::Insert(0, "asdf".to_string()));
    ted.log_index = 1;
    ted.log_command(Command::Insert(0, "hi".to_string()));

    assert!(ted.log == vec![Command::Insert(0, "asdf".to_string()),
                               Command::Insert(0, "asdf".to_string()),
                               Command::Insert(0, "hi".to_string())]);
    assert!(ted.log_index == 2);
}

#[test]
fn ted_log_command_end() {
    let mut ted = Ted::new();
    ted.log_command(Command::Insert(0, "asdf".to_string()));
    ted.log_command(Command::Insert(0, "hi".to_string()));

    assert!(ted.log == vec![Command::Insert(0, "asdf".to_string()),
                               Command::Insert(0, "hi".to_string())]);
    assert!(ted.log_index == 1);
}