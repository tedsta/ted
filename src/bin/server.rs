extern crate ted;

use std::thread::Builder;

use ted::net;
use ted::ted_server::TedServer;

fn main() {
    let mut server = net::Server::new();
    let slot = server.create_slot(); // Create default slot
    let mut ted_server = TedServer::new(slot);

    // Start the server engine thing
    Builder::new().name("server_master".to_string()).spawn(move || {
        server.listen("0.0.0.0:3910");
    }).unwrap();

    // Run the ted server
    ted_server.run();
}
