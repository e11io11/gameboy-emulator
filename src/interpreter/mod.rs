pub mod disassembler;
use crate::hardware::cpu::{CPU, Register};
use crate::hardware::memory::MemoryMap;
use crate::utils::{
    borrow_occurred_byte, borrow_occurred_word, endianess_conversion, get_bit_of_byte,
    overflow_occured_byte, overflow_occured_word, set_bit_of_byte,
};
use disassembler::Cond;
use disassembler::Instruction;
use disassembler::R8;
use disassembler::R16;
use disassembler::R16mem;
use disassembler::R16stk;

#[derive(Debug)]
pub enum ExecutionError {
    IllegalInstructionError(Instruction, String),
    MemoryOutOfBoundsError(usize),
}

pub fn execute(
    mem_map: &mut MemoryMap,
    cpu: &mut CPU,
    instruction: &Instruction,
) -> Result<u32, ExecutionError> {
    use Instruction::*;
    return Ok(match instruction {
        Unkown(_) => {
            return Err(ExecutionError::IllegalInstructionError(
                instruction.clone(),
                "Instruction is unkown or has not yet been implemented".to_string(),
            ));
        }
        NOP => 1,
        RLCA => execute_rlca(cpu),
        RRCA => execute_rrca(cpu),
        RLA => execute_rla(cpu),
        RRA => execute_rra(cpu),
        DAA => execute_daa(cpu),
        CPL => execute_cpl(cpu),
        SCF => execute_scf(cpu),
        CCF => execute_ccf(cpu),
        STOP => execute_stop(),
        HALT => todo!(),
        DI => execute_di(cpu),
        EI => execute_ei(cpu),
        LdR16Imm16(..) => execute_ld_r16_imm16(mem_map, cpu, instruction)?,
        LdR16memA(..) => execute_ld_r16mem_a(mem_map, cpu, instruction)?,
        LdAR16mem(..) => execute_ld_a_r16mem(mem_map, cpu, instruction)?,
        LdAddrImm16Sp(..) => execute_ld_addrimm16_sp(mem_map, cpu, instruction)?,
        LdR8Imm8(..) => execute_ld_r8_imm8(mem_map, cpu, instruction)?,
        LdR8R8(dst, src) => execute_ld_r8_r8(mem_map, cpu, dst, src)?,
        LdAddrImm16A(word) => execute_ld_addr_imm16_a(mem_map, cpu, *word)?,
        LdAAddrImm16(word) => execute_ld_a_addr_imm16(mem_map, cpu, *word)?,
        LdhAddrCA => execute_ldh_addr_c_a(mem_map, cpu)?,
        LdhAAddrC => execute_ldh_a_addr_c(mem_map, cpu)?,
        LdhAddrImm8A(byte) => execute_ldh_addr_imm8_a(mem_map, cpu, *byte)?,
        LdhAAddrImm8(byte) => execute_ldh_a_addr_imm8(mem_map, cpu, *byte)?,
        IncR8(r8) => execute_inc_r8(mem_map, cpu, r8)?,
        IncR16(r16) => execute_inc_dec_r16(mem_map, cpu, r16, true)?,
        DecR8(r8) => execute_dec_r8(mem_map, cpu, r8)?,
        DecR16(r16) => execute_inc_dec_r16(mem_map, cpu, r16, false)?,
        AddHlR16(r16) => execute_add_hl_r16(cpu, r16),
        AddAR8(r8) => execute_add_a_r8(mem_map, cpu, r8)?,
        AdcAR8(r8) => execute_adc_a_r8(mem_map, cpu, r8)?,
        SubAR8(r8) => execute_sub_a_r8(mem_map, cpu, r8)?,
        SbcAR8(r8) => execute_sbc_a_r8(mem_map, cpu, r8)?,
        AndAR8(r8) => execute_and_a_r8(mem_map, cpu, r8)?,
        XorAR8(r8) => execute_xor_a_r8(mem_map, cpu, r8)?,
        OrAR8(r8) => execute_or_a_r8(mem_map, cpu, r8)?,
        CpAR8(r8) => execute_cp_a_r8(mem_map, cpu, r8)?,
        AddAImm8(byte) => execute_add_a_imm8(cpu, *byte),
        AdcAImm8(byte) => execute_adc_a_imm8(cpu, *byte),
        SubAImm8(byte) => execute_sub_a_imm8(cpu, *byte),
        SbcAImm8(byte) => execute_sbc_a_imm8(cpu, *byte),
        AndAImm8(byte) => execute_and_a_imm8(cpu, *byte),
        XorAImm8(byte) => execute_xor_a_imm8(cpu, *byte),
        OrAImm8(byte) => execute_or_a_imm8(cpu, *byte),
        CpAImm8(byte) => execute_cp_a_imm8(cpu, *byte),
        Ret => execute_ret(mem_map, cpu)?,
        Reti => todo!(),
        RetCond(cond) => execute_ret_cond(mem_map, cpu, cond)?,
        JpImm16(word) => execute_jp_imm16(cpu, *word),
        JpCondImm16(cond, word) => execute_jp_cond_imm16(cpu, cond, *word),
        JpHl => execute_jp_hl(cpu),
        JrImm8(offset) => execute_jr(cpu, *offset),
        JrCondImm8(cond, offset) => execute_jr_cond(cpu, cond, *offset),
        PopR16stk(r16stk) => execute_pop_r16stk(mem_map, cpu, r16stk)?,
        PushR16stk(r16stk) => execute_push_r16stk(mem_map, cpu, r16stk)?,
    });
}

