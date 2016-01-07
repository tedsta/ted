#![feature(convert)]
#![feature(drain)]

#[macro_use]
extern crate clap;

extern crate bincode;
extern crate rustc_serialize;
extern crate rustbox;
extern crate time;

use std::thread::Builder;

use buffer_operator::BufferOperator;
use editor::Editor;
use ted::Ted;
use ted_server::TedServer;

mod buffer;
mod buffer_operator;
mod cursor;
mod editor;
mod net;
mod operation;
mod ted;
mod ted_client;
mod ted_server;

fn main() {
	let yml = load_yaml!("cli.yml");
    let m = clap::App::from_yaml(yml).get_matches();

    if let Some(ref matches) = m.subcommand_matches("serve") {
        let buf_op =
            match matches.value_of("file") {
                Some(file) => {
                    println!("Serving {}", file);
                    BufferOperator::from_file(file.to_string()).unwrap()
                },
                None => {
                    println!("Serving new file");
                    BufferOperator::new()
                },
            };
        let mut server = net::Server::new();
        let slot = server.create_slot(); // Create default slot
        let mut ted_server = TedServer::new(buf_op, slot);

        // Start the server engine thing
        Builder::new().name("server_master".to_string()).spawn(move || {
            server.listen("0.0.0.0:3910");
        }).unwrap();

        // Run the ted server
        ted_server.run();
    } else if let Some(ref matches) = m.subcommand_matches("connect") {
        // Run our client editor
        // address is required
        Editor::from_server(matches.value_of("address").unwrap()).unwrap().run();
    } else {
        match m.value_of("file") {
            Some(file_path) => {
                Editor::from_file(file_path.to_string()).unwrap().run();
            },
            None => {
                let mut editor = Editor::new();
                editor.run();
            },
        }
    }
}
