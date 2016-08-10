use std::error::Error;
use std::default::Default;
use std::io;

use rustbox;
use rustbox::{
    Color,
    Key,
    RustBox
};

use time::Duration;

use net;
use ted::{Event, Mode, Ted};
use ted_client::TedClient;

pub struct Editor {
    ted: Ted,
    ted_client: Option<TedClient>,
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
            ted: Ted::new((rust_box.height()-2) as u64),
            ted_client: None,
            rust_box: rust_box,
            left_column: 3,
            right_column: 3,
        }
    }

    pub fn from_string(text: String) -> Editor {
        let rust_box =
            match RustBox::init(Default::default()) {
                Result::Ok(v) => v,
                Result::Err(e) => panic!("Failed to create editor's RustBox: {}", e),
            };

        Editor {
            ted: Ted::from_string((rust_box.height()-2) as u64, text),
            ted_client: None,
            rust_box: rust_box,
            left_column: 3,
            right_column: 3,
        }
    }

    pub fn from_file(path: String) -> io::Result<Editor> {
        let rust_box =
            match RustBox::init(Default::default()) {
                Result::Ok(v) => v,
                Result::Err(e) => panic!("Failed to create editor's RustBox: {}", e),
            };

        let ted = try!(Ted::from_file((rust_box.height()-2) as u64, path));

        Ok(Editor {
            ted: ted,
            ted_client: None,
            rust_box: rust_box,
            left_column: 3,
            right_column: 3,
        })
    }

    pub fn from_server(address: &str) -> Result<Editor, String> {
        let rust_box =
            try!(RustBox::init(Default::default())
                .map_err(|e| format!("Failed to create editor's RustBox: {}", e)));

        let client = net::Client::new(address);
        let mut ted_client = TedClient::new(client);
        let mut ted =
            try!(ted_client.download_buffer()
                           .map_err(|e| format!("Failed to download buffer from server: {}", e)));
        ted.height = (rust_box.height()-2) as u64;

        Ok(Editor {
            ted: ted,
            ted_client: Some(ted_client),
            rust_box: rust_box,
            left_column: 3,
            right_column: 3,
        })
    }

    pub fn run(&mut self) {
        while self.ted.running() {
            if self.ted.is_dirty() {
                // Redraw screen if ted is dirty
                self.present();
            }
            if let Some(e) = self.poll_event() {
                self.ted.handle_event(e);
                if let Some(ref mut ted_client) = self.ted_client {
                    ted_client.send_commands(&mut self.ted);
                    ted_client.send_operations(&mut self.ted);
                }
            }
            if let Some(ref mut ted_client) = self.ted_client {
                while let Ok(mut packet) = ted_client.client.try_receive() {
                    ted_client.handle_packet(&mut self.ted, &mut packet);
                }
            }
        }
    }

    fn present(&mut self) {
        use std::cmp;

        // Clear dirty flag
        self.ted.dirty = false;

        self.rust_box.clear();

        // Draw main text
        let text = self.ted.buffer();
        for i in self.ted.scroll..cmp::min(text.line_count() as u64, self.ted.scroll+self.ted.height) {
            self.rust_box.print(self.left_column, (i - self.ted.scroll) as usize,
                                rustbox::RB_BOLD, Color::White, Color::Default,
                                text.line(i as usize));
        }

        // Draw command
        if self.ted.mode() == Mode::Command {
            if let Some(command) = self.ted.aux_buffer(0) {
                self.rust_box.print(0, (self.ted.height + 1) as usize,
                                    rustbox::RB_BOLD, Color::White, Color::Default, ":");
                self.rust_box.print(1, (self.ted.height + 1) as usize,
                                    rustbox::RB_BOLD, Color::White, Color::Default,
                                    command.buffer().as_str());
            }
        } 

        // Draw editor status 
        match self.ted.mode() {
            Mode::Normal => {
                self.rust_box.print(0, self.ted.height as usize,
                                    rustbox::RB_BOLD, Color::Blue, Color::Default,
                                    "--NORMAL--");
            },
            Mode::Insert => {
                self.rust_box.print(0, self.ted.height as usize,
                                    rustbox::RB_BOLD, Color::Red, Color::Default,
                                    "--INSERT--");
            },
            Mode::Command => {
                self.rust_box.print(0, self.ted.height as usize,
                                    rustbox::RB_BOLD, Color::Green, Color::Default,
                                    "--COMMAND--");
            },
        }

        // Draw the cursor
        let (cursor_x, cursor_y) = self.ted.cursor.get_display_xy(self.ted.buffer());
        self.rust_box.set_cursor((cursor_x as usize + self.left_column) as isize,
                                 (cursor_y - self.ted.scroll) as isize);

        self.rust_box.present();
    }

    fn poll_event(&self) -> Option<Event> {
        match self.rust_box.peek_event(Duration::milliseconds(0), false) {
            Ok(rustbox::Event::KeyEvent(key)) => {
                if let Some(k) = key {
                    match k {
                        Key::Char(c) => { return Some(Event::Char(c)); },
                        Key::Backspace => { return Some(Event::Backspace) },
                        Key::Enter => { return Some(Event::Enter); },
                        Key::Esc => { return Some(Event::Esc); },
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
