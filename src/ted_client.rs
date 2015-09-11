use std::collections::VecDeque;

use cursor::Cursor;
use net;
use operation::Operation;
use ted::Ted;
use ted_server::{PacketId, Request, Response};

pub struct TedClient {
    pub client: net::Client,

    timeline: Vec<Operation>,
    last_sync: usize,
    sync_queue: Vec<Operation>,

    pending_queue: usize,

    op_queue: usize, // Start index in ted.log of ops that need to be sent to the server
}

impl TedClient {
    pub fn new(client: net::Client) -> TedClient {
        TedClient {
            client: client,
            timeline: Vec::new(),
            last_sync: 0,
            sync_queue: Vec::new(),
            pending_queue: 0,
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
            Response::Op => {
                let op_index = self.pending_queue;
                self.timeline.push(ted.log[op_index].clone());
                self.pending_queue += 1;
                self.last_sync += 1;
            }
        }
    }

    fn handle_sync_packet(&mut self, ted: &mut Ted, packet: &mut net::InPacket) {
        let num_ops: u64 = packet.read().unwrap();
        for _ in 0..num_ops {
            let mut op = packet.read().unwrap();
            self.timeline.push(op);
        }

        self.merge_synced_ops(ted);

        self.last_sync = self.timeline.len();
    }

    fn send_operation(&mut self, op_index: usize, op: Operation) {
        let mut packet = net::OutPacket::new();
        packet.write(&Request::Op(self.timeline.len() as u64, op));
        self.client.send(&packet);
    }

    fn cursor_moved(&mut self, cursor: &Cursor) {
    }

    fn merge_synced_ops(&mut self, ted: &mut Ted) {
        // TODO: This whole function could be optimized to merge the ops without undoing and
        // redoing the pending operations
        
        let ops = &self.timeline[self.last_sync..];

        // Undo operations that are still pending
        for i in (self.pending_queue..ted.log.len()).rev() {
            // TODO: I shouldn't have to clone here
            let inverse = ted.log[i].clone().inverse();
            ted.do_operation(&inverse);
        }
        // Apply operations to merge before the pending operations, adjusting pending operations as
        // necessary
        for op in ops {
            // Iterate backwards so we don't do any unnecessary adjustments if operations are
            // cancelled.
            for i in (self.pending_queue..ted.log.len()).rev() {
                if !op.do_before(&mut ted.log[i]) {
                    // Pending operation was cancelled. Remove it and adjust later pending
                    // operations.
                    for j in i..ted.log.len() {
                    }
                }
            }
            ted.do_operation(op);
        }
        // Redo operations that are still pending
        for i in self.pending_queue..ted.log.len() {
            let op = ted.log[i].clone();
            ted.do_operation(&op);
        }
    }
}
