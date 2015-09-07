#![feature(convert)]

extern crate bincode;
extern crate rustc_serialize;
extern crate rustbox;

use std::thread::Builder;

use editor::Editor;
use ted_server::TedServer;

mod buffer;
mod cursor;
mod editor;
mod net;
mod operation;
mod ted;
mod ted_client;
mod ted_server;

fn main() {
    // Run our client editor
    Editor::from_string("fooobarrrr\nyumm I like cheese\ncheese loves me too <3.".to_string()).run();
}
