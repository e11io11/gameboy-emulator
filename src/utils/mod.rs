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

pub fn endianess_conversion(word: u16) -> u16 {
    return (word >> 8) | (word << 8);
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

/// Read the bit subset of interval `[from;to[` from a byte and interprets it as u8.
/// Indexes are read from left to right, e.g. 0b01234567.
pub fn get_bits_of_byte(byte: u8, from: usize, to: usize) -> u8 {
    assert!(from < 8 && to <= 8 && from <= to);
    return (byte << from) >> (from + (8 - to));
}

/// Read the bit of index from a byte and interprets it as a bool with 0=False, 1=True.
/// Index is read from left to right, e.g. 0b01234567.
pub fn get_bit_of_byte(byte: u8, i: usize) -> bool {
    assert!(i < 8);
    return get_bits_of_byte(byte, i, i + 1) == 1;
}

/// Index is read from left to right, e.g. 0b01234567.
pub fn set_bit_of_byte(byte: u8, i: usize, bit: bool) -> u8 {
    assert!(i < 8);
    if bit {
        return byte | (1 << i);
    }
    return byte & !(1 << i);
}

/// Position is read from right to left, e.g. 0b76543210
pub fn overflow_occured_word(op1: u16, op2: u16, result: u16, position: usize) -> bool {
    assert!(position < 16);
    (op1 ^ op2 ^ result) & (1 << position) != 0
}

/// Position is read from right to left, e.g. 0b76543210
pub fn overflow_occured_byte(op1: u8, op2: u8, result: u8, position: usize) -> bool {
    assert!(position < 8);
    (op1 ^ op2 ^ result) & (1 << position) != 0
}

/// Position is read from right to left, e.g. 0b76543210
pub fn borrow_occurred_word(op1: u16, op2: u16, position: usize) -> bool {
    assert!(position < 16);
    ((!(op1) & op2) & (1 << position)) != 0
}

/// Position is read from right to left, e.g. 0b76543210
pub fn borrow_occurred_byte(op1: u8, op2: u8, position: usize) -> bool {
    assert!(position < 8);
    ((!(op1) & op2) & (1 << position)) != 0
}

#[derive(Debug, PartialEq, Clone)]
pub enum DataSize {
    BYTE = 8,
    WORD = 16,
    BIT = 1,
}

pub fn variant_eq<T>(a: &T, b: &T) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}
