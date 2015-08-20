use std::error::Error;
use std::default::Default;

use rustbox;
use rustbox::{
    Color,
    Key,
    RustBox
};

pub enum Event {
    Backspace,
    Char(char),
}

pub struct View {
    rust_box: RustBox,
}

impl View {
    pub fn new() -> View {
        let rust_box =
            match RustBox::init(Default::default()) {
                Result::Ok(v) => v,
                Result::Err(e) => panic!("Failed to create view's RustBox: {}", e),
            };

        rust_box.print(1, 1, rustbox::RB_BOLD, Color::White, Color::Black, "Hello, world!");
        rust_box.print(1, 3, rustbox::RB_BOLD, Color::White, Color::Black, "Press any key to quit");
        rust_box.present();

        View {
            rust_box: rust_box,
        }
    }

    pub fn poll_event(&self) -> Option<Event> {
        match self.rust_box.poll_event(false) {
            Ok(rustbox::Event::KeyEvent(key)) => {
                if let Some(k) = key {
                    match k {
                        Key::Char(c) => { return Some(Event::Char(c)); },
                        Key::Backspace => { return Some(Event::Backspace) },
                        _ => { },
                    }
                }
            },
            Err(e) => panic!("{}", e.description()),
            _ => { },
        }
        None
    }

    pub fn present(&self) {
        self.rust_box.present();
    }
}
