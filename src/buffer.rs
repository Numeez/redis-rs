use crate::protocol;

pub const MAX_SIZE: usize = 32 << 20;

pub struct Buffer {
    data: Vec<u8>,
    start: usize,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            data: vec![],
            start: 0,
        }
    }
    pub fn append(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }
    fn clear(&mut self) {
        self.data.clear();
        self.start = 0;
    }
    pub fn consume(&mut self, n: usize) {
        debug_assert!(n <= self.len());
        self.start += n;
        if self.start == self.data.len() {
            self.clear();
        } else if self.start > self.data.len() / 2 {
            self.data.drain(..self.start);
            self.start = 0;
        }
    }
    pub fn data(&self) -> &[u8] {
        &self.data[self.start..]
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn len(&self) -> usize {
        return self.data.len() - self.start;
    }

    pub fn set_header(&mut self, pos: usize, header: &[u8]) {
        let i = self.start + pos;
        self.data[i..i + 4].copy_from_slice(header);
    }
    pub fn truncate(&mut self, end: usize) {
        self.data.truncate(self.start + end);
    }
}

pub fn response_start(out: &mut Buffer) -> usize {
    let length = out.len();
    out.append(&[0u8; 4]);
    length
}

pub fn response_end(out: &mut Buffer, pos: usize) {
    let mut header_length = out.len() - pos - 4;
    if header_length > MAX_SIZE {
        out.truncate(pos + 4);
        protocol::out_error(out, protocol::ERR_TOO_BIG, b"response is too big");
        header_length = out.len() - pos - 4;
    }
    out.set_header(pos, &(header_length as u32).to_le_bytes());
}
