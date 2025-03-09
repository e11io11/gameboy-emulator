pub mod hardware;
mod interpreter;
mod vue;
pub mod utils;

use hardware::cpu::CPU;
use hardware::memory::MemoryMap;
use interpreter::disassembler;
use interpreter::disassembler::Operation;

//fn main() {
//    let input = [
//        //0b00001000, 0b00110111, 0b00010001, 0b00010010, 0b00011001, 0b00011111, 0b00110001,
//        //0b11000001, 0b11111001, 0b00011001, 0b00110010, 0b00110001, 0b11000001, 0b11111001,
//        //0b00111010, 0b00110111, 0b00010001, 0b00110001, 0b11011101, 0b00011001, 0b00011111,
//        //0b00110001, 0b11000001, 0b11111001,
//        0b00000001, 0b11111111, 0b10000000, 0b00001010, 0b00000110, 0b11111111,
//    ];
//    vue::main(&cpu);
//
//    let mut mem_map = MemoryMap::new(24);
//    let mut cpu = CPU::new();
//    let mut head = 0;
//    mem_map.write_bytes(head, input.to_vec()).unwrap();
//    let program = disassembler::disassemble_program(&input).unwrap();
//    println!("Full program:\n{:?}\n", program);
//    loop {
//        let next_bytes = mem_map.read_bytes(head, 3).unwrap();
//        let (operation, offset) = disassembler::get_operation(&next_bytes).unwrap();
//        println!("{:?}", operation);
//        let _ = interpreter::execute(&mut mem_map, &mut cpu, &operation).unwrap();
//        head += offset;
//    }
//}

use eframe::egui;

pub struct EmulatorApp {
    mem_map: MemoryMap,
    cpu: CPU,
    head: usize,
    step_flag: bool,
}

impl EmulatorApp {
    fn step(&mut self, operation: Operation, offset: usize) {
        println!("{:?}", operation);
        let _ = interpreter::execute(&mut self.mem_map, &mut self.cpu, &operation);
        self.head += offset;
    }

    fn next_operation(&mut self) -> (Operation, usize) {
        let next_bytes = self.mem_map.read_bytes(self.head, 3).unwrap();
        return disassembler::get_operation(&next_bytes).unwrap();
    }
}

impl eframe::App for EmulatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.step_flag = false;
        let (operation, offset) = self.next_operation();
        vue::debug::show(ctx, _frame, self, &operation);
        ctx.request_repaint();
        if self.step_flag {
            self.step(operation, offset);
        }
    }
}

fn main() -> eframe::Result<()> {
    let input = [
        //0b00001000, 0b00110111, 0b00010001, 0b00010010, 0b00011001, 0b00011111, 0b00110001,
        //0b11000001, 0b11111001, 0b00011001, 0b00110010, 0b00110001, 0b11000001, 0b11111001,
        //0b00111010, 0b00110111, 0b00010001, 0b00110001, 0b11011101, 0b00011001, 0b00011111,
        //0b00110001, 0b11000001, 0b11111001,
        0b00000001, 0b11111111, 0b10000000, 0b00001010, 0b00000110, 0b11111111,
    ];
    let program = disassembler::disassemble_program(&input).unwrap();
    println!("Full program:\n{:?}\n", program);
    let mut mem_map = MemoryMap::new();
    let cpu = CPU::new();
    let head = 0;

    mem_map.write_bytes(head, input.to_vec()).unwrap();

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Emulator",
        options,
        Box::new(|_cc| {
            Ok(Box::new(EmulatorApp {
                mem_map,
                cpu,
                head,
                step_flag: false,
            }))
        }),
    )
}
