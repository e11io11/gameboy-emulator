use disassembler::disassemble;

mod disassembler;
mod hardware;

fn main() {
    let input = [
        0b00001000, 0b00110111, 0b00010001, 0b00010010, 0b00011001, 0b00011111, 0b00110001,
        0b11000001, 0b11111001, 0b00011001, 0b00110010, 0b00110001, 0b11000001, 0b11111001,
        0b00111010, 0b00110111, 0b00010001, 0b00110001, 0b11011101, 0b00011001, 0b00011111,
        0b00110001, 0b11000001, 0b11111001,
    ];
    match disassemble(&input) {
        Ok(instructions) => println!("{:?}", instructions),
        Err(e) => println!("{:?}", e),
    }
}
