pub mod disassembler;
use crate::hardware::cpu::{CPU, Register};
use crate::hardware::memory::MemoryMap;
use crate::utils::DataSize;
use disassembler::Operand;
use disassembler::Operation;

#[derive(Debug)]
pub enum ExecutionError {
    IllegalOperationError(Operation, String),
    MemoryOutOfBoundsError(usize),
}

use ExecutionError::*;

pub fn execute(
    mem_map: &mut MemoryMap,
    cpu: &mut CPU,
    operation: &Operation,
) -> Result<(), ExecutionError> {
    match operation {
        Operation::LD(dest, src) => execute_ld(mem_map, cpu, operation, &dest, &src)?,
        _ => todo!(),
    }
    return Ok(());
}

pub fn execute_ld(
    mem_map: &mut MemoryMap,
    cpu: &mut CPU,
    current_operation: &Operation,
    dest: &Operand,
    src: &Operand,
) -> Result<(), ExecutionError> {
    use Register::HL;
    let data_size = ld_get_data_size(current_operation, dest, src)?;
    let src_value = match src {
        Operand::Register(r) => r.read(cpu),
        Operand::Address(op) => ld_read_address(mem_map, cpu, current_operation, &**op, data_size)?,
        Operand::Decr(r) if matches!(r, HL) => {
            let value = r.read(cpu);
            r.write(cpu, value - 1);
            value
        }
        Operand::Incr(r) if matches!(r, HL) => {
            let value = r.read(cpu);
            r.write(cpu, value + 1);
            value
        }
        Operand::Decr(_) | Operand::Incr(_) => {
            return Err(IllegalOperationError(
                current_operation.clone(),
                String::from("LD Incr/Decr should be used with register HL"),
            ));
        }
        Operand::Byte(value) => value.clone() as u16,
        Operand::Word(value) => value.clone(),
        Operand::Not(_) => {
            return Err(IllegalOperationError(
                current_operation.clone(),
                String::from("LD with bit size operand"),
            ));
        }
    };
    match dest {
        Operand::Register(r) => r.write(cpu, src_value),
        _ => todo!(),
    }
    return Ok(());
}

fn ld_read_address(
    mem_map: &mut MemoryMap,
    cpu: &mut CPU,
    current_operation: &Operation,
    operand: &Operand,
    data_size: DataSize,
) -> Result<u16, ExecutionError> {
    use DataSize::*;
    let address = ld_eval_address(cpu, current_operation, operand)?;
    return Ok(match data_size {
        BYTE => mem_map.read_byte(address as usize)? as u16,
        WORD => mem_map.read_word(address as usize)?,
        BIT => {
            return Err(IllegalOperationError(
                current_operation.clone(),
                String::from("Cannot read bit from address"),
            ));
        }
    });
}

fn ld_eval_address(
    cpu: &mut CPU,
    current_operation: &Operation,
    operand: &Operand,
) -> Result<u16, ExecutionError> {
    use Register::HL;
    let address = match operand {
        Operand::Register(r) if r.is_word_register() => r.read(cpu),
        Operand::Decr(r) if matches!(r, HL) => {
            let value = r.read(cpu);
            r.write(cpu, value - 1);
            value
        }
        Operand::Incr(r) if matches!(r, HL) => {
            let value = r.read(cpu);
            r.write(cpu, value + 1);
            value
        }
        Operand::Decr(_) | Operand::Incr(_) => {
            return Err(IllegalOperationError(
                current_operation.clone(),
                String::from("LD Incr/Decr should be used with register HL"),
            ));
        }
        Operand::Word(value) => value.clone(),
        _ => {
            return Err(IllegalOperationError(
                current_operation.clone(),
                String::from("LD Address should be of word size"),
            ));
        }
    };
    Ok(address)
}

fn ld_get_data_size(
    current_operation: &Operation,
    dest: &Operand,
    src: &Operand,
) -> Result<DataSize, ExecutionError> {
    use DataSize::*;
    let op1_size = dest.get_data_size();
    let op2_size = src.get_data_size();
    if op1_size.is_none() && op2_size.is_none() {
        return Err(IllegalOperationError(
            current_operation.clone(),
            String::from("LD from address to address"),
        ));
    }
    if (op1_size.is_some() && op1_size.clone().unwrap() == BIT)
        || (op2_size.is_some() && op2_size.clone().unwrap() == BIT)
    {
        return Err(IllegalOperationError(
            current_operation.clone(),
            String::from("LD with bit size operand"),
        ));
    }
    if op1_size.is_none() {
        return Ok(op2_size.unwrap());
    }
    if op2_size.is_none() {
        return Ok(op1_size.unwrap());
    }
    if op1_size != op2_size {
        return Err(IllegalOperationError(
            current_operation.clone(),
            String::from("Mismatch data size on LD operators"),
        ));
    }
    return Ok(op1_size.unwrap());
}
