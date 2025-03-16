use crate::hardware::cpu::Register;
use crate::utils::{bytes_to_word_little_endian, get_bits_of_byte};

#[derive(Clone, Debug)]
pub enum Instruction {
    Unkown(u8),
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
    HALT,
    LdR16Imm16(R16, u16),
    LdR16memA(R16mem),
    LdAR16mem(R16mem),
    LdAddrImm16Sp(u16),
    LdR8Imm8(R8, u8),
    LdR8R8(R8, R8),
    JrImm8(u8),
    JrCondImm8(Cond, u8),
    IncR8(R8),
    IncR16(R16),
    DecR8(R8),
    DecR16(R16),
    AddHlR16(R16),
    AddAR8(R8),
    AdcAR8(R8),
    SubAR8(R8),
    SbcAR8(R8),
    AndAR8(R8),
    XorAR8(R8),
    OrAR8(R8),
    CpAR8(R8),
    AddAImm8(u8),
    AdcAImm8(u8),
    SubAImm8(u8),
    SbcAImm8(u8),
    AndAImm8(u8),
    XorAImm8(u8),
    OrAImm8(u8),
    CpAImm8(u8),
}

impl Instruction {
    pub fn get_size(&self) -> usize {
        use Instruction::*;
        return match self {
            Unkown(..) | NOP | RLCA | RRCA | RLA | RRA | DAA | CPL | SCF | CCF | STOP | HALT
            | AddAR8(..) | AdcAR8(..) | SubAR8(..) | SbcAR8(..) | AndAR8(..) | XorAR8(..)
            | OrAR8(..) | CpAR8(..) | IncR8(..) | IncR16(..) | DecR8(..) | DecR16(..)
            | AddHlR16(..) | LdR16memA(..) | LdAR16mem(..) | LdR8R8(..) => 1,
            AddAImm8(..) | AdcAImm8(..) | SubAImm8(..) | SbcAImm8(..) | AndAImm8(..)
            | XorAImm8(..) | OrAImm8(..) | CpAImm8(..) | LdR8Imm8(..) | JrImm8(..)
            | JrCondImm8(..) => 2,
            LdR16Imm16(..) | LdAddrImm16Sp(..) => 3,
        };
    }
}

#[derive(Debug)]
pub enum DisassemblyError {
    MissingOperand(u8),
    EOF,
    //UnrecognisedInstruction(u8),
}

#[derive(Clone, Debug)]
pub enum R8 {
    B,
    C,
    D,
    E,
    H,
    L,
    AddrHL,
    A,
}

impl From<usize> for R8 {
    fn from(i: usize) -> R8 {
        assert!(i < 8);
        use R8::*;
        return match i {
            0 => B,
            1 => C,
            2 => D,
            3 => E,
            4 => H,
            5 => L,
            6 => AddrHL,
            7 => A,
            _ => panic!("This should never happen."),
        };
    }
}

impl Into<Register> for R8 {
    fn into(self) -> Register {
        use R8::*;
        return match self {
            B => Register::B,
            C => Register::C,
            D => Register::D,
            E => Register::E,
            H => Register::H,
            L => Register::L,
            AddrHL => Register::HL,
            A => Register::A,
        };
    }
}

#[derive(Clone, Debug)]
pub enum R16 {
    BC,
    DE,
    HL,
    SP,
}

impl From<usize> for R16 {
    fn from(i: usize) -> R16 {
        assert!(i < 4);
        use R16::*;
        return match i {
            0 => BC,
            1 => DE,
            2 => HL,
            3 => SP,
            _ => panic!("This should never happen."),
        };
    }
}

impl Into<Register> for R16 {
    fn into(self) -> Register {
        use R16::*;
        return match self {
            BC => Register::BC,
            DE => Register::DE,
            HL => Register::HL,
            SP => Register::SP,
        };
    }
}

#[derive(Clone, Debug)]
pub enum R16mem {
    BC,
    DE,
    IncrHL,
    DecrHL,
}

impl From<usize> for R16mem {
    fn from(i: usize) -> R16mem {
        assert!(i < 4);
        use R16mem::*;
        return match i {
            0 => BC,
            1 => DE,
            2 => IncrHL,
            3 => DecrHL,
            _ => panic!("This should never happen."),
        };
    }
}

impl Into<Register> for R16mem {
    fn into(self) -> Register {
        use R16mem::*;
        return match self {
            BC => Register::BC,
            DE => Register::DE,
            IncrHL => Register::HL,
            DecrHL => Register::HL,
        };
    }
}

