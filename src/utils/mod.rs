pub fn bytes_to_word_little_endian(fst: u8, snd: u8) -> u16 {
    return ((snd as u16) << 8) + fst as u16;
}

pub fn word_to_bytes_little_endian(word: u16) -> (u8, u8) {
    return ((word >> 8) as u8, ((word << 8) >> 8) as u8);
}
