const BUFFER_SIZE: usize = 4096; // 4 KiB

pub struct ProxyBuffer {
    buffer: [u8; BUFFER_SIZE],
    offset: usize, // indicates where sent data ends
    data_len: usize,
}

impl ProxyBuffer {
    pub fn new() -> ProxyBuffer {
        ProxyBuffer {
            buffer: [0u8; BUFFER_SIZE],
            offset: 0,
            data_len: 0,
        }
    }

    pub fn is_full(&self) -> bool {
        self.data_len == BUFFER_SIZE
    }

    pub fn is_empty(&self) -> bool {
        self.data_len == 0
    }

    pub fn clear(&mut self) {
        self.offset = 0;
        self.data_len = 0;
    }

    pub fn get_unsent(&mut self) -> &mut [u8] {
        &mut self.buffer[self.offset..self.data_len]
    }

    pub fn get_available(&mut self) -> &mut [u8] {
        &mut self.buffer[self.data_len..BUFFER_SIZE]
    }

    pub fn advance_offset(&mut self, n: usize) {
        self.offset += n;
        if self.offset == self.data_len {
            self.clear();
        }
    }

    pub fn advance_data_len(&mut self, n: usize) {
        self.data_len += n
    }
}
