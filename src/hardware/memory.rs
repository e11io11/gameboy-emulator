use std::usize;

use crate::interpreter::ExecutionError;
use crate::interpreter::ExecutionError::MemoryOutOfBoundsError;
use crate::utils::{bytes_to_word_little_endian, word_to_bytes_little_endian};

pub struct MemoryMap {
    data: Vec<u8>,
}

impl MemoryMap {
    pub fn new() -> Self {
        Self {
            data: vec![0; 65536],
        }
    }

    pub fn size(&self) -> usize {
        return self.data.len();
    }

    pub fn read_byte(&self, address: usize) -> Result<u8, ExecutionError> {
        if !self.is_inbound_byte(address) {
            return Err(MemoryOutOfBoundsError(address));
        }
        return Ok(self.data[address]);
    }

    pub fn read_bytes(&self, address: usize, n: usize) -> Result<Vec<u8>, ExecutionError> {
        return (0..n)
            .map(|offset| self.read_byte(address + offset))
            .collect();
    }

    pub fn read_word(&self, address: usize) -> Result<u16, ExecutionError> {
        if !self.is_inbound_word(address) {
            return Err(MemoryOutOfBoundsError(address));
        }
        return Ok(bytes_to_word_little_endian(
            self.data[address],
            self.data[address + 1],
        ));
    }

    pub fn write_byte(&mut self, address: usize, byte: u8) -> Result<(), ExecutionError> {
        if !self.is_inbound_byte(address) {
            return Err(MemoryOutOfBoundsError(address));
        }
        self.data[address] = byte;
        return Ok(());
    }

    pub fn write_word(&mut self, address: usize, word: u16) -> Result<(), ExecutionError> {
        if !self.is_inbound_word(address) {
            return Err(MemoryOutOfBoundsError(address));
        }
        let (fst, snd) = word_to_bytes_little_endian(word);
        self.data[address] = fst;
        self.data[address + 1] = snd;
        return Ok(());
    }

    pub fn write_bytes(&mut self, address: usize, bytes: Vec<u8>) -> Result<(), ExecutionError> {
        for (offset, byte) in bytes.iter().enumerate() {
            self.write_byte(address + offset, *byte)?
        }
        return Ok(());
    }

    pub fn incr_byte(&mut self, address: usize, n: u8) -> Result<(), ExecutionError> {
        self.write_byte(address, self.read_byte(address)?.wrapping_add(n))?;
        return Ok(());
    }

    pub fn decr_byte(&mut self, address: usize, n: u8) -> Result<(), ExecutionError> {
        self.write_byte(address, self.read_byte(address)?.wrapping_sub(n))?;
        return Ok(());
    }

    fn is_inbound_byte(&self, address: usize) -> bool {
        return self.size() > address;
    }

    fn is_inbound_word(&self, address: usize) -> bool {
        return self.size() > (address + 1);
    }
}
