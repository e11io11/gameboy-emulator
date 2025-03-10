pub mod disassembler;
use crate::hardware::cpu::{CPU, Register};
use crate::hardware::memory::MemoryMap;
use disassembler::Operation;

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
        LdR16Imm16(dst, src) => cpu.write_word(&dst.clone().into(), *src),
        LdR16memA(_) => execute_ld_r16mem_a(mem_map, cpu, operation)?,
        LdAR16mem(_) => execute_ld_a_r16mem(mem_map, cpu, operation)?,
        LdAddrImm16Sp(_) => execute_ld_addrimm16_sp(mem_map, cpu, operation)?,
        LdR8Imm8(_, _) => execute_ld_r8_imm8(mem_map, cpu, operation)?,
        _ => todo!(),
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
    _mem_map: &mut MemoryMap,
    cpu: &mut CPU,
    operation: &Operation,
) -> Result<(), ExecutionError> {
    assert!(matches!(operation, Operation::LdR8Imm8(_, _)));
    if let Operation::LdR8Imm8(dst, src) = operation {
        cpu.write_byte(&dst.clone().into(), *src);
    }
    return Ok(());
}
