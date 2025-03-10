use crate::utils::{
    get_bit_of_byte, get_word_left_byte, get_word_right_byte, set_bit_of_byte, set_word_left_byte,
    set_word_right_byte,
};

#[derive(Debug, Clone, Copy)]
pub enum Register {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
    FlagZ,
    FlagN,
    FlagH,
    FlagC,
}

impl Register {
    pub fn read(&self, cpu: &CPU) -> u16 {
        use Register::*;
        return match self {
            A | B | C | D | E | H | L => cpu.read_byte(self) as u16,
            AF | BC | DE | HL | SP | PC => cpu.read_word(self),
            FlagZ | FlagN | FlagH | FlagC => cpu.read_bit(self) as u16,
        };
    }

    pub fn write(&self, cpu: &mut CPU, data: u16) {
        use Register::*;
        match self {
            A | B | C | D | E | H | L => cpu.write_byte(self, get_word_right_byte(data)),
            AF | BC | DE | HL | SP | PC => cpu.write_word(self, data),
            FlagZ | FlagN | FlagH | FlagC => {
                cpu.write_bit(self, get_bit_of_byte(get_word_right_byte(data), 7))
            }
        }
    }

    pub fn is_byte_register(&self) -> bool {
        use Register::*;
        return matches!(self, A | B | C | D | E | H | L);
    }
    pub fn is_word_register(&self) -> bool {
        use Register::*;
        return matches!(self, AF | BC | DE | HL | SP | PC);
    }
    pub fn is_bit_register(&self) -> bool {
        use Register::*;
        return matches!(self, FlagZ | FlagN | FlagH | FlagC);
    }
}

#[derive(Debug)]
pub struct CPU {
    af: u16,
    bc: u16,
    de: u16,
    hl: u16,
    sp: u16,
    pc: u16,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            af: 0,
            bc: 0,
            de: 0,
            hl: 0,
            sp: 0,
            pc: 0,
        }
    }

    pub fn read_word(&self, register: &Register) -> u16 {
        use Register::*;
        match register {
            AF => self.af,
            BC => self.bc,
            DE => self.de,
            HL => self.hl,
            SP => self.sp,
            PC => self.pc,
            _ => panic!("Cannot read word with register {:?}", register),
        }
    }

    pub fn write_word(&mut self, register: &Register, word: u16) {
        use Register::*;
        match register {
            AF => self.af = word,
            BC => self.bc = word,
            DE => self.de = word,
            HL => self.hl = word,
            SP => self.sp = word,
            PC => self.pc = word,
            _ => panic!("Cannot write word with register {:?}", register),
        }
    }

    pub fn read_byte(&self, register: &Register) -> u8 {
        use Register::*;
        match register {
            A => get_word_left_byte(self.af),
            B => get_word_left_byte(self.bc),
            C => get_word_right_byte(self.bc),
            D => get_word_left_byte(self.de),
            E => get_word_right_byte(self.de),
            H => get_word_left_byte(self.hl),
            L => get_word_right_byte(self.hl),
            _ => panic!("Cannot read byte with register {:?}", register),
        }
    }
    pub fn write_byte(&mut self, register: &Register, byte: u8) {
        use Register::*;
        match register {
            A => self.af = set_word_left_byte(self.af, byte),
            B => self.bc = set_word_left_byte(self.bc, byte),
            C => self.bc = set_word_right_byte(self.bc, byte),
            D => self.de = set_word_left_byte(self.de, byte),
            E => self.de = set_word_right_byte(self.de, byte),
            H => self.hl = set_word_left_byte(self.hl, byte),
            L => self.hl = set_word_right_byte(self.hl, byte),
            _ => panic!("Cannot write byte with register {:?}", register),
        }
    }

    pub fn read_bit(&self, register: &Register) -> bool {
        use Register::*;
        match register {
            FlagZ => get_bit_of_byte(get_word_right_byte(self.af), 0),
            FlagN => get_bit_of_byte(get_word_right_byte(self.af), 1),
            FlagH => get_bit_of_byte(get_word_right_byte(self.af), 2),
            FlagC => get_bit_of_byte(get_word_right_byte(self.af), 3),
            _ => panic!("Cannot read bit with register {:?}", register),
        }
    }

    pub fn write_bit(&mut self, register: &Register, bit: bool) {
        use Register::*;
        match register {
            FlagZ => {
                self.af = set_word_right_byte(
                    self.af,
                    set_bit_of_byte(get_word_right_byte(self.af), 0, bit),
                )
            }
            FlagN => {
                self.af = set_word_right_byte(
                    self.af,
                    set_bit_of_byte(get_word_right_byte(self.af), 1, bit),
                )
            }
            FlagH => {
                self.af = set_word_right_byte(
                    self.af,
                    set_bit_of_byte(get_word_right_byte(self.af), 2, bit),
                )
            }
            FlagC => {
                self.af = set_word_right_byte(
                    self.af,
                    set_bit_of_byte(get_word_right_byte(self.af), 3, bit),
                )
            }
            _ => panic!("Cannot write bit with register {:?}", register),
        }
    }

    pub fn incr_word(&mut self, register: &Register) {
        use Register::*;
        match register {
            AF => self.af = self.af.wrapping_add(1),
            BC => self.bc = self.bc.wrapping_add(1),
            DE => self.de = self.de.wrapping_add(1),
            HL => self.hl = self.hl.wrapping_add(1),
            SP => self.sp = self.sp.wrapping_add(1),
            PC => self.pc = self.pc.wrapping_add(1),
            _ => panic!("Cannot increase word with register {:?}", register),
        }
    }

    pub fn decr_word(&mut self, register: &Register) {
        use Register::*;
        match register {
            AF => self.af = self.af.wrapping_sub(1),
            BC => self.bc = self.bc.wrapping_sub(1),
            DE => self.de = self.de.wrapping_sub(1),
            HL => self.hl = self.hl.wrapping_sub(1),
            SP => self.sp = self.sp.wrapping_sub(1),
            PC => self.pc = self.pc.wrapping_sub(1),
            _ => panic!("Cannot decrease word with register {:?}", register),
        }
    }
}