fn execute_di(cpu: &mut CPU) -> u32 {
    cpu.disable_interupts();
    return 1;
}

fn execute_ei(cpu: &mut CPU) -> u32 {
    cpu.enable_interupts();
    return 1;
}

fn execute_jp_imm16(cpu: &mut CPU, word: u16) -> u32 {
    cpu.write_word(&Register::PC, word);
    return 4;
}

fn execute_jp_cond_imm16(cpu: &mut CPU, cond: &Cond, word: u16) -> u32 {
    let condition = match cond {
        Cond::Z | Cond::C => true,
        Cond::NotZ | Cond::NotC => false,
    };
    if cpu.read_bit(&cond.clone().into()) == condition {
        return execute_jp_imm16(cpu, word);
    }
    return 3;
}

fn execute_jp_hl(cpu: &mut CPU) -> u32 {
    cpu.write_word(&Register::PC, cpu.read_word(&Register::HL));
    return 1;
}

fn execute_ret(mem_map: &MemoryMap, cpu: &mut CPU) -> Result<u32, ExecutionError> {
    let value = mem_map.read_word(cpu.read_word(&Register::SP) as usize)?;
    cpu.write_word(&Register::PC, endianess_conversion(value));
    cpu.add_word(&Register::SP, 2);
    return Ok(4);
}

fn execute_ret_cond(
    mem_map: &MemoryMap,
    cpu: &mut CPU,
    cond: &Cond,
) -> Result<u32, ExecutionError> {
    let condition = match cond {
        Cond::Z | Cond::C => true,
        Cond::NotZ | Cond::NotC => false,
    };
    if cpu.read_bit(&cond.clone().into()) == condition {
        execute_ret(mem_map, cpu)?;
        return Ok(5);
    }
    return Ok(2);
}

fn execute_ld_addr_imm16_a(
    mem_map: &mut MemoryMap,
    cpu: &mut CPU,
    word: u16,
) -> Result<u32, ExecutionError> {
    mem_map.write_byte(word as usize, cpu.read_byte(&Register::A))?;
    return Ok(3);
}

fn execute_ld_a_addr_imm16(
    mem_map: &mut MemoryMap,
    cpu: &mut CPU,
    word: u16,
) -> Result<u32, ExecutionError> {
    cpu.write_byte(&Register::A, mem_map.read_byte(word as usize)?);
    return Ok(2);
}

fn execute_ldh_addr_c_a(mem_map: &mut MemoryMap, cpu: &mut CPU) -> Result<u32, ExecutionError> {
    mem_map.write_byte(
        0xFF00 + cpu.read_byte(&Register::C) as usize,
        cpu.read_byte(&Register::A),
    )?;
    return Ok(2);
}

fn execute_ldh_a_addr_c(mem_map: &mut MemoryMap, cpu: &mut CPU) -> Result<u32, ExecutionError> {
    cpu.write_byte(
        &Register::A,
        mem_map.read_byte(0xFF00 + cpu.read_byte(&Register::C) as usize)?,
    );
    return Ok(2);
}

fn execute_ldh_addr_imm8_a(
    mem_map: &mut MemoryMap,
    cpu: &mut CPU,
    byte: u8,
) -> Result<u32, ExecutionError> {
    mem_map.write_byte(0xFF00 + byte as usize, cpu.read_byte(&Register::A))?;
    return Ok(3);
}

