use std::collections::HashMap;

use net;
use operation::Operation;
use ted::Ted;

#[derive(RustcEncodable, RustcDecodable)]
pub enum Request {
    Op(u16, Operation),
}

#[derive(RustcEncodable, RustcDecodable)]
pub enum Response {
    Op(u16, bool),
}

pub struct TedServer {
    ted: Ted,
    
    slot: net::ServerSlot,
    
    timeline: Vec<Operation>,
    client_data: HashMap<net::ClientId, ClientData>,
}

impl TedServer {
    pub fn new(slot: net::ServerSlot) -> TedServer {
        TedServer {
            ted: Ted::new(100),

            slot: slot,

            timeline: Vec::new(),
            client_data: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.slot.receive() {
                net::SlotInMsg::Joined(client_id) => {
                    self.client_data.insert(client_id, ClientData::new(self.timeline.len() as u64, 0.25));
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
        let packet: Request = packet.read().unwrap();
        match packet {
            Request::Op(op_id, op) => {
                self.process_operation(client_id, op_id, op);
            },
        }
    }

    fn process_operation(&mut self, client_id: net::ClientId, op_id: u16, op: Operation) {
        let client_data = self.client_data.get_mut(&client_id).unwrap();
        let merge_start = client_data.version as usize;
        let merge_end = self.timeline.len();

        let mut op = Some(op);
        for timeline_op in &self.timeline[merge_start..merge_end] {
            op = op.and_then(|op| timeline_op.do_before(op));
            if op.is_none() { break; }
        }

        let response =
            match op {
                Some(op) => {
                    self.timeline.push(op);
                    client_data.version = self.timeline.len() as u64;

                    Response::Op(op_id, true);
                },
                None => {
                    Response::Op(op_id, false);
                },
            };
        
        let mut packet = net::OutPacket::new();
        packet.write(&response);
        self.slot.send(client_id, packet);
    }

    fn sync_client(&mut self, client_id: net::ClientId) {
        let client_data = self.client_data.get_mut(&client_id).unwrap();
        if client_data.version < self.timeline.len() as u64 {
            let mut packet = net::OutPacket::new();

            // Write all of the operations that happened since last sync
            let merge_start = client_data.version as usize;
            let merge_end = self.timeline.len();
            packet.write(&((merge_start - merge_end) as u64)); // Write the number of operations
            for timeline_op in &self.timeline[merge_start..merge_end] {
                packet.write(timeline_op);
            }

            self.slot.send(client_id, packet);
        }
    }
}

struct ClientData {
    version: u64,
    send_rate: f64,
}

impl ClientData {
    fn new(version: u64, send_rate: f64) -> ClientData {
        ClientData {
            version: version,
            send_rate: send_rate,
        }
    }
}