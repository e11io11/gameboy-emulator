pub mod disassembler;
use crate::hardware::cpu::{CPU, Register};
use crate::hardware::memory::MemoryMap;
use crate::utils::{check_overflow_word, get_bit_of_byte, set_bit_of_byte, variant_eq};
use disassembler::Cond;
use disassembler::Instruction;
use disassembler::R8;
use disassembler::R16;
use disassembler::R16mem;

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
        LdR16Imm16(..) => execute_ld_r16_imm16(mem_map, cpu, instruction)?,
        LdR16memA(..) => execute_ld_r16mem_a(mem_map, cpu, instruction)?,
        LdAR16mem(..) => execute_ld_a_r16mem(mem_map, cpu, instruction)?,
        LdAddrImm16Sp(..) => execute_ld_addrimm16_sp(mem_map, cpu, instruction)?,
        LdR8Imm8(..) => execute_ld_r8_imm8(mem_map, cpu, instruction)?,
        LdR8R8(dst, src) => execute_ld_r8_r8(mem_map, cpu, dst, src)?,
        IncR8(r8) => execute_inc_r8(mem_map, cpu, r8)?,
        IncR16(r16) => execute_inc_dec_r16(mem_map, cpu, r16, true)?,
        DecR8(r8) => execute_dec_r8(mem_map, cpu, r8)?,
        DecR16(r16) => execute_inc_dec_r16(mem_map, cpu, r16, false)?,
        AddHlR16(r16) => execute_add_hl_r16(cpu, r16),
        JrImm8(offset) => execute_jr(cpu, *offset as i8),
        JrCondImm8(cond, offset) => execute_jr_cond(cpu, cond, *offset as i8),
    });
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
    let bit_11_overflow = check_overflow_word(prev_value, added, new_value, 11);
    let bit_15_overflow = check_overflow_word(prev_value, added, new_value, 15);
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

fn execute_jr_cond(cpu: &mut CPU, cond: &Cond, offset: i8) -> u32 {
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

fn execute_jr(cpu: &mut CPU, offset: i8) -> u32 {
    if offset >= 0 {
        cpu.add_byte(&Register::PC, offset.unsigned_abs());
    } else {
        cpu.sub_byte(&Register::PC, offset.unsigned_abs());
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
