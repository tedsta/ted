use cursor::Cursor;
use net;
use operation::Operation;
use ted::Ted;
use ted_server::{PacketId, Request, Response};

pub struct TedClient {
    pub client: net::Client,

    timeline: Vec<Operation>,

    pending: Vec<usize>,
    free_ids: Vec<u16>,

    op_queue: usize, // Start index in ted.log of ops that need to be sent to the server
}

impl TedClient {
    pub fn new(client: net::Client) -> TedClient {
        TedClient {
            client: client,
            timeline: Vec::new(),
            pending: Vec::new(),
            free_ids: Vec::new(),
            op_queue: 0,
        }
    }

    pub fn update(&mut self, ted: &mut Ted) {
        for i in self.op_queue..ted.log.len() {
            // TODO: Optimize, I shouldn't have to clone
            self.send_operation(i, ted.log[i].clone());
        }
        self.op_queue = ted.log.len();

        while let Ok(mut packet) = self.client.try_receive() {
            self.handle_packet(ted, &mut packet);
        }
    }

    pub fn handle_packet(&mut self, ted: &mut Ted, packet: &mut net::InPacket) {
        let packet_id = packet.read().unwrap();

        match packet_id {
            PacketId::Response => { self.handle_response_packet(ted, packet); },
            PacketId::Sync => { self.handle_sync_packet(ted, packet); },
        }
    }

    fn handle_response_packet(&mut self, ted: &mut Ted, packet: &mut net::InPacket) {
        let response: Response = packet.read().unwrap();
        match response {
            Response::Op(id, new_coords) => {
                let op_index = self.pending[id as usize];
                match new_coords {
                    Some(new_coords) => {
                        ted.log[op_index].set_coords(&new_coords);
                        self.timeline.push(ted.log[op_index].clone());
                    },
                    None => {
                        ted.log.remove(op_index);
                    },
                }
                self.free_pending_op(id);
            }
        }
    }

    fn handle_sync_packet(&mut self, ted: &mut Ted, packet: &mut net::InPacket) {
        let num_ops: u64 = packet.read().unwrap();
        for _ in 0..num_ops {
            let op = packet.read().unwrap();
            ted.do_operation(&op);
            self.timeline.push(op);
        }
    }

    fn send_operation(&mut self, op_index: usize, op: Operation) {
        let op_id = self.queue_op(op_index);

        let mut packet = net::OutPacket::new();
        packet.write(&Request::Op(self.timeline.len() as u64, op_id, op));
        self.client.send(&packet);
    }

    fn cursor_moved(&mut self, cursor: &Cursor) {
    }

    fn queue_op(&mut self, op_index: usize) -> u16 {
        match self.free_ids.pop() {
            Some(id) => {
                self.pending[id as usize] = op_index;
                id
            },
            None => {
                let id = self.pending.len() as u16;
                self.pending.push(op_index);
                id
            }
        }
    }

    fn free_pending_op(&mut self, op_id: u16) {
        self.free_ids.push(op_id);
    }
}