fn execute_ldh_a_addr_imm8(
    mem_map: &mut MemoryMap,
    cpu: &mut CPU,
    byte: u8,
) -> Result<u32, ExecutionError> {
    cpu.write_byte(&Register::A, mem_map.read_byte(0xFF00 + byte as usize)?);
    return Ok(3);
}

fn execute_push_r16stk(
    mem_map: &MemoryMap,
    cpu: &mut CPU,
    r16stk: &R16stk,
) -> Result<u32, ExecutionError> {
    use Register::*;
    let reg = r16stk.clone().into();
    let value = mem_map.read_word(cpu.read_word(&reg) as usize)?;
    cpu.write_word(&SP, endianess_conversion(value));
    cpu.sub_word(&SP, 2);
    return Ok(3);
}

fn execute_pop_r16stk(
    mem_map: &MemoryMap,
    cpu: &mut CPU,
    r16stk: &R16stk,
) -> Result<u32, ExecutionError> {
    use Register::*;
    let reg = r16stk.clone().into();
    let value = mem_map.read_word(cpu.read_word(&SP) as usize)?;
    cpu.write_word(&reg, endianess_conversion(value));
    cpu.add_word(&SP, 2);
    return Ok(3);
}

fn execute_cp_a_imm8(cpu: &mut CPU, byte: u8) -> u32 {
    use Register::*;
    let prev_value = cpu.read_byte(&A);
    let new_value = prev_value.wrapping_sub(byte);
    let bit_4_borrow = borrow_occurred_byte(prev_value, byte, 4);
    let borrow = byte > prev_value;
    if new_value == 0 {
        cpu.write_bit(&FlagZ, true);
    }
    if bit_4_borrow {
        cpu.write_bit(&FlagH, true);
    }
    if borrow {
        cpu.write_bit(&FlagC, true);
    }
    cpu.write_bit(&FlagN, true);
    return 2;
}

fn execute_cp_a_r8(mem_map: &MemoryMap, cpu: &mut CPU, r8: &R8) -> Result<u32, ExecutionError> {
    if matches!(r8, R8::AddrHL) {
        execute_cp_a_imm8(
            cpu,
            mem_map.read_byte(cpu.read_word(&Register::HL) as usize)?,
        );
        return Ok(2);
    } else {
        execute_cp_a_imm8(cpu, cpu.read_byte(&r8.clone().into()));
        return Ok(1);
    }
}

fn execute_or_a_imm8(cpu: &mut CPU, byte: u8) -> u32 {
    use Register::*;
    cpu.write_byte(&A, cpu.read_byte(&A) | byte);
    if cpu.read_byte(&A) == 0 {
        cpu.write_bit(&FlagZ, true);
    }

    cpu.write_bit(&FlagN, false);
    cpu.write_bit(&FlagH, false);
    cpu.write_bit(&FlagC, false);
    return 2;
}

fn execute_or_a_r8(mem_map: &MemoryMap, cpu: &mut CPU, r8: &R8) -> Result<u32, ExecutionError> {
    if matches!(r8, R8::AddrHL) {
        execute_or_a_imm8(
            cpu,
            mem_map.read_byte(cpu.read_word(&Register::HL) as usize)?,
        );
        return Ok(2);
    } else {
        execute_or_a_imm8(cpu, cpu.read_byte(&r8.clone().into()));
        return Ok(1);
    }
}

fn execute_xor_a_imm8(cpu: &mut CPU, byte: u8) -> u32 {
    use Register::*;
    cpu.write_byte(&A, cpu.read_byte(&A) ^ byte);
    if cpu.read_byte(&A) == 0 {
        cpu.write_bit(&FlagZ, true);
    }

    cpu.write_bit(&FlagN, false);
    cpu.write_bit(&FlagH, false);
    cpu.write_bit(&FlagC, false);
    return 2;
}

fn execute_xor_a_r8(mem_map: &MemoryMap, cpu: &mut CPU, r8: &R8) -> Result<u32, ExecutionError> {
    if matches!(r8, R8::AddrHL) {
        execute_xor_a_imm8(
            cpu,
            mem_map.read_byte(cpu.read_word(&Register::HL) as usize)?,
        );
        return Ok(2);
    } else {
        execute_xor_a_imm8(cpu, cpu.read_byte(&r8.clone().into()));
        return Ok(1);
    }
}

