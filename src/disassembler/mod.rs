use std::usize;

#[derive(Debug)]
enum Operation {
    Nop,
    Rlca,
    Rrca,
    Rla,
    Rra,
    Daa,
    Cpl,
    Scf,
    Ccf,
    Ld { dest: Operand, source: Operand },
    Unrecognised,
    //Stop,
}

#[derive(Debug)]
enum R8 {
    B,
    C,
    D,
    E,
    H,
    L,
    Hl,
    A,
}

#[derive(Debug)]
enum R16 {
    Bc,
    De,
    Hl,
    Sp,
}

#[derive(Debug)]
enum R16mem {
    Bc,
    De,
    Hli,
    Hld,
}

#[derive(Debug)]
enum Register {
    R8(R8),
    R16(R16),
    R16mem(R16mem),
}

#[derive(Debug)]
enum Address {
    Register(Register),
    U8(u8),
    U16(u16),
}

#[derive(Debug)]
enum Operand {
    Address(Address),
    Register(Register),
    U8(u8),
    U16(u16),
}

impl From<u8> for R16 {
    fn from(input: u8) -> R16 {
        let masked = apply_mask(input, 0b11111100);
        match masked {
            0b11111100 => return R16::Bc,
            0b11111101 => return R16::De,
            0b11111110 => return R16::Hl,
            0b11111111 => return R16::Sp,
            _ => panic!("This should never happen."),
        }
    }
}

impl From<u8> for R16mem {
    fn from(input: u8) -> R16mem {
        let masked = apply_mask(input, 0b11111100);
        match masked {
            0b11111100 => return R16mem::Bc,
            0b11111101 => return R16mem::De,
            0b11111110 => return R16mem::Hli,
            0b11111111 => return R16mem::Hld,
            _ => panic!("This should never happen."),
        }
    }
}

fn to_u16_litle_endian(fst: u8, snd: u8) -> u16 {
    return ((snd as u16) << 8) + fst as u16;
}

fn apply_mask(input: u8, mask: u8) -> u8 {
    return input | mask;
}

fn apply_mask_equal(input: u8, mask: u8) -> bool {
    return apply_mask(input, mask) == mask;
}

fn block_0(bytes: &[u8]) -> (Operation, usize) {
    assert!(!bytes.is_empty());
    let current = bytes[0];
    match current {
        0b00000000 => return (Operation::Nop, 1),
        0b00000111 => return (Operation::Rlca, 1),
        0b00001111 => return (Operation::Rrca, 1),
        0b00010111 => return (Operation::Rla, 1),
        0b00011111 => return (Operation::Rra, 1),
        0b00100111 => return (Operation::Daa, 1),
        0b00101111 => return (Operation::Cpl, 1),
        0b00110111 => return (Operation::Scf, 1),
        0b00111111 => return (Operation::Ccf, 1),
        _ => (),
    }
    if apply_mask(current, 0b00110000) == 0b00110001 {
        // ld r16, imm16
        let dest = Operand::Register(Register::R16(R16::from((current << 2) >> 6)));
        let source = Operand::U16(to_u16_litle_endian(bytes[1], bytes[2]));
        return (Operation::Ld { dest, source }, 3);
    }
    if apply_mask(current, 0b00110000) == 0b00110010 {
        // ld [r16mem], a
        let dest = Operand::Address(Address::Register(Register::R16mem(R16mem::from(
            (current << 2) >> 6,
        ))));
        let source = Operand::Register(Register::R8(R8::A));
        return (Operation::Ld { dest, source }, 1);
    }
    if apply_mask(current, 0b00110000) == 0b00111010 {
        // ld a, [r16mem]
        let dest = Operand::Register(Register::R8(R8::A));
        let source = Operand::Address(Address::Register(Register::R16mem(R16mem::from(
            (current << 2) >> 6,
        ))));
        return (Operation::Ld { dest, source }, 1);
    }
    if current == 0b00001000 {
        // ld [imm16], sp
        let dest = Operand::Address(Address::U16(to_u16_litle_endian(bytes[1], bytes[2])));
        let source = Operand::Register(Register::R16(R16::Sp));
        return (Operation::Ld { dest, source }, 3);
    }

    return (Operation::Unrecognised, 1);
}

fn next_operation(bytes: &[u8]) -> (Operation, usize) {
    assert!(!bytes.is_empty());
    let current = bytes[0];
    if apply_mask_equal(current, 0b00111111) {
        return block_0(bytes);
    }
    return (Operation::Unrecognised, 1);
}

fn main() {
    let input = [
        0b00001000, 0b00110111, 0b00010001, 0b00010010, 0b00011001, 0b00011111, 0b00110001,
        0b11000001, 0b11111001, 0b00011001, 0b00110010, 0b00110001, 0b11000001, 0b11111001,
        0b00111010, 0b00110111, 0b00010001, 0b00110001, 0b11011101, 0b00011001, 0b00011111,
        0b00110001, 0b11000001, 0b11111001,
    ];
    let mut bytes: &[u8] = &input;
    while !bytes.is_empty() {
        //print!("{:?}\n", bytes);
        let (operation, offset) = next_operation(bytes);
        print!("{:?}\n", operation);
        bytes = &bytes[offset..];
    }
}
