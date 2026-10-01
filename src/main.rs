mod chip8;
use crate::chip8::Chip8;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let result = std::fs::read(&args[1]).expect("ROM 파일을 읽을 수 없음");

    let mut chip8 = Chip8::new();

    chip8.read_rom(&result);
    chip8.dump(0x250,result.len());
    
    for _ in 0..50 { 
        chip8.step();
    }
}
