mod hardware;
mod interpreter;
pub mod utils;

use hardware::cpu::CPU;
use hardware::memory::MemoryMap;
use interpreter::disassembler;

fn main() {
    let input = [
        //0b00001000, 0b00110111, 0b00010001, 0b00010010, 0b00011001, 0b00011111, 0b00110001,
        //0b11000001, 0b11111001, 0b00011001, 0b00110010, 0b00110001, 0b11000001, 0b11111001,
        //0b00111010, 0b00110111, 0b00010001, 0b00110001, 0b11011101, 0b00011001, 0b00011111,
        //0b00110001, 0b11000001, 0b11111001,
        0b00000001, 0b11111111, 0b10000000, 0b00001010, 0b00000110, 0b11111111,
    ];

    let mut mem_map = MemoryMap::new(24);
    let mut cpu = CPU::new();
    let mut head = 0;
    mem_map.write_bytes(head, input.to_vec()).unwrap();
    let program = disassembler::disassemble_program(&input).unwrap();
    println!("Full program:\n{:?}\n", program);
    loop {
        let next_bytes = mem_map.read_bytes(head, 3).unwrap();
        let (operation, offset) = disassembler::get_operation(&next_bytes).unwrap();
        println!("{:?}", operation);
        let _ = interpreter::execute(&mut mem_map, &mut cpu, &operation).unwrap();
        head += offset;
    }
}