fn execute_and_a_imm8(cpu: &mut CPU, byte: u8) -> u32 {
    use Register::*;
    cpu.write_byte(&A, cpu.read_byte(&A) & byte);
    if cpu.read_byte(&A) == 0 {
        cpu.write_bit(&FlagZ, true);
    }

    cpu.write_bit(&FlagN, false);
    cpu.write_bit(&FlagH, true);
    cpu.write_bit(&FlagC, false);
    return 2;
}

fn execute_and_a_r8(mem_map: &MemoryMap, cpu: &mut CPU, r8: &R8) -> Result<u32, ExecutionError> {
    if matches!(r8, R8::AddrHL) {
        execute_and_a_imm8(
            cpu,
            mem_map.read_byte(cpu.read_word(&Register::HL) as usize)?,
        );
        return Ok(2);
    } else {
        execute_and_a_imm8(cpu, cpu.read_byte(&r8.clone().into()));
        return Ok(1);
    }
}

fn execute_sbc_a_imm8(cpu: &mut CPU, byte: u8) -> u32 {
    let sub_c = match cpu.read_bit(&Register::FlagC) {
        true => 1,
        false => 0,
    };
    execute_sub_a_imm8(cpu, byte.wrapping_sub(sub_c));
    return 2;
}

fn execute_sbc_a_r8(mem_map: &MemoryMap, cpu: &mut CPU, r8: &R8) -> Result<u32, ExecutionError> {
    let sub_c = match cpu.read_bit(&Register::FlagC) {
        true => 1,
        false => 0,
    };
    if matches!(r8, R8::AddrHL) {
        execute_sub_a_imm8(
            cpu,
            mem_map
                .read_byte(cpu.read_word(&Register::HL) as usize)?
                .wrapping_sub(sub_c),
        );
        return Ok(2);
    } else {
        execute_sub_a_imm8(cpu, cpu.read_byte(&r8.clone().into()).wrapping_sub(sub_c));
        return Ok(1);
    }
}

fn execute_sub_a_imm8(cpu: &mut CPU, byte: u8) -> u32 {
    use Register::*;
    let prev_value = cpu.read_byte(&A);
    cpu.sub_byte(&A, byte);
    let new_value = cpu.read_byte(&A);
    let bit_4_borrow = borrow_occurred_byte(prev_value, byte, 4);
    let borrow = byte > prev_value;
    if new_value == 0 {
        cpu.write_bit(&FlagZ, true);
    }
    if bit_4_borrow {
        cpu.write_bit(&FlagH, true);
    }
    if borrow {
        cpu.write_bit(&FlagC, true);
    }
    cpu.write_bit(&FlagN, true);
    return 2;
}

fn execute_sub_a_r8(mem_map: &MemoryMap, cpu: &mut CPU, r8: &R8) -> Result<u32, ExecutionError> {
    if matches!(r8, R8::AddrHL) {
        execute_sub_a_imm8(
            cpu,
            mem_map.read_byte(cpu.read_word(&Register::HL) as usize)?,
        );
        return Ok(2);
    } else {
        execute_sub_a_imm8(cpu, cpu.read_byte(&r8.clone().into()));
        return Ok(1);
    }
}

fn execute_adc_a_r8(mem_map: &MemoryMap, cpu: &mut CPU, r8: &R8) -> Result<u32, ExecutionError> {
    let add_c = match cpu.read_bit(&Register::FlagC) {
        true => 1,
        false => 0,
    };
    if matches!(r8, R8::AddrHL) {
        execute_add_a_imm8(
            cpu,
            mem_map
                .read_byte(cpu.read_word(&Register::HL) as usize)?
                .wrapping_add(add_c),
        );
        return Ok(2);
    } else {
        execute_add_a_imm8(cpu, cpu.read_byte(&r8.clone().into()).wrapping_add(add_c));
        return Ok(1);
    }
}

fn execute_adc_a_imm8(cpu: &mut CPU, byte: u8) -> u32 {
    let add_c = match cpu.read_bit(&Register::FlagC) {
        true => 1,
        false => 0,
    };
    execute_add_a_imm8(cpu, byte.wrapping_add(add_c));
    return 2;
}

