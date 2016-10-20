use std::fs::File;
use std::io;
use std::io::{
    BufReader,
    Read,
    Write,
};

use buffer::Buffer;
use operation::Operation;

pub struct BufferOperator {
    buffer: Buffer,

    pub dirty: bool,
    pub file_path: Option<String>,
}

impl BufferOperator {
    pub fn new() -> BufferOperator {
        BufferOperator {
            buffer: Buffer::new(),

            dirty: true,
            file_path: None,
        }
    }

    pub fn from_string(text: String) -> BufferOperator {
        BufferOperator {
            buffer: Buffer::from_string(text),

            dirty: true,
            file_path: None,
        }
    }

    pub fn from_file(path: String) -> io::Result<BufferOperator> {
        // Read the file into file_contents
        let mut file = BufReader::new(try!(File::open(path.as_str())));
        let mut file_contents = String::new();
        try!(file.read_to_string(&mut file_contents));

        Ok(BufferOperator {
            buffer: Buffer::from_string(file_contents),

            dirty: true,
            file_path: Some(path),
        })
    }

    pub fn write_file(&self) -> Result<(), String> {
        match self.file_path {
            Some(ref path) => {
                let mut file =
                    try!(File::open(path.as_str())
                            .map_err(|e| format!("Failed to open buffer's file: {}", e)));
                file.write_all(self.buffer.buffer().as_bytes());
                Ok(())
            },
            None => Err("Buffer file_path is None".to_string()),
        }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn buffer_bytes(&self) -> &[u8] {
        &self.buffer.buffer().as_bytes()
    }
    
    pub fn buffer_mut(&mut self) -> &mut Buffer {
        &mut self.buffer
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////
    // Operation stuff

    pub fn do_operation(&mut self, operation: &Operation) {
        match *operation {
            Operation::InsertChar(index, c) => { self.buffer.insert_char(index as usize, c); },
            Operation::Insert(index, ref text) => { self.buffer.insert(index as usize, text.as_str()); },
            Operation::RemoveChar(index, _) => { self.buffer.remove_char(index as usize); },
            Operation::Remove(start, end, _) => { self.buffer.remove(start as usize, end as usize); },
        }
        self.dirty = true;
    }

    pub fn insert_char(&mut self, index: u64, c: char) -> Operation {
        self.dirty = true;
        self.buffer.insert_char(index as usize, c);
        Operation::InsertChar(index, c)
    }

    pub fn insert(&mut self, index: u64, text: String) -> Operation {
        self.dirty = true;
        self.buffer.insert(index as usize, text.as_str());
        Operation::Insert(index, text)
    }

    pub fn remove_char(&mut self, index: u64) -> Operation {
        self.dirty = true;
        let c = self.buffer.remove_char(index as usize);
        Operation::RemoveChar(index, c)
    }

    pub fn remove(&mut self, from: u64, to: u64) -> Operation {
        self.dirty = true;
        let text = self.buffer.remove(from as usize, to as usize);
        Operation::Remove(from, to, text)
    }
}
