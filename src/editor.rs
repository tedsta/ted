use buffer::Buffer;
use command::Command;
use view::{Event, View};

pub struct Editor {
    view: View,
    buffers: Vec<Buffer>,
    log: Vec<Command>,
    log_index: usize, // Current position in the log from undoing/redoing
    dirty: bool,
}

impl Editor {
    pub fn new() -> Editor {
        Editor {
            view: View::new(),
            buffers: vec![Buffer::new()],
            log: Vec::new(),
            log_index: 0,
            dirty: true,
        }
    }

    pub fn run(&mut self) {
        loop {
            if let Some(e) = self.view.poll_event() {
                match e {
                    Event::Backspace => { },
                    Event::Char(c) => { break; },
                }
            }
            if self.dirty {
                self.view.present();
            }
        }
    }

    fn log_command(&mut self, command: Command) {
        self.log.truncate(self.log_index+1);
        self.log.push(command);
        self.log_index = self.log.len()-1;
    }
}

#[test]
fn editor_log_command_empty() {
    let mut editor = Editor::new();
    editor.log_command(Command::Insert(0, "asdf".to_string()));

    assert!(editor.log == vec![Command::Insert(0, "asdf".to_string())]);
    assert!(editor.log_index == 0);
}

#[test]
fn editor_log_command_beginning() {
    let mut editor = Editor::new();
    editor.log_command(Command::Insert(0, "asdf".to_string()));
    editor.log_index = 0;
    editor.log_command(Command::Insert(0, "hi".to_string()));

    assert!(editor.log == vec![Command::Insert(0, "asdf".to_string()),
                               Command::Insert(0, "hi".to_string())]);
    assert!(editor.log_index == 1);
}

#[test]
fn editor_log_command_middle() {
    let mut editor = Editor::new();
    editor.log_command(Command::Insert(0, "asdf".to_string()));
    editor.log_command(Command::Insert(0, "asdf".to_string()));
    editor.log_command(Command::Insert(0, "asdf".to_string()));
    editor.log_index = 1;
    editor.log_command(Command::Insert(0, "hi".to_string()));

    assert!(editor.log == vec![Command::Insert(0, "asdf".to_string()),
                               Command::Insert(0, "asdf".to_string()),
                               Command::Insert(0, "hi".to_string())]);
    assert!(editor.log_index == 2);
}

#[test]
fn editor_log_command_end() {
    let mut editor = Editor::new();
    editor.log_command(Command::Insert(0, "asdf".to_string()));
    editor.log_command(Command::Insert(0, "hi".to_string()));

    assert!(editor.log == vec![Command::Insert(0, "asdf".to_string()),
                               Command::Insert(0, "hi".to_string())]);
    assert!(editor.log_index == 1);
}
