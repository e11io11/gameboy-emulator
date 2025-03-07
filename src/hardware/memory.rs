use std::usize;

use crate::utils::{bytes_to_word_little_endian, word_to_bytes_little_endian};

struct MemoryMap {
    data: Vec<u8>,
}

struct MemoryOutOfBoundsError(u16);

impl MemoryMap {
    fn new(size: usize) -> Self {
        Self {
            data: vec![0; size],
        }
    }

    fn size(&self) -> usize {
        return self.data.len();
    }

    fn read_byte(&self, address: u16) -> Option<u8> {
        if !self.is_inbound_byte(address) {
            return None;
        }
        return Some(self.data[address as usize]);
    }

    fn read_word(&self, address: u16) -> Option<u16> {
        if !self.is_inbound_word(address) {
            return None;
        }
        return Some(bytes_to_word_little_endian(
            self.data[address as usize],
            self.data[address as usize + 1],
        ));
    }

    fn write_byte(&mut self, address: u16, byte: u8) -> Result<(), MemoryOutOfBoundsError> {
        if !self.is_inbound_byte(address) {
            return Err(MemoryOutOfBoundsError(address));
        }
        self.data[address as usize] = byte;
        return Ok(());
    }

    fn write_word(&mut self, address: u16, word: u16) -> Result<(), MemoryOutOfBoundsError> {
        if !self.is_inbound_word(address) {
            return Err(MemoryOutOfBoundsError(address));
        }
        let (fst, snd) = word_to_bytes_little_endian(word);
        self.data[address as usize] = fst;
        self.data[address as usize + 1] = snd;
        return Ok(());
    }

    fn is_inbound_byte(&self, address: u16) -> bool {
        return self.size() > address as usize;
    }

    fn is_inbound_word(&self, address: u16) -> bool {
        return self.size() > (address + 1) as usize;
    }
}
