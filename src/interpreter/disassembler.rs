use crate::hardware::cpu::Register;
use crate::utils::{bytes_to_word_little_endian, get_bits_of_byte, DataSize};

#[derive(Clone, Debug)]
pub enum Operation {
    NOP,
    RLCA,
    RRCA,
    RLA,
    RRA,
    DAA,
    CPL,
    SCF,
    CCF,
    STOP,
    LD(Operand, Operand),
    JR(Operand),
    JRC(Operand, Operand),
    INC(Operand),
    DEC(Operand),
    ADD(Operand, Operand),
}

#[derive(Clone, Debug)]
pub enum Operand {
    Address(Box<Operand>),
    Register(Register),
    Incr(Register),
    Decr(Register),
    Not(Register),
    Byte(u8),
    Word(u16),
}

impl Operand {
    pub fn get_data_size(&self) -> Option<DataSize> {
        use DataSize::*;
        match self {
            Operand::Register(r) => {
                if r.is_byte_register() {
                    return Some(BYTE);
                }
                if r.is_word_register() {
                    return Some(WORD);
                } else {
                    return Some(BIT);
                }
            }
            Operand::Decr(_) | Operand::Incr(_) | Operand::Word(_) => return Some(WORD),
            Operand::Not(_) => return Some(BIT),
            Operand::Byte(_) => return Some(BYTE),
            Operand::Address(_) => return None,
        }
    }
}

#[derive(Debug)]
pub enum DisassemblyError {
    MissingOperand(u8),
    UnrecognisedOperation(u8),
}

/// Must be called with i < 8
fn get_r8(i: u8) -> Operand {
    assert!(i < 8);
    use Register::*;
    match i {
        0 => return Operand::Register(B),
        1 => return Operand::Register(C),
        2 => return Operand::Register(D),
        3 => return Operand::Register(E),
        4 => return Operand::Register(H),
        5 => return Operand::Register(L),
        6 => return Operand::Address(Box::new(Operand::Register(HL))),
        7 => return Operand::Register(A),
        _ => panic!("This should never happen."),
    }
}

/// Must be called with i < 4
fn get_r16(i: u8) -> Operand {
    use Register::*;
    assert!(i < 4);
    match i {
        0 => return Operand::Register(BC),
        1 => return Operand::Register(DE),
        2 => return Operand::Register(HL),
        3 => return Operand::Register(SP),
        _ => panic!("This should never happen."),
    }
}

/// Must be called with i < 4
fn get_r16mem(i: u8) -> Operand {
    use Register::*;
    assert!(i < 4);
    match i {
        0 => return Operand::Register(BC),
        1 => return Operand::Register(DE),
        2 => return Operand::Incr(HL),
        3 => return Operand::Decr(SP),
        _ => panic!("This should never happen."),
    }
}

/// Must be called with i < 4
fn get_cond(i: u8) -> Operand {
    use Register::*;
    assert!(i < 4);
    match i {
        0 => return Operand::Not(FlagZ),
        1 => return Operand::Register(FlagZ),
        2 => return Operand::Not(FlagC),
        3 => return Operand::Register(FlagC),
        _ => panic!("This should never happen."),
    }
}

fn apply_mask(input: u8, mask: u8) -> u8 {
    return input | mask;
}

fn apply_mask_equal(input: u8, mask: u8) -> bool {
    return apply_mask(input, mask) == mask;
}

