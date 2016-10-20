use std::error::Error;
use std::default::Default;
use std::io;
use std::io::{Write, Stdout, Stdin, stdout};

use termion::{self, AsyncReader, async_stdin, color, cursor, style};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

use time::Duration;

use net;
use ted::{Event, Mode, Ted};
use ted_client::TedClient;

pub struct Editor {
    ted: Ted,
    ted_client: Option<TedClient>,
    stdin: AsyncReader,
    stdout: RawTerminal<Stdout>,
    left_column: usize,
    right_column: usize,
}

impl Editor {
    pub fn new() -> Editor {
        let (_, terminal_height) = termion::terminal_size().unwrap();

        Editor {
            ted: Ted::new((terminal_height-2) as u64),
            ted_client: None,
            stdin: async_stdin(),
            stdout: stdout().into_raw_mode().unwrap(),
            left_column: 3,
            right_column: 3,
        }
    }

    pub fn from_string(text: String) -> Editor {
        let (_, terminal_height) = termion::terminal_size().unwrap();

        Editor {
            ted: Ted::from_string((terminal_height-2) as u64, text),
            ted_client: None,
            stdin: async_stdin(),
            stdout: stdout().into_raw_mode().unwrap(),
            left_column: 3,
            right_column: 3,
        }
    }

    pub fn from_file(path: String) -> io::Result<Editor> {
        let (_, terminal_height) = termion::terminal_size().unwrap();
        let ted = try!(Ted::from_file((terminal_height-2) as u64, path));

        Ok(Editor {
            ted: ted,
            ted_client: None,
            stdin: async_stdin(),
            stdout: stdout().into_raw_mode().unwrap(),
            left_column: 3,
            right_column: 3,
        })
    }

    pub fn from_server(address: &str) -> Result<Editor, String> {
        let (_, terminal_height) = termion::terminal_size().unwrap();
        let client = net::Client::new(address);
        let mut ted_client = TedClient::new(client);
        let mut ted =
            try!(ted_client.download_buffer()
                           .map_err(|e| format!("Failed to download buffer from server: {}", e)));
        ted.height = (terminal_height-2) as u64;

        Ok(Editor {
            ted: ted,
            ted_client: Some(ted_client),
            stdin: async_stdin(),
            stdout: stdout().into_raw_mode().unwrap(),
            left_column: 3,
            right_column: 3,
        })
    }

    pub fn run(&mut self) {
        use termion::input::TermRead;

        while self.ted.running() {
            self.handle_events();
            if let Some(ref mut ted_client) = self.ted_client {
                while let Ok(mut packet) = ted_client.client.try_receive() {
                    ted_client.handle_packet(&mut self.ted, &mut packet);
                }
            }
            if self.ted.is_dirty() {
                // Redraw screen if ted is dirty
                self.present();
            }
        }
    }

    fn present(&mut self) {
        use std::cmp;

        // Clear dirty flag
        self.ted.clean();

        // Clear the screen
        write!(self.stdout, "{}", termion::clear::All);

        // Draw main text
        let text = self.ted.buffer();
        for i in self.ted.scroll..cmp::min(text.line_count() as u64, self.ted.scroll+self.ted.height) {
            write!(self.stdout, "{}{}{}{}{}",
                   cursor::Goto(self.left_column as u16 + 1, (i - self.ted.scroll) as u16 + 1),
                   style::Bold, color::Fg(color::White),
                   color::Bg(color::Reset), text.line(i as usize));
        }

        // Draw command
        if self.ted.mode() == Mode::Command {
            write!(self.stdout, "{}{}{}{}{}",
                   cursor::Goto(1, (self.ted.height + 1) as u16 + 1),
                   style::Bold, color::Fg(color::White),
                   color::Bg(color::Reset), ":");
            write!(self.stdout, "{}{}{}{}{}",
                   cursor::Goto(2, (self.ted.height + 1) as u16 + 1),
                   style::Bold, color::Fg(color::White),
                   color::Bg(color::Reset), self.ted.command_buffer().buffer().as_str());
        } 

        // Draw editor status 
        match self.ted.mode() {
            Mode::Normal => {
                write!(self.stdout, "{}{}{}{}{}",
                       cursor::Goto(1, self.ted.height as u16 + 1),
                       style::Bold, color::Fg(color::Blue),
                       color::Bg(color::Reset), "--NORMAL--");
            },
            Mode::Insert => {
                write!(self.stdout, "{}{}{}{}{}",
                       cursor::Goto(1, self.ted.height as u16 + 1),
                       style::Bold, color::Fg(color::Red),
                       color::Bg(color::Reset), "--INSERT--");
            },
            Mode::Command => {
                write!(self.stdout, "{}{}{}{}{}",
                       cursor::Goto(1, self.ted.height as u16 + 1),
                       style::Bold, color::Fg(color::Green),
                       color::Bg(color::Reset), "--COMMAND--");
            },
            Mode::VisualChar { start: _ } => {
                write!(self.stdout, "{}{}{}{}{}",
                       cursor::Goto(1, self.ted.height as u16 + 1),
                       style::Bold, color::Fg(color::Yellow),
                       color::Bg(color::Reset), "--VISUAL CHARACTER--");
            },
            Mode::VisualLine { start: _ } => {
                write!(self.stdout, "{}{}{}{}{}",
                       cursor::Goto(1, self.ted.height as u16 + 1),
                       style::Bold, color::Fg(color::Yellow),
                       color::Bg(color::Reset), "--VISUAL LINE--");
            },
            Mode::VisualBlock { start: _ } => {
                write!(self.stdout, "{}{}{}{}{}",
                       cursor::Goto(1, self.ted.height as u16 + 1),
                       style::Bold, color::Fg(color::Yellow),
                       color::Bg(color::Reset), "--VISUAL BLOCK--");
            },
        }

        // Draw the cursor
        let (cursor_x, cursor_y) = self.ted.cursor.get_display_xy(self.ted.buffer());
        write!(self.stdout, "{}{}",
               cursor::Goto((cursor_x as usize + self.left_column) as u16 + 1,
                            (cursor_y - self.ted.scroll) as u16 + 1),
               cursor::Show);
        self.stdout.flush().unwrap();
    }

    fn handle_events(&mut self) {
        use std::io::Read;
        use termion::event;

        let mut bytes = [0u8; 64];
        let bytes_read = self.stdin.read(&mut bytes).unwrap();
        let ref mut bytes = bytes[..bytes_read].iter().map(|b| Ok(*b));
        while let Some(b) = bytes.next() {
            let e = event::parse_event(b, bytes).unwrap();
            let k = if let event::Event::Key(k) = e { k } else { continue; };
            let e = match k {
                Key::Char('\n') => { Event::Enter },
                Key::Char('~') => { Event::Esc },
                Key::Char(c) => { Event::Char(c) },
                Key::Backspace => { Event::Backspace },
                //Key::Escape => { Event::Esc },
                _ => { continue; },
            };
            self.ted.handle_event(e);
            if let Some(ref mut ted_client) = self.ted_client {
                ted_client.send_commands(&mut self.ted);
                ted_client.send_operations(&mut self.ted);
            }
        }
    }
}