fn execute_add_a_imm8(cpu: &mut CPU, byte: u8) -> u32 {
    use Register::*;
    let prev_value = cpu.read_byte(&A);
    cpu.add_byte(&A, byte);
    let new_value = cpu.read_byte(&A);
    let bit_3_overflow = overflow_occured_byte(prev_value, byte, new_value, 3);
    let bit_7_overflow = overflow_occured_byte(prev_value, byte, new_value, 7);
    if new_value == 0 {
        cpu.write_bit(&FlagZ, true);
    }
    if bit_3_overflow {
        cpu.write_bit(&FlagH, true);
    }
    if bit_7_overflow {
        cpu.write_bit(&FlagC, true);
    }
    cpu.write_bit(&FlagN, false);
    return 2;
}

fn execute_add_a_r8(mem_map: &MemoryMap, cpu: &mut CPU, r8: &R8) -> Result<u32, ExecutionError> {
    if matches!(r8, R8::AddrHL) {
        execute_add_a_imm8(
            cpu,
            mem_map.read_byte(cpu.read_word(&Register::HL) as usize)?,
        );
        return Ok(2);
    } else {
        execute_add_a_imm8(cpu, cpu.read_byte(&r8.clone().into()));
        return Ok(1);
    }
}

fn execute_ld_r8_r8(
    mem_map: &mut MemoryMap,
    cpu: &mut CPU,
    dst: &R8,
    src: &R8,
) -> Result<u32, ExecutionError> {
    use R8::*;
    if matches!(dst, AddrHL) && matches!(src, AddrHL) {
        return Err(ExecutionError::IllegalInstructionError(
            Instruction::LdR8R8(dst.clone(), src.clone()),
            "ld [hl], [hl] is illegal".into(),
        ));
    }
    if matches!(src, AddrHL) {
        cpu.write_byte(
            &dst.clone().into(),
            mem_map.read_byte(cpu.read_word(&Register::HL) as usize)?,
        )
    } else if matches!(dst, AddrHL) {
        mem_map.write_byte(
            cpu.read_word(&Register::HL) as usize,
            cpu.read_byte(&dst.clone().into()),
        )?;
    } else {
        cpu.write_byte(&dst.clone().into(), cpu.read_byte(&src.clone().into()));
    }
    return Ok(1);
}

fn execute_add_hl_r16(cpu: &mut CPU, r16: &R16) -> u32 {
    use Register::*;
    let added = cpu.read_word(&r16.clone().into());
    let prev_value = cpu.read_word(&HL);
    cpu.add_word(&HL, added);
    let new_value = cpu.read_word(&HL);
    cpu.write_bit(&FlagN, false);
    let bit_11_overflow = overflow_occured_word(prev_value, added, new_value, 11);
    let bit_15_overflow = overflow_occured_word(prev_value, added, new_value, 15);
    if bit_11_overflow {
        cpu.write_bit(&FlagH, true);
    }
    if bit_15_overflow {
        cpu.write_bit(&FlagC, true);
    }
    return 2;
}

fn execute_stop() -> u32 {
    // todo
    return 0;
}

fn execute_ccf(cpu: &mut CPU) -> u32 {
    use Register::*;
    cpu.write_bit(&FlagN, false);
    cpu.write_bit(&FlagH, false);
    cpu.write_bit(&FlagC, !cpu.read_bit(&FlagC));
    return 1;
}

fn execute_scf(cpu: &mut CPU) -> u32 {
    use Register::*;
    cpu.write_bit(&FlagN, false);
    cpu.write_bit(&FlagH, false);
    cpu.write_bit(&FlagC, true);
    return 1;
}

fn execute_cpl(cpu: &mut CPU) -> u32 {
    use Register::*;
    cpu.write_byte(&A, !cpu.read_byte(&A));
    cpu.write_bit(&FlagN, true);
    cpu.write_bit(&FlagH, true);
    return 1;
}

fn execute_daa(cpu: &mut CPU) -> u32 {
    use Register::*;
    let mut adjustment = 0;
    if cpu.read_bit(&FlagN) {
        if cpu.read_bit(&FlagH) {
            adjustment += 6;
        }
        if cpu.read_bit(&FlagC) {
            adjustment += 96;
        }
        cpu.sub_byte(&A, adjustment)
    } else {
        let a_value = cpu.read_byte(&A);
        if cpu.read_bit(&FlagH) || a_value & 15 > 9 {
            adjustment += 6;
        }
        if cpu.read_bit(&FlagC) || a_value > 153 {
            adjustment += 96;
            cpu.write_bit(&A, true);
        }
        cpu.add_byte(&A, adjustment)
    }
    cpu.write_bit(&FlagZ, cpu.read_byte(&A) == 0);
    cpu.write_bit(&FlagH, false);
    return 1;
}

