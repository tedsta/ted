use std::collections::HashMap;

use net;
use operation::{OpCoords, Operation};
use ted::Ted;

#[derive(RustcEncodable, RustcDecodable)]
pub enum Request {
    Op(u64, Operation), // Op(client_version, op_id, op)
}

#[derive(RustcEncodable, RustcDecodable)]
pub enum Response {
    Op,
}

#[derive(RustcEncodable, RustcDecodable)]
pub enum PacketId {
    Response,
    Sync,
}

pub struct TedServer {
    ted: Ted,
    
    slot: net::ServerSlot,
    
    timeline: Vec<(net::ClientId, Operation)>,
    client_data: HashMap<net::ClientId, ClientData>,
}

impl TedServer {
    pub fn new(slot: net::ServerSlot) -> TedServer {
        TedServer {
            ted: Ted::from_file(1, "src/ted.rs".to_string()).unwrap(),

            slot: slot,

            timeline: Vec::new(),
            client_data: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.slot.receive() {
                net::SlotInMsg::Joined(client_id) => {
                    println!("Client {} joined", client_id);
                    self.client_data.insert(client_id, ClientData::new(self.timeline.len() as u64, 0.25));
                },
                net::SlotInMsg::Disconnected(client_id) => {
                    println!("Client {} disconnected", client_id);
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
            Request::Op(client_version, op) => {
                self.process_operation(client_id, client_version, op);
            },
        }
    }

    fn process_operation(&mut self, client_id: net::ClientId,
                         client_version: u64, mut op: Operation) {
        self.sync_client(client_id);

        let client_data = self.client_data.get_mut(&client_id).unwrap();
        let merge_start = client_version as usize;
        let merge_end = self.timeline.len();

        println!("Processing operation");

        // Adjust op's coordinates because client may not know what happened since 
        let mut op_success = true;
        for &(op_client_id, ref timeline_op) in &self.timeline[merge_start..merge_end] {
            if op_client_id != client_id {
                op_success = timeline_op.do_before(&mut op);
                if !op_success { break; }
            }
        }

        println!("Adjusted coordinates based on {} prior ops", merge_end-merge_start);
        println!("Op successful? {}", op_success);

        // Send the response
        if op_success {
            let response = Response::Op;
            self.timeline.push((client_id, op));

            let mut packet = net::OutPacket::new();
            packet.write(&PacketId::Response);
            packet.write(&response);
            self.slot.send(client_id, packet);
        }

        client_data.version = self.timeline.len() as u64;
    }

    fn sync_client(&mut self, client_id: net::ClientId) {
        let client_data = self.client_data.get_mut(&client_id).unwrap();
        if client_data.version < self.timeline.len() as u64 {
            let mut packet = net::OutPacket::new();
            packet.write(&PacketId::Sync);

            // Write all of the operations that happened since last sync
            let merge_start = client_data.version as usize;
            let merge_end = self.timeline.len();

            println!("syncing {} ops", merge_end-merge_start);
            packet.write(&((merge_end - merge_start) as u64)); // Write the number of operations

            for &(_, ref timeline_op) in &self.timeline[merge_start..merge_end] {
                packet.write(timeline_op);
            }

            self.slot.send(client_id, packet);

            client_data.version = self.timeline.len() as u64;
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
