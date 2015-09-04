use cursor::Cursor;
use net;
use operation::Operation;
use ted::Ted;
use ted_server::{Request, Response};

pub struct TedClient {
    client: net::Client,

    pending: Vec<Option<usize>>,
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
        let response: Response = packet.read().unwrap();
        match response {
            Response::Op(id, success) => {
                if success{
                    self.pending[id as usize] = None;
                } else {
                }
            }
        }
    }

    fn on_operation(&mut self, op: Operation) {
        let op_id = self.next_op_id();

        let mut packet = net::OutPacket::new();
        packet.write(&Request::Op(op_id, op));
        self.client.send(&packet);
    }

    fn cursor_moved(&mut self, cursor: &Cursor) {
    }

    fn next_op_id(&mut self) -> u16 {
        match self.free_ids.pop() {
            Some(id) => id,
            None => {
                let id = self.pending.len() as u16;
                self.pending.push(Some(0));
                id
            }
        }
    }
}
