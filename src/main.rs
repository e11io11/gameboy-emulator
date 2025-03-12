pub mod hardware;
mod interpreter;
pub mod utils;
mod vue;

use hardware::cpu::CPU;
use hardware::cpu::Register;
use hardware::memory::MemoryMap;
use interpreter::disassembler;
use interpreter::disassembler::Operation;

use eframe::egui;

pub struct EmulatorApp {
    mem_map: MemoryMap,
    cpu: CPU,
    step_flag: bool,
}

impl EmulatorApp {
    fn step(&mut self, operation: Operation) {
        println!("{:?}", operation);
        self.cpu
            .incr_word(&Register::PC, operation.get_size() as u16);
        let _ = interpreter::execute(&mut self.mem_map, &mut self.cpu, &operation);
    }

    fn next_operation(&mut self) -> Operation {
        let next_bytes = self
            .mem_map
            .read_bytes(self.cpu.read_word(&Register::PC) as usize, 3)
            .unwrap();
        return disassembler::get_operation(&next_bytes).unwrap();
    }
}

impl eframe::App for EmulatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.step_flag = false;
        let operation = self.next_operation();
        vue::debug::show(ctx, _frame, self, &operation);
        ctx.request_repaint();
        if self.step_flag {
            self.step(operation);
        }
    }
}

fn main() -> eframe::Result<()> {
    let input = [
        //0b00001000, 0b00110111, 0b00010001, 0b00010010, 0b00011001, 0b00011111, 0b00110001,
        //0b11000001, 0b11111001, 0b00011001, 0b00110010, 0b00110001, 0b11000001, 0b11111001,
        //0b00111010, 0b00110111, 0b00010001, 0b00110001, 0b11011101, 0b00011001, 0b00011111,
        //0b00110001, 0b11000001, 0b11111001,
        0b00000001, 0b11111111, 0b10000000, 0b00001010, 0b00000110, 0b11111111, 0b00110010,
        0b00101010, 0b00110110, 0b11111111,
    ];
    let program = disassembler::disassemble_program(&input).unwrap();
    println!("Full program:\n{:?}\n", program);
    let mut mem_map = MemoryMap::new();
    let cpu = CPU::new();

    mem_map.write_bytes(0, input.to_vec()).unwrap();

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Emulator",
        options,
        Box::new(|_cc| {
            Ok(Box::new(EmulatorApp {
                mem_map,
                cpu,
                step_flag: false,
            }))
        }),
    )
}