#[derive(Clone, Debug)]
pub enum Cond {
    NotZ,
    Z,
    NotC,
    C,
}

impl From<usize> for Cond {
    fn from(i: usize) -> Cond {
        assert!(i < 4);
        use Cond::*;
        return match i {
            0 => NotZ,
            1 => Z,
            2 => NotC,
            3 => C,
            _ => panic!("This should never happen."),
        };
    }
}

impl Into<Register> for Cond {
    fn into(self) -> Register {
        use Cond::*;
        return match self {
            NotZ | Z => Register::FlagZ,
            NotC | C => Register::FlagC,
        };
    }
}

fn apply_mask(input: u8, mask: u8) -> u8 {
    return input | mask;
}

fn apply_mask_equal(input: u8, mask: u8) -> bool {
    return apply_mask(input, mask) == mask;
}

fn get_byte(bytes: &[u8], index: usize) -> Result<u8, DisassemblyError> {
    if bytes.is_empty() {
        return Err(DisassemblyError::EOF);
    }
    return bytes
        .get(index)
        .copied()
        .ok_or(DisassemblyError::MissingOperand(bytes[0]));
}

fn block_0(bytes: &[u8]) -> Result<Instruction, DisassemblyError> {
    // Instructions starting bith bits 00
    use Instruction::*;
    let current = get_byte(bytes, 0)?;
    match current {
        0b00000000 => return Ok(NOP),
        0b00000111 => return Ok(RLCA),
        0b00001111 => return Ok(RRCA),
        0b00010111 => return Ok(RLA),
        0b00011111 => return Ok(RRA),
        0b00100111 => return Ok(DAA),
        0b00101111 => return Ok(CPL),
        0b00110111 => return Ok(SCF),
        0b00111111 => return Ok(CCF),
        0b00010000 => {
            get_byte(bytes, 1)?;
            return Ok(STOP);
        }
        _ => (),
    }
    if apply_mask(current, 0b00110000) == 0b00110001 {
        // ld r16, imm16
        let dst = R16::from(get_bits_of_byte(current, 2, 4) as usize);
        let src = bytes_to_word_little_endian(get_byte(bytes, 1)?, get_byte(bytes, 2)?);
        return Ok(LdR16Imm16(dst, src));
    }
    if apply_mask(current, 0b00110000) == 0b00110010 {
        // ld [r16mem], a
        let dst = R16mem::from(get_bits_of_byte(current, 2, 4) as usize);
        return Ok(LdR16memA(dst));
    }
    if apply_mask(current, 0b00110000) == 0b00111010 {
        // ld a, [r16mem]
        let src = R16mem::from(get_bits_of_byte(current, 2, 4) as usize);
        return Ok(LdAR16mem(src));
    }
    if current == 0b00001000 {
        // ld [imm16], sp
        let dst = bytes_to_word_little_endian(get_byte(bytes, 1)?, get_byte(bytes, 2)?);
        return Ok(LdAddrImm16Sp(dst));
    }
    if apply_mask(current, 0b00110000) == 0b00110011 {
        // inc r16
        return Ok(IncR16(R16::from(get_bits_of_byte(current, 2, 4) as usize)));
    }
    if apply_mask(current, 0b00110000) == 0b00111011 {
        // dec r16
        return Ok(DecR16(R16::from(get_bits_of_byte(current, 2, 4) as usize)));
    }
    if apply_mask(current, 0b00110000) == 0b00111001 {
        // add hl, r16
        let r16 = R16::from(get_bits_of_byte(current, 2, 4) as usize);
        return Ok(AddHlR16(r16));
    }
    if apply_mask(current, 0b00111000) == 0b00111100 {
        // inc r8
        return Ok(IncR8(R8::from(get_bits_of_byte(current, 2, 5) as usize)));
    }
    if apply_mask(current, 0b00111000) == 0b00111101 {
        // dec r8
        return Ok(DecR8(R8::from(get_bits_of_byte(current, 2, 5) as usize)));
    }
    if apply_mask(current, 0b00111000) == 0b00111110 {
        // ld r8, imm8
        let dst = R8::from(get_bits_of_byte(current, 2, 5) as usize);
        let src = get_byte(bytes, 1)?;
        return Ok(LdR8Imm8(dst, src));
    }
    if current == 0b00011000 {
        // jr imm8
        let dst = get_byte(bytes, 1)?;
        return Ok(JrImm8(dst));
    }
    if apply_mask(current, 0b00011000) == 0b00111000 {
        // jr cond, imm8
        let cond = Cond::from(get_bits_of_byte(current, 3, 5) as usize);
        let dst = get_byte(bytes, 1)?;
        return Ok(JrCondImm8(cond, dst));
    }

    //return Err(DisassemblyError::UnrecognisedInstruction(current));
    return Ok(Unkown(current));
}

