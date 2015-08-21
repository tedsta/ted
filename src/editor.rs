use std::error::Error;
use std::default::Default;

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

    scroll: usize,
    height: usize,
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
            ted: Ted::from_str("hello world!!!!!\n    super duper text\n awwwww yeah mann\n"),
            rust_box: rust_box,
            scroll: 0,
            height: 25,
            left_column: 3,
            right_column: 3,
        }
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
            for i in (self.scroll..cmp::min(text.line_count(), self.height)) {
                self.rust_box.print(self.left_column, i - self.scroll,
                                    rustbox::RB_BOLD, Color::White, Color::Black,
                                    text.line(i));
            }
        }

        // Draw command
        if self.ted.mode() == Mode::Command {
            if let Some(command) = self.ted.buffer(1) {
                self.rust_box.print(0, self.height + 1,
                                    rustbox::RB_BOLD, Color::White, Color::Black, ":");
                self.rust_box.print(1, self.height + 1,
                                    rustbox::RB_BOLD, Color::White, Color::Black,
                                    command.buffer().as_str());
            }
        } 

        // Draw editor status 
        match self.ted.mode() {
            Mode::Normal => {
                self.rust_box.print(0, self.height + 2,
                                    rustbox::RB_BOLD, Color::White, Color::Black,
                                    "--NORMAL--  ");
            },
            Mode::Insert => {
                self.rust_box.print(0, self.height + 2,
                                    rustbox::RB_BOLD, Color::White, Color::Black,
                                    "--INSERT--  ");
            },
            Mode::Command => {
                self.rust_box.print(0, self.height + 2,
                                    rustbox::RB_BOLD, Color::White, Color::Black,
                                    "--COMMAND-- ");
            },
        }

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
