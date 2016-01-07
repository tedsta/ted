use std::collections::VecDeque;
use std::io;

use cursor::Cursor;
use net;
use operation::Operation;
use ted::Ted;
use ted_server::{PacketId, Request, Response};

pub struct TedClient {
    pub client: net::Client,

    timeline: Vec<Operation>,
    last_sync: usize,

    pending_queue: usize,

    op_queue: usize, // Start index in ted.log of ops that need to be sent to the server
    cmd_queue: usize,
}

impl TedClient {
    pub fn new(client: net::Client) -> TedClient {
        TedClient {
            client: client,
            timeline: Vec::new(),
            last_sync: 0,
            pending_queue: 0,
            op_queue: 0,
            cmd_queue: 0,
        }
    }
    
    pub fn download_buffer(&mut self) -> Result<Ted, String> {
        let mut packet = self.client.receive();
        let buffer: String = try!(packet.read().map_err(|e| e.to_string()));
        let timeline: Vec<(net::ClientId, Operation)> =
            try!(packet.read().map_err(|e| e.to_string()));
        self.timeline = timeline.into_iter().map(|(_, op)| op).collect();

        self.last_sync = self.timeline.len();

        Ok(Ted::from_string(1, buffer))
    }

    pub fn send_operations(&mut self, ted: &mut Ted) {
        for op in &ted.log[self.op_queue..] {
            // TODO: Optimize, I shouldn't have to clone
            self.send_operation(op.clone());
        }
        self.op_queue = ted.log.len();
    }

    pub fn send_commands(&mut self, ted: &mut Ted) {
        for cmd in &ted.cmd_log[self.cmd_queue..] {
            // TODO: I shouldn't have to clone
            self.send_command(cmd.clone());
        }
        self.op_queue = ted.log.len();
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
            let op = packet.read().unwrap();
            self.timeline.push(op);
        }

        self.merge_synced_ops(ted);

        self.last_sync = self.timeline.len();
    }

    fn send_operation(&mut self, op: Operation) {
        let mut packet = net::OutPacket::new();
        packet.write(&Request::Op(self.timeline.len() as u64, op)).unwrap();
        self.client.send(&packet);
    }

    fn send_command(&mut self, cmd: String) {
        let mut packet = net::OutPacket::new();
        packet.write(&Request::Command(self.timeline.len() as u64, cmd)).unwrap();
        self.client.send(&packet);
    }

    fn cursor_moved(&mut self, cursor: &Cursor) {
    }

    fn merge_synced_ops(&mut self, ted: &mut Ted) {
        // TODO: This whole function could be optimized to merge the ops without undoing and
        // redoing the pending operations
        
        let ops = &self.timeline[self.last_sync..];

        // If there's nothing to merge...
        if ops.is_empty() { return; }

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
