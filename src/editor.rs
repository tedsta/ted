use buffer::Buffer;
use command::Command;

pub struct Editor {
    buffers: Vec<Buffer>,
    log: Vec<Command>,
}

impl Editor {
    pub fn new() -> Editor {
        Editor {
            buffers: vec![Buffer::new()],
            log: Vec::new(),
        }
    }
}
