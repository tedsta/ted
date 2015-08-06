use buffer::Buffer;

pub struct Editor {
    buffers: Vec<Buffer>,
}

impl Editor {
    pub fn new() -> Editor {
        Editor {
            buffers: vec![Buffer::new()],
        }
    }
}