fn execute_rla(cpu: &mut CPU) -> u32 {
    use Register::*;
    let value = cpu.read_byte(&A);
    let left_bit = get_bit_of_byte(value, 0);
    cpu.write_byte(&A, set_bit_of_byte(value << 1, 7, cpu.read_bit(&FlagC)));
    cpu.write_bit(&FlagZ, false);
    cpu.write_bit(&FlagN, false);
    cpu.write_bit(&FlagH, false);
    cpu.write_bit(&FlagC, left_bit);
    return 1;
}

fn execute_rra(cpu: &mut CPU) -> u32 {
    use Register::*;
    let value = cpu.read_byte(&A);
    let right_bit = get_bit_of_byte(value, 7);
    cpu.write_byte(&A, set_bit_of_byte(value >> 1, 0, cpu.read_bit(&FlagC)));
    cpu.write_bit(&FlagZ, false);
    cpu.write_bit(&FlagN, false);
    cpu.write_bit(&FlagH, false);
    cpu.write_bit(&FlagC, right_bit);
    return 1;
}

fn execute_rlca(cpu: &mut CPU) -> u32 {
    use Register::*;
    let value = cpu.read_byte(&A);
    let left_bit = get_bit_of_byte(value, 0);
    cpu.write_byte(&A, (value << 1) | (value >> 7));
    cpu.write_bit(&FlagZ, false);
    cpu.write_bit(&FlagN, false);
    cpu.write_bit(&FlagH, false);
    cpu.write_bit(&FlagC, left_bit);
    return 1;
}

fn execute_rrca(cpu: &mut CPU) -> u32 {
    use Register::*;
    let value = cpu.read_byte(&A);
    let right_bit = get_bit_of_byte(value, 7);
    cpu.write_byte(&A, (value >> 1) | (value << 7));
    cpu.write_bit(&FlagZ, false);
    cpu.write_bit(&FlagN, false);
    cpu.write_bit(&FlagH, false);
    cpu.write_bit(&FlagC, right_bit);
    return 1;
}

fn execute_jr_cond(cpu: &mut CPU, cond: &Cond, offset: u8) -> u32 {
    use Cond::*;
    let condition = match cond {
        Z | C => true,
        NotZ | NotC => false,
    };
    if cpu.read_bit(&Z.clone().into()) != condition {
        return 2;
    }
    let _ = execute_jr(cpu, offset);
    return 3;
}

fn execute_jr(cpu: &mut CPU, offset: u8) -> u32 {
    let signed_offset = (offset as i8) as i16;
    if signed_offset >= 0 {
        cpu.add_word(&Register::PC, signed_offset.unsigned_abs());
    } else {
        cpu.sub_word(&Register::PC, signed_offset.unsigned_abs());
    }
    return 2;
}

fn execute_inc_r8(mem_map: &mut MemoryMap, cpu: &mut CPU, r8: &R8) -> Result<u32, ExecutionError> {
    let cycles = match r8 {
        R8::AddrHL => {
            let address = cpu.read_byte(&Register::HL) as usize;
            let prev_value = mem_map.read_byte(address)?;
            mem_map.add_byte(address, 1)?;
            let new_value = mem_map.read_byte(address)?;
            if new_value == 0 {
                cpu.write_bit(&Register::FlagZ, true)
            }
            if get_bit_of_byte(prev_value, 4) && !get_bit_of_byte(new_value, 4) {
                cpu.write_bit(&Register::FlagH, true)
            }
            3
        }
        _ => {
            let register = &r8.clone().into();
            let prev_value = cpu.read_byte(register);
            cpu.add_byte(register, 1);
            let new_value = cpu.read_byte(register);
            if new_value == 0 {
                cpu.write_bit(&Register::FlagZ, true)
            }
            if get_bit_of_byte(prev_value, 4) && !get_bit_of_byte(new_value, 4) {
                cpu.write_bit(&Register::FlagH, true)
            }
            1
        }
    };
    cpu.write_bit(&Register::FlagN, false);
    return Ok(cycles);
}

