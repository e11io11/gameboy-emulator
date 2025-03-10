pub mod disassembler;
use crate::hardware::cpu::{CPU, Register};
use crate::hardware::memory::MemoryMap;
use disassembler::Operation;
use disassembler::R8;
use disassembler::R16mem;

#[derive(Debug)]
pub enum ExecutionError {
    IllegalOperationError(Operation, String),
    MemoryOutOfBoundsError(usize),
}

pub fn execute(
    mem_map: &mut MemoryMap,
    cpu: &mut CPU,
    operation: &Operation,
) -> Result<(), ExecutionError> {
    use Operation::*;
    //use Register::*;
    match operation {
        LdR16Imm16(..) => execute_ld_r16_imm16(mem_map, cpu, operation)?,
        LdR16memA(..) => execute_ld_r16mem_a(mem_map, cpu, operation)?,
        LdAR16mem(..) => execute_ld_a_r16mem(mem_map, cpu, operation)?,
        LdAddrImm16Sp(..) => execute_ld_addrimm16_sp(mem_map, cpu, operation)?,
        LdR8Imm8(..) => execute_ld_r8_imm8(mem_map, cpu, operation)?,
        _ => todo!(),
    }
    return Ok(());
}

fn handle_r16mem_incr_or_decr(cpu: &mut CPU, r16mem: &R16mem) {
    use R16mem::*;
    match r16mem {
        IncrHL => cpu.incr_word(&Register::HL, 1),
        DecrHL => cpu.decr_word(&Register::HL, 1),
        _ => (),
    }
}

fn execute_ld_r16_imm16(
    _mem_map: &mut MemoryMap,
    cpu: &mut CPU,
    operation: &Operation,
) -> Result<(), ExecutionError> {
    assert!(matches!(operation, Operation::LdR16Imm16(..)));
    if let Operation::LdR16Imm16(dst, src) = operation {
        cpu.write_word(&dst.clone().into(), *src);
    }
    return Ok(());
}

fn execute_ld_r16mem_a(
    mem_map: &mut MemoryMap,
    cpu: &mut CPU,
    operation: &Operation,
) -> Result<(), ExecutionError> {
    assert!(matches!(operation, Operation::LdR16memA(_)));
    if let Operation::LdR16memA(dst) = operation {
        mem_map.write_byte(
            cpu.read_word(&dst.clone().into()) as usize,
            cpu.read_byte(&Register::A),
        )?;
        handle_r16mem_incr_or_decr(cpu, dst);
    }
    return Ok(());
}

fn execute_ld_a_r16mem(
    mem_map: &mut MemoryMap,
    cpu: &mut CPU,
    operation: &Operation,
) -> Result<(), ExecutionError> {
    assert!(matches!(operation, Operation::LdAR16mem(_)));
    if let Operation::LdAR16mem(src) = operation {
        cpu.write_byte(
            &Register::A,
            mem_map.read_byte(cpu.read_word(&src.clone().into()) as usize)?,
        );
        handle_r16mem_incr_or_decr(cpu, src);
    }
    return Ok(());
}

fn execute_ld_addrimm16_sp(
    mem_map: &mut MemoryMap,
    cpu: &mut CPU,
    operation: &Operation,
) -> Result<(), ExecutionError> {
    assert!(matches!(operation, Operation::LdAddrImm16Sp(_)));
    if let Operation::LdAddrImm16Sp(dst) = operation {
        mem_map.write_word(*dst as usize, cpu.read_word(&Register::SP))?;
    }
    return Ok(());
}

fn execute_ld_r8_imm8(
    mem_map: &mut MemoryMap,
    cpu: &mut CPU,
    operation: &Operation,
) -> Result<(), ExecutionError> {
    assert!(matches!(operation, Operation::LdR8Imm8(_, _)));
    use R8::*;
    if let Operation::LdR8Imm8(dst, src) = operation {
        match dst {
            AddrHL => mem_map.write_byte(cpu.read_word(&Register::HL) as usize, *src)?,
            _ => cpu.write_byte(&dst.clone().into(), *src),
        }
    }
    return Ok(());
}
