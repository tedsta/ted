use cursor::Cursor;
use net;
use operation::Operation;
use ted::Ted;
use ted_server::{PacketId, Request, Response};

pub struct TedClient {
    client: net::Client,

    pending: Vec<usize>,
    free_ids: Vec<u16>,
}

impl TedClient {
    pub fn new(client: net::Client) -> TedClient {
        TedClient {
            client: client,
            pending: Vec::new(),
            free_ids: Vec::new(),
        }
    }

    pub fn handle_packet(&mut self, ted: &mut Ted, packet: &mut net::InPacket) {
        let packet_id = packet.read().unwrap();

        match packet_id {
            PacketId::Response => { self.handle_results_packet(ted, packet); },
            PacketId::Sync => { self.handle_sync_packet(ted, packet); },
        }
    }

    fn handle_results_packet(&mut self, ted: &mut Ted, packet: &mut net::InPacket) {
        let response: Response = packet.read().unwrap();
        match response {
            Response::Op(id, new_coords) => {
                let op_index = self.pending[id as usize];
                match new_coords {
                    Some(new_coords) => {
                        ted.log[op_index].set_coords(&new_coords);
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
        let num_ops = packet.read().unwrap();
        for _ in 0..num_ops {
            let op = packet.read().unwrap();
            ted.do_operation(&op);
        }
    }

    fn on_operation(&mut self, op_index: usize, op: Operation) {
        let op_id = self.queue_op(op_index);

        let mut packet = net::OutPacket::new();
        packet.write(&Request::Op(op_id, op));
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