fn execute_dec_r8(mem_map: &mut MemoryMap, cpu: &mut CPU, r8: &R8) -> Result<u32, ExecutionError> {
    let cycles = match r8 {
        R8::AddrHL => {
            let address = cpu.read_byte(&Register::HL) as usize;
            let prev_value = mem_map.read_byte(address)?;
            mem_map.sub_byte(address, 1)?;
            let new_value = mem_map.read_byte(address)?;
            if new_value == 0 {
                cpu.write_bit(&Register::FlagZ, true)
            }
            if get_bit_of_byte(prev_value, 4) && !get_bit_of_byte(new_value, 4) {
                cpu.write_bit(&Register::FlagH, true)
            }
            3
        }
        _ => {
            let register = &r8.clone().into();
            let prev_value = cpu.read_byte(register);
            cpu.add_byte(register, 1);
            let new_value = cpu.read_byte(register);
            if new_value == 0 {
                cpu.write_bit(&Register::FlagZ, true)
            }
            if get_bit_of_byte(prev_value, 3) && !get_bit_of_byte(new_value, 3) {
                cpu.write_bit(&Register::FlagH, true)
            }
            1
        }
    };
    cpu.write_bit(&Register::FlagN, true);
    return Ok(cycles);
}

fn execute_inc_dec_r16(
    _mem_map: &mut MemoryMap,
    cpu: &mut CPU,
    r16: &R16,
    inc: bool,
) -> Result<u32, ExecutionError> {
    if inc {
        cpu.add_word(&r16.clone().into(), 1);
    } else {
        cpu.sub_word(&r16.clone().into(), 1);
    }
    return Ok(2);
}

fn handle_r16mem_add_or_decr(cpu: &mut CPU, r16mem: &R16mem) {
    use R16mem::*;
    match r16mem {
        IncrHL => cpu.add_word(&Register::HL, 1),
        DecrHL => cpu.sub_word(&Register::HL, 1),
        _ => (),
    }
}

fn execute_ld_r16_imm16(
    _mem_map: &mut MemoryMap,
    cpu: &mut CPU,
    instruction: &Instruction,
) -> Result<u32, ExecutionError> {
    assert!(matches!(instruction, Instruction::LdR16Imm16(..)));
    if let Instruction::LdR16Imm16(dst, src) = instruction {
        cpu.write_word(&dst.clone().into(), *src);
    }
    return Ok(3);
}

fn execute_ld_r16mem_a(
    mem_map: &mut MemoryMap,
    cpu: &mut CPU,
    instruction: &Instruction,
) -> Result<u32, ExecutionError> {
    assert!(matches!(instruction, Instruction::LdR16memA(_)));
    if let Instruction::LdR16memA(dst) = instruction {
        mem_map.write_byte(
            cpu.read_word(&dst.clone().into()) as usize,
            cpu.read_byte(&Register::A),
        )?;
        handle_r16mem_add_or_decr(cpu, dst);
    }
    return Ok(2);
}

fn execute_ld_a_r16mem(
    mem_map: &mut MemoryMap,
    cpu: &mut CPU,
    instruction: &Instruction,
) -> Result<u32, ExecutionError> {
    assert!(matches!(instruction, Instruction::LdAR16mem(_)));
    if let Instruction::LdAR16mem(src) = instruction {
        cpu.write_byte(
            &Register::A,
            mem_map.read_byte(cpu.read_word(&src.clone().into()) as usize)?,
        );
        handle_r16mem_add_or_decr(cpu, src);
    }
    return Ok(2);
}

fn execute_ld_addrimm16_sp(
    mem_map: &mut MemoryMap,
    cpu: &mut CPU,
    instruction: &Instruction,
) -> Result<u32, ExecutionError> {
    assert!(matches!(instruction, Instruction::LdAddrImm16Sp(_)));
    if let Instruction::LdAddrImm16Sp(dst) = instruction {
        mem_map.write_word(*dst as usize, cpu.read_word(&Register::SP))?;
    }
    return Ok(5);
}

fn execute_ld_r8_imm8(
    mem_map: &mut MemoryMap,
    cpu: &mut CPU,
    instruction: &Instruction,
) -> Result<u32, ExecutionError> {
    assert!(matches!(instruction, Instruction::LdR8Imm8(_, _)));
    use R8::*;
    if let Instruction::LdR8Imm8(dst, src) = instruction {
        match dst {
            AddrHL => mem_map.write_byte(cpu.read_word(&Register::HL) as usize, *src)?,
            _ => cpu.write_byte(&dst.clone().into(), *src),
        }
    }
    return Ok(2);
}
