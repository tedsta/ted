use std::borrow::Cow;
use std::cell::Cell;
use std::collections::HashMap;

use buffer_operator::BufferOperator;
use net;
use operation::{OpCoords, Operation};

#[derive(RustcEncodable, RustcDecodable)]
pub enum Request<'a> {
    Op(u64, Cow<'a, Operation>), // Op(client_version, op_id, op)
    Command(u64, Cow<'a, String>), // Command(client_version, op_id, op)
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
    buf_op: BufferOperator,
    
    slot: net::ServerSlot,
    
    timeline: Vec<(net::ClientId, Operation)>,
    client_data: HashMap<net::ClientId, ClientData>,
}

impl TedServer {
    pub fn new(buf_op: BufferOperator, slot: net::ServerSlot) -> TedServer {
        TedServer {
            buf_op: buf_op,

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
                    self.client_data.insert(client_id, ClientData::new(self.timeline.len() as u64));

                    // Send the current buffer and timeline
                    let mut packet: net::OutPacket = net::OutPacket::new();
                    packet.write(&self.buf_op.buffer().buffer()).unwrap();
                    packet.write(&self.timeline).unwrap();
                    self.slot.send(client_id, packet);
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
                self.process_operation(client_id, client_version, op.into_owned());
            },
            Request::Command(client_version, cmd) => {
            },
        }
    }

    fn process_operation(&mut self, client_id: net::ClientId,
                         client_version: u64, mut op: Operation) {
        self.sync_client(client_id);

        {
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

            // Do and send the response
            if op_success {
                self.buf_op.do_operation(&op);
                self.timeline.push((client_id, op));

                let response = Response::Op;
                let mut packet = net::OutPacket::new();
                packet.write(&PacketId::Response).unwrap();
                packet.write(&response).unwrap();
                self.slot.send(client_id, packet);
            }

            client_data.version.set(self.timeline.len() as u64);
        }

        // Sync all the other clients now
        self.sync_all_clients(Some(client_id));
    }

    fn sync_client(&self, client_id: net::ClientId) {
        let client_data = self.client_data.get(&client_id).unwrap();
        self._sync_client(client_id, client_data);
    }

    fn sync_all_clients(&mut self, exclude: Option<net::ClientId>) {
        for (client_id, client_data) in &self.client_data {
            if let Some(exclude) = exclude {
                if exclude == *client_id { continue; }
            }
            self._sync_client(*client_id, client_data);
        }
    }

    fn _sync_client(&self, client_id: net::ClientId, client_data: &ClientData) {
        if client_data.version.get() < self.timeline.len() as u64 {
            let mut packet = net::OutPacket::new();
            packet.write(&PacketId::Sync).unwrap();

            // Write all of the operations that happened since last sync
            let merge_start = client_data.version.get() as usize;
            let merge_end = self.timeline.len();

            println!("syncing {} ops", merge_end-merge_start);
            packet.write(&((merge_end - merge_start) as u64)).unwrap(); // Write the number of operations

            for &(_, ref timeline_op) in &self.timeline[merge_start..merge_end] {
                packet.write(timeline_op).unwrap();
            }

            self.slot.send(client_id, packet);

            client_data.version.set(self.timeline.len() as u64);
        }
    }
}

struct ClientData {
    version: Cell<u64>,
}

impl ClientData {
    fn new(version: u64) -> ClientData {
        ClientData {
            version: Cell::new(version),
        }
    }
}
