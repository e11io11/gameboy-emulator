/// 0x12, 0x34 => 0x1234
pub fn bytes_to_word_big_endian(fst: u8, snd: u8) -> u16 {
    return ((fst as u16) << 8) + snd as u16;
}

/// 0x1234 => 0x12, 0x34
pub fn word_to_bytes_big_endian(word: u16) -> (u8, u8) {
    return (((word << 8) >> 8) as u8, (word >> 8) as u8);
}

/// 0x12, 0x34 => 0x3412
pub fn bytes_to_word_little_endian(fst: u8, snd: u8) -> u16 {
    return bytes_to_word_big_endian(snd, fst);
}

/// 0x1234 => 0x34, 0x12
pub fn word_to_bytes_little_endian(word: u16) -> (u8, u8) {
    let (fst, snd) = word_to_bytes_big_endian(word);
    return (snd, fst);
}

pub fn get_word_left_byte(word: u16) -> u8 {
    let (fst, _) = word_to_bytes_big_endian(word);
    return fst;
}

pub fn get_word_right_byte(word: u16) -> u8 {
    let (fst, _) = word_to_bytes_big_endian(word);
    return fst;
}

pub fn set_word_left_byte(word: u16, byte: u8) -> u16 {
    let (_, snd) = word_to_bytes_big_endian(word);
    return bytes_to_word_big_endian(byte, snd);
}

pub fn set_word_right_byte(word: u16, byte: u8) -> u16 {
    let (fst, _) = word_to_bytes_big_endian(word);
    return bytes_to_word_big_endian(fst, byte);
}

/// Read the bit subset of interval `[from;to[` from a byte and interprets it as u8
pub fn read_bits_of_byte(byte: u8, from: usize, to: usize) -> u8 {
    assert!(from < 8 && to <= 8 && from <= to);
    return (byte << from) >> (from + (8 - to));
}
