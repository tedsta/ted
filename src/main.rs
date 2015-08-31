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
mod ted_server;

fn main() {
    Editor::from_file("src/ted.rs").unwrap().run();

    let mut server = net::Server::new();
    let slot = server.create_slot(); // Create default slot
    let mut ted_server = TedServer::new(slot);

    Builder::new().name("server_master".to_string()).spawn(move || {
        server.listen("localhost:30000");
    }).unwrap();

    Builder::new().name("ted_server".to_string()).spawn(move || {
        ted_server.run();
    }).unwrap();
}
