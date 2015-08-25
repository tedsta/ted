use std::error::Error;
use std::default::Default;
use std::fs::File;
use std::io;
use std::io::{
    Read,
    BufReader,
};
use std::path::Path;

use rustbox;
use rustbox::{
    Color,
    Key,
    RustBox
};

use ted::{Event, Mode, Ted};

pub struct Editor {
    ted: Ted,
    rust_box: RustBox,

    left_column: usize,
    right_column: usize,
}

impl Editor {
    pub fn new() -> Editor {
        let rust_box =
            match RustBox::init(Default::default()) {
                Result::Ok(v) => v,
                Result::Err(e) => panic!("Failed to create editor's RustBox: {}", e),
            };

        Editor {
            ted: Ted::new(rust_box.height()-2),
            rust_box: rust_box,
            left_column: 3,
            right_column: 3,
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Editor> {
        // Read the file into file_contents
        let mut file = BufReader::new(try!(File::open(path)));
        let mut file_contents = String::new();
        try!(file.read_to_string(&mut file_contents));

        let rust_box =
            match RustBox::init(Default::default()) {
                Result::Ok(v) => v,
                Result::Err(e) => panic!("Failed to create editor's RustBox: {}", e),
            };

        Ok(Editor {
            ted: Ted::from_string(rust_box.height()-2, file_contents),
            rust_box: rust_box,
            left_column: 3,
            right_column: 3,
        })
    }

    pub fn run(&mut self) {
        while self.ted.running() {
            if self.ted.dirty {
                // Redraw screen if ted is dirty
                self.present();
            }
            if let Some(e) = self.poll_event() {
                self.ted.handle_event(e);
            }
        }
    }

    fn present(&mut self) {
        use std::cmp;

        // Clear dirty flag
        self.ted.dirty = false;

        self.rust_box.clear();

        // Draw main text
        if let Some(text) = self.ted.buffer(0) {
            for i in (self.ted.scroll..cmp::min(text.line_count(), self.ted.scroll+self.ted.height)) {
                self.rust_box.print(self.left_column, i - self.ted.scroll,
                                    rustbox::RB_BOLD, Color::White, Color::Default,
                                    text.line(i));
            }
        }

        // Draw command
        if self.ted.mode() == Mode::Command {
            if let Some(command) = self.ted.buffer(1) {
                self.rust_box.print(0, self.ted.height + 1,
                                    rustbox::RB_BOLD, Color::White, Color::Default, ":");
                self.rust_box.print(1, self.ted.height + 1,
                                    rustbox::RB_BOLD, Color::White, Color::Default,
                                    command.buffer().as_str());
            }
        } 

        // Draw editor status 
        match self.ted.mode() {
            Mode::Normal => {
                self.rust_box.print(0, self.ted.height,
                                    rustbox::RB_BOLD, Color::Red, Color::Default,
                                    "--NORMAL--");
            },
            Mode::Insert => {
                self.rust_box.print(0, self.ted.height,
                                    rustbox::RB_BOLD, Color::Red, Color::Default,
                                    "--INSERT--");
            },
            Mode::Command => {
                self.rust_box.print(0, self.ted.height,
                                    rustbox::RB_BOLD, Color::Red, Color::Default,
                                    "--COMMAND--");
            },
        }

        // Draw the cursor
        let (cursor_x, cursor_y) = self.ted.cursor.get_display_xy(self.ted.buffer(0).unwrap());
        self.rust_box.set_cursor((cursor_x + self.left_column) as isize,
                                 (cursor_y - self.ted.scroll) as isize);

        self.rust_box.present();
    }

    fn poll_event(&self) -> Option<Event> {
        match self.rust_box.poll_event(false) {
            Ok(rustbox::Event::KeyEvent(key)) => {
                if let Some(k) = key {
                    match k {
                        Key::Char(c) => { return Some(Event::Char(c)); },
                        Key::Backspace => { return Some(Event::Backspace) },
                        Key::Enter => { return Some(Event::Enter) },
                        Key::Esc => { return Some(Event::Esc) },
                        _ => { },
                    }
                }
            },
            Err(e) => panic!("{}", e.description()),
            _ => { },
        }
        None
    }
}
