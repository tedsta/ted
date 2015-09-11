#![feature(convert)]
#![feature(drain)]

#[macro_use]
extern crate clap;

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
	let yml = load_yaml!("cli.yml");
    let m = clap::App::from_yaml(yml).get_matches();

    if let Some(mode) = m.value_of("mode") {
        match mode {
            "fast" => println!("We're really going now!"),
            "slow" => println!("Awwww, too slow :("),
            _      => unreachable!()
        }
    } else {
        println!("--mode <MODE> wasn't used...");
    }

    // Run our client editor
    Editor::from_string("fooobarrrr\nyumm I like cheese\ncheese loves me too <3.".to_string()).run();
}