fn block_1(bytes: &[u8]) -> Result<Instruction, DisassemblyError> {
    // Instructions starting bith bits 01
    use Instruction::*;
    let current = get_byte(bytes, 0)?;
    if current == 0b01110110 {
        // halt
        return Ok(HALT);
    }
    let dst = R8::from(get_bits_of_byte(current, 2, 5) as usize);
    let src = R8::from(get_bits_of_byte(current, 5, 8) as usize);
    if matches!(dst, R8::AddrHL) && matches!(src, R8::AddrHL) {
        return Ok(Unkown(current));
    }
    // ld r8, r8
    return Ok(LdR8R8(dst, src));
}

fn block_2(bytes: &[u8]) -> Result<Instruction, DisassemblyError> {
    // Instructions starting bith bits 10
    use Instruction::*;
    let current = get_byte(bytes, 0)?;
    let src = R8::from(get_bits_of_byte(current, 5, 8) as usize);
    if apply_mask(current, 0b00000111) == 0b10000111 {
        // add a, r8
        return Ok(AddAR8(src));
    }
    if apply_mask(current, 0b00000111) == 0b10001111 {
        // adc a, r8
        return Ok(AdcAR8(src));
    }
    if apply_mask(current, 0b00000111) == 0b10010111 {
        // sub a, r8
        return Ok(SubAR8(src));
    }
    if apply_mask(current, 0b00000111) == 0b10011111 {
        // sbc a, r8
        return Ok(SbcAR8(src));
    }
    if apply_mask(current, 0b00000111) == 0b10100111 {
        // and a, r8
        return Ok(AndAR8(src));
    }
    if apply_mask(current, 0b00000111) == 0b10101111 {
        // xor a, r8
        return Ok(XorAR8(src));
    }
    if apply_mask(current, 0b00000111) == 0b10110111 {
        // or a, r8
        return Ok(OrAR8(src));
    }
    if apply_mask(current, 0b00000111) == 0b10111111 {
        // cp a, r8
        return Ok(CpAR8(src));
    }
    return Ok(Unkown(current));
}

fn block_3(bytes: &[u8]) -> Result<Instruction, DisassemblyError> {
    // Instructions starting bith bits 11
    use Instruction::*;
    let current = get_byte(bytes, 0)?;
    match current {
        0b11000110 => return Ok(AddAImm8(get_byte(bytes, 1)?)),
        0b11001110 => return Ok(AdcAImm8(get_byte(bytes, 1)?)),
        0b11010110 => return Ok(SubAImm8(get_byte(bytes, 1)?)),
        0b11011110 => return Ok(SbcAImm8(get_byte(bytes, 1)?)),
        0b11100110 => return Ok(AndAImm8(get_byte(bytes, 1)?)),
        0b11101110 => return Ok(XorAImm8(get_byte(bytes, 1)?)),
        0b11110110 => return Ok(OrAImm8(get_byte(bytes, 1)?)),
        0b11111110 => return Ok(CpAImm8(get_byte(bytes, 1)?)),
        _ => (),
    }
    return Ok(Unkown(current));
}

pub fn get_instruction(bytes: &[u8]) -> Result<Instruction, DisassemblyError> {
    use Instruction::Unkown;
    assert!(!bytes.is_empty());
    let current = get_byte(bytes, 0)?;
    if apply_mask_equal(current, 0b00111111) {
        return block_0(bytes);
    }
    if apply_mask(current, 0b00111111) == 0b01111111 {
        return block_1(bytes);
    }
    if apply_mask(current, 0b00111111) == 0b10111111 {
        return block_2(bytes);
    }
    if apply_mask(current, 0b00111111) == 0b11111111 {
        return block_3(bytes);
    }
    //return Err(DisassemblyError::UnrecognisedInstruction(current));
    return Ok(Unkown(current));
}

pub fn disassemble_program(bytes: &[u8]) -> Result<Vec<Instruction>, DisassemblyError> {
    let mut instructions = vec![];
    let mut head = 0;
    while head < bytes.len() {
        let instruction = get_instruction(&bytes[head..bytes.len()])?;
        head += instruction.get_size();
        instructions.push(instruction);
    }
    return Ok(instructions);
}
