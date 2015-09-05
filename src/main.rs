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
    let mut server = net::Server::new();
    let slot = server.create_slot(); // Create default slot
    let mut ted_server = TedServer::new(slot);

    // Start the server engine thing
    Builder::new().name("server_master".to_string()).spawn(move || {
        server.listen("localhost:3910");
    }).unwrap();

    // Start the ted server
    Builder::new().name("ted_server".to_string()).spawn(move || {
        ted_server.run();
    }).unwrap();

    // Run our client editor
    Editor::from_file("src/ted.rs").unwrap().run();
}
