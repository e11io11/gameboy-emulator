pub mod disassembler;
use crate::hardware::cpu::{CPU, Register};
use crate::hardware::memory::MemoryMap;
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
) -> Result<u16, ExecutionError> {
    use Instruction::*;
    return Ok(match instruction {
        NOP => todo!(),
        RLCA => todo!(),
        RRCA => todo!(),
        RLA => todo!(),
        RRA => todo!(),
        DAA => todo!(),
        CPL => todo!(),
        SCF => todo!(),
        CCF => todo!(),
        STOP => todo!(),
        LdR16Imm16(..) => execute_ld_r16_imm16(mem_map, cpu, instruction)?,
        LdR16memA(..) => execute_ld_r16mem_a(mem_map, cpu, instruction)?,
        LdAR16mem(..) => execute_ld_a_r16mem(mem_map, cpu, instruction)?,
        LdAddrImm16Sp(..) => execute_ld_addrimm16_sp(mem_map, cpu, instruction)?,
        LdR8Imm8(..) => execute_ld_r8_imm8(mem_map, cpu, instruction)?,
        IncR8(r8) => execute_inc_dec_r8(mem_map, cpu, r8, true)?,
        IncR16(r16) => execute_inc_dec_r16(mem_map, cpu, r16, true)?,
        DecR8(r8) => execute_inc_dec_r8(mem_map, cpu, r8, false)?,
        DecR16(r16) => execute_inc_dec_r16(mem_map, cpu, r16, false)?,
        JrImm8(offset) => execute_jr(cpu, *offset as i8),
        JrCondImm8(cond, offset) => execute_jr_cond(cpu, cond, *offset as i8),
    });
}

fn execute_jr_cond(cpu: &mut CPU, cond: &Cond, offset: i8) -> u16 {
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

fn execute_jr(cpu: &mut CPU, offset: i8) -> u16 {
    if offset >= 0 {
        cpu.add_byte(&Register::PC, offset.unsigned_abs());
    } else {
        cpu.sub_byte(&Register::PC, offset.unsigned_abs());
    }
    return 2;
}

fn execute_inc_dec_r8(
    mem_map: &mut MemoryMap,
    cpu: &mut CPU,
    r8: &R8,
    inc: bool,
) -> Result<u16, ExecutionError> {
    return Ok(match r8 {
        R8::AddrHL if inc => {
            mem_map.add_byte(cpu.read_byte(&Register::HL) as usize, 1)?;
            3
        }
        R8::AddrHL => {
            mem_map.sub_byte(cpu.read_byte(&Register::HL) as usize, 1)?;
            3
        }
        _ if inc => {
            cpu.add_byte(&r8.clone().into(), 1);
            1
        }
        _ => {
            cpu.sub_byte(&r8.clone().into(), 1);
            1
        }
    });
}

fn execute_inc_dec_r16(
    _mem_map: &mut MemoryMap,
    cpu: &mut CPU,
    r16: &R16,
    inc: bool,
) -> Result<u16, ExecutionError> {
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
) -> Result<u16, ExecutionError> {
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
) -> Result<u16, ExecutionError> {
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
) -> Result<u16, ExecutionError> {
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
) -> Result<u16, ExecutionError> {
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
) -> Result<u16, ExecutionError> {
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
