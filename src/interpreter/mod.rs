pub mod disassembler;
use crate::hardware::cpu::CPU;
use crate::hardware::memory::MemoryMap;
use crate::utils::DataSize;
use disassembler::Operand;
use disassembler::Operation;

#[derive(Debug)]
pub struct IllegalOperationError(Operation, String);

pub fn execute(
    mem_map: &mut MemoryMap,
    cpu: &mut CPU,
    operation: Operation,
) -> Result<(), IllegalOperationError> {
    match operation {
        Operation::LD(dest, src) => execute_ld(mem_map, cpu, &dest, &src)?,
        _ => todo!(),
    }
    return Ok(());
}

pub fn execute_ld(
    _mem_map: &mut MemoryMap,
    cpu: &mut CPU,
    dest: &Operand,
    src: &Operand,
) -> Result<(), IllegalOperationError> {
    let data_size = get_ld_data_size(dest, src)?;
    let src_value = match src {
        Operand::Register(r) => r.read(cpu),
        Operand::Address(_) => todo!(),
        Operand::Decr(_r) => todo!(),
        Operand::Incr(_r) => todo!(),
        Operand::Not(_) => todo!(),
        Operand::Byte(_) => todo!(),
        Operand::Word(_) => todo!(),
    };
    match dest {
        Operand::Register(r) => r.write(cpu, src_value),
        _ => todo!()
    }
    return Ok(());
}

fn get_ld_data_size(dest: &Operand, src: &Operand) -> Result<DataSize, IllegalOperationError> {
    use DataSize::*;
    let op1_size = dest.get_data_size();
    let op2_size = src.get_data_size();
    if op1_size.is_none() && op2_size.is_none() {
        return Err(IllegalOperationError(
            Operation::LD(dest.clone(), src.clone()),
            String::from("LD from address to address"),
        ));
    }
    if (op1_size.is_some() && op1_size.clone().unwrap() == BIT)
        || (op2_size.is_some() && op2_size.clone().unwrap() == BIT)
    {
        return Err(IllegalOperationError(
            Operation::LD(dest.clone(), src.clone()),
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
            Operation::LD(dest.clone(), src.clone()),
            String::from("Mismatch data size on LD operators"),
        ));
    }
    return Ok(op1_size.unwrap());
}
