use buffer::Buffer;
use command::Command;

pub struct Editor {
    buffers: Vec<Buffer>,
    log: Vec<Command>,
    log_index: usize, // Current position in the log from undoing/redoing
}

impl Editor {
    pub fn new() -> Editor {
        Editor {
            buffers: vec![Buffer::new()],
            log: Vec::new(),
            log_index: 0,
        }
    }

    pub fn do_command(&mut self, command: Command) {
        use command::Command::*;
        
        // Do the command
        match command {
            InsertLines(line_num, ref lines) => {
                self.buffers[0].insert_lines(line_num, lines);
            }
            Insert(cursor, ref text) => {
                self.buffers[0].insert(cursor, text);
            }
            _ => { },
        }
        
        // Log the command
        self.log.truncate(self.log_index+1);
        self.log.push(command);
        self.log_index = self.log.len()-1;
    }
}

#[test]
fn editor_do_command_empty() {
    let mut editor = Editor::new();
    editor.do_command(Command::InsertLines(0, vec!["asdf".to_string()]));

    assert!(editor.log == vec![Command::InsertLines(0, vec!["asdf".to_string()])]);
    assert!(editor.log_index == 0);
}

#[test]
fn editor_do_command_beginning() {
    let mut editor = Editor::new();
    editor.do_command(Command::InsertLines(0, vec!["asdf".to_string()]));
    editor.log_index = 0;
    editor.do_command(Command::InsertLines(0, vec!["hi".to_string()]));

    assert!(editor.log == vec![Command::InsertLines(0, vec!["asdf".to_string()]),
                               Command::InsertLines(0, vec!["hi".to_string()])]);
    assert!(editor.log_index == 1);
}

#[test]
fn editor_do_command_middle() {
    let mut editor = Editor::new();
    editor.do_command(Command::InsertLines(0, vec!["asdf".to_string()]));
    editor.do_command(Command::InsertLines(0, vec!["asdf".to_string()]));
    editor.do_command(Command::InsertLines(0, vec!["asdf".to_string()]));
    editor.log_index = 1;
    editor.do_command(Command::InsertLines(0, vec!["hi".to_string()]));

    assert!(editor.log == vec![Command::InsertLines(0, vec!["asdf".to_string()]),
                               Command::InsertLines(0, vec!["asdf".to_string()]),
                               Command::InsertLines(0, vec!["hi".to_string()])]);
    assert!(editor.log_index == 2);
}

#[test]
fn editor_do_command_end() {
    let mut editor = Editor::new();
    editor.do_command(Command::InsertLines(0, vec!["asdf".to_string()]));
    editor.do_command(Command::InsertLines(0, vec!["hi".to_string()]));

    assert!(editor.log == vec![Command::InsertLines(0, vec!["asdf".to_string()]),
                               Command::InsertLines(0, vec!["hi".to_string()])]);
    assert!(editor.log_index == 1);
}
