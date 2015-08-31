use std::collections::HashMap;

use net;
use operation::Operation;
use ted::Ted;

#[derive(RustcEncodable, RustcDecodable)]
pub enum Packet {
    Op(Operation),
}

pub struct TedServer {
    ted: Ted,
    version: u64,
    
    slot: net::ServerSlot,
    
    timeline: Vec<Operation>,
    client_data: HashMap<net::ClientId, ClientData>,
}

impl TedServer {
    pub fn new(slot: net::ServerSlot) -> TedServer {
        TedServer {
            ted: Ted::new(100),
            version: 0,

            slot: slot,

            timeline: Vec::new(),
            client_data: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.slot.receive() {
                net::SlotInMsg::Joined(client_id) => {
                    self.client_data.insert(client_id,
                                            ClientData::new(self.timeline.clone(), self.version));
                },
                net::SlotInMsg::Disconnected(client_id) => {
                    self.client_data.remove(&client_id);
                },
                net::SlotInMsg::ReceivedPacket(client_id, mut packet) => {
                    self.handle_packet(client_id, &mut packet);
                },
            }
        }
    }

    fn handle_packet(&mut self, client_id: net::ClientId, packet: &mut net::InPacket) {
        let packet: Packet = packet.read().unwrap();
        match packet {
            Packet::Op(op) => {
                let client_data = self.client_data.get_mut(&client_id).unwrap();
                client_data.timeline.push(op);
            },
        }
    }
}

struct ClientData {
    timeline: Vec<Operation>,
    version: u64,
}

impl ClientData {
    fn new(timeline: Vec<Operation>, version: u64) -> ClientData {
        ClientData {
            timeline: timeline,
            version: version,
        }
    }
}
