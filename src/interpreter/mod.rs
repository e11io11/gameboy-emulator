pub mod disassembler;
use crate::hardware::cpu::{CPU, Register};
use crate::hardware::memory::MemoryMap;
use disassembler::Cond;
use disassembler::Operation;
use disassembler::R8;
use disassembler::R16;
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
    match operation {
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
        LdR16Imm16(..) => execute_ld_r16_imm16(mem_map, cpu, operation)?,
        LdR16memA(..) => execute_ld_r16mem_a(mem_map, cpu, operation)?,
        LdAR16mem(..) => execute_ld_a_r16mem(mem_map, cpu, operation)?,
        LdAddrImm16Sp(..) => execute_ld_addrimm16_sp(mem_map, cpu, operation)?,
        LdR8Imm8(..) => execute_ld_r8_imm8(mem_map, cpu, operation)?,
        IncR8(r8) => execute_inc_dec_r8(mem_map, cpu, r8, true)?,
        IncR16(r16) => execute_inc_dec_r16(mem_map, cpu, r16, true)?,
        DecR8(r8) => execute_inc_dec_r8(mem_map, cpu, r8, false)?,
        DecR16(r16) => execute_inc_dec_r16(mem_map, cpu, r16, false)?,
        JrImm8(offset) => execute_jr(cpu, *offset as i8),
        JrCondImm8(cond, offset) => execute_jr_cond(cpu, cond, *offset as i8),
    }
    return Ok(());
}

fn execute_jr_cond(cpu: &mut CPU, cond: &Cond, offset: i8) {
    use Cond::*;
    let condition = match cond {
        Z | C => true,
        NotZ | NotC => false,
    };
    if cpu.read_bit(&Z.clone().into()) != condition {
        return;
    }
    execute_jr(cpu, offset)
}

fn execute_jr(cpu: &mut CPU, offset: i8) {
    if offset >= 0 {
        cpu.incr_byte(&Register::PC, offset.unsigned_abs());
    } else {
        cpu.decr_byte(&Register::PC, offset.unsigned_abs());
    }
}

fn execute_inc_dec_r8(
    mem_map: &mut MemoryMap,
    cpu: &mut CPU,
    r8: &R8,
    inc: bool,
) -> Result<(), ExecutionError> {
    match r8 {
        R8::AddrHL if inc => mem_map.incr_byte(cpu.read_byte(&Register::HL) as usize, 1)?,
        R8::AddrHL => mem_map.decr_byte(cpu.read_byte(&Register::HL) as usize, 1)?,
        _ if inc => cpu.incr_byte(&r8.clone().into(), 1),
        _ => cpu.decr_byte(&r8.clone().into(), 1),
    }
    return Ok(());
}

fn execute_inc_dec_r16(
    _mem_map: &mut MemoryMap,
    cpu: &mut CPU,
    r16: &R16,
    inc: bool,
) -> Result<(), ExecutionError> {
    if inc {
        cpu.incr_word(&r16.clone().into(), 1);
    } else {
        cpu.decr_word(&r16.clone().into(), 1);
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