fn block_0(bytes: &[u8]) -> Result<(Operation, usize), DisassemblyError> {
    // Instructions starting bith bits 00
    assert!(!bytes.is_empty());
    use Operation::*;
    use Register::*;
    let current = bytes[0];
    match current {
        0b00000000 => return Ok((NOP, 1)),
        0b00000111 => return Ok((RLCA, 1)),
        0b00001111 => return Ok((RRCA, 1)),
        0b00010111 => return Ok((RLA, 1)),
        0b00011111 => return Ok((RRA, 1)),
        0b00100111 => return Ok((DAA, 1)),
        0b00101111 => return Ok((CPL, 1)),
        0b00110111 => return Ok((SCF, 1)),
        0b00111111 => return Ok((CCF, 1)),
        0b00010000 => {
            if bytes.len() < 2 {
                return Err(DisassemblyError::MissingOperand(current));
            }
            return Ok((STOP, 1));
        }
        _ => (),
    }
    if apply_mask(current, 0b00110000) == 0b00110001 {
        // ld r16, imm16
        if bytes.len() < 3 {
            return Err(DisassemblyError::MissingOperand(current));
        }
        let dest = get_r16(get_bits_of_byte(current, 2, 4));
        let source = Operand::Word(bytes_to_word_little_endian(bytes[1], bytes[2]));
        return Ok((LD(dest, source), 3));
    }
    if apply_mask(current, 0b00110000) == 0b00110010 {
        // ld [r16mem], a
        let dest = Operand::Address(Box::new(get_r16mem(get_bits_of_byte(current, 2, 4))));
        let source = Operand::Register(A);
        return Ok((LD(dest, source), 1));
    }
    if apply_mask(current, 0b00110000) == 0b00111010 {
        // ld a, [r16mem]
        let dest = Operand::Register(A);
        let source = Operand::Address(Box::new(get_r16mem(get_bits_of_byte(current, 2, 4))));
        return Ok((LD(dest, source), 1));
    }
    if current == 0b00001000 {
        // ld [imm16], sp
        let dest = Operand::Address(Box::new(Operand::Word(bytes_to_word_little_endian(
            bytes[1], bytes[2],
        ))));
        let source = Operand::Register(SP);
        return Ok((LD(dest, source), 3));
    }
    if apply_mask(current, 0b00110000) == 0b00110011 {
        // inc r16
        return Ok((INC(get_r16(get_bits_of_byte(current, 2, 4))), 1));
    }
    if apply_mask(current, 0b00110000) == 0b00111011 {
        // dec r16
        return Ok((DEC(get_r16(get_bits_of_byte(current, 2, 4))), 1));
    }
    if apply_mask(current, 0b00110000) == 0b00111001 {
        // add hl, r16
        let op1 = Operand::Register(HL);
        let op2 = get_r16(get_bits_of_byte(current, 2, 4));
        return Ok((ADD(op1, op2), 1));
    }
    if apply_mask(current, 0b00111000) == 0b00111100 {
        // inc r8
        return Ok((INC(get_r8(get_bits_of_byte(current, 2, 5))), 1));
    }
    if apply_mask(current, 0b00111000) == 0b00111101 {
        // dec r8
        return Ok((DEC(get_r8(get_bits_of_byte(current, 2, 5))), 1));
    }
    if apply_mask(current, 0b00111000) == 0b00111110 {
        // ld r8, imm8
        let dest = get_r8(get_bits_of_byte(current, 2, 5));
        let source = Operand::Byte(bytes[1]);
        return Ok((LD(dest, source), 2));
    }
    if current == 0b00011000 {
        // jr imm8
        let dest = Operand::Byte(bytes[1]);
        return Ok((Operation::JR(dest), 2));
    }
    if apply_mask(current, 0b00011000) == 0b00111000 {
        // jr cond, imm8
        let cond = get_cond(get_bits_of_byte(current, 3, 5));
        let dest = Operand::Byte(bytes[1]);
        return Ok((JRC(cond, dest), 2));
    }

    return Err(DisassemblyError::UnrecognisedOperation(current));
}

pub fn get_operation(bytes: &[u8]) -> Result<(Operation, usize), DisassemblyError> {
    assert!(!bytes.is_empty());
    let current = bytes[0];
    if apply_mask_equal(current, 0b00111111) {
        return block_0(bytes);
    }
    return Err(DisassemblyError::UnrecognisedOperation(current));
}

pub fn disassemble_program(bytes: &[u8]) -> Result<Vec<Operation>, DisassemblyError> {
    let mut operations = vec![];
    let mut head = 0;
    while head < bytes.len() {
        let (operation, offset) = get_operation(&bytes[head..bytes.len()])?;
        operations.push(operation);
        head += offset;
    }
    return Ok(operations);
}
