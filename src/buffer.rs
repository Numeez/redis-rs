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
}
