use core::panic;

const START:usize = 0x200;
const MEMORY_SIZE:usize = 4096;
const SCREEN_W:usize = 64;
const SCREEN_H:usize = 32;
const _FONT_START:usize = 0x050;

const CHIP8_FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];


#[allow(dead_code)]
pub struct Chip8 {
    memory: [u8; MEMORY_SIZE],
    v_register: [u8; 16],
    i_register: u16,
    pc: u16,
    stack: [u16; 16],
    stack_pointer: usize,
    delay_timer: u8,
    sound_timer: u8,
    screen: [bool; SCREEN_W*SCREEN_H],
    key_input: [bool; 16]
}

#[allow(dead_code)]
pub struct Instruction {
    self_type: u16,
    x: usize,
    y: usize,
    n: u8,
    nn: u8,
    nnn: u16,
    opcode: u16
}

impl Chip8 {
    pub fn read_rom(&mut self, rom: &[u8]) { // 롬 읽기
        const MEMORY_ROM_SIZE: usize = 4096 - 0x200;


        if rom.len() <= MEMORY_ROM_SIZE {
            self.memory[START..START+rom.len()].copy_from_slice(rom);
        }
        else {
            panic!("메모리 크기를 초과하였습니다.")
        }

        self.memory[0x050..0x050+CHIP8_FONTSET.len()].copy_from_slice(&CHIP8_FONTSET);
    }

    pub fn new() -> Self { // 초기화
        Chip8 {
            memory: [0; MEMORY_SIZE],
            v_register: [0; 16],
            i_register: 0,
            pc: 0x200,
            stack: [0; 16],
            stack_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,
            screen: [false; SCREEN_H*SCREEN_W],
            key_input: [false; 16]
        }
    }

    pub fn dump(&self, start_address:usize, rom_len: usize) { // 출력
        for (index, chunk) in self.memory[start_address..start_address+rom_len].chunks(16).enumerate() {
            print!("{:04X}: ", index*16+start_address);
            for byte in chunk {
                print!("{:02X} ", byte)
            }
            println!();
        }
    }

    pub fn fetch(&mut self) -> u16 { // 8byte -> 16byte
        let high: u8 = self.memory[self.pc as usize];
        let low: u8 = self.memory[self.pc as usize + 1];
        let opcode: u16 = (high as u16) <<8 | low as u16;
        self.pc += 2;
        println!("{:04X}", opcode);
        return opcode;
    }

    #[allow(non_snake_case)]
    pub fn decode(&self, opcode: u16) -> Instruction {
        let self_type = opcode >> 12;
        let X: usize = (opcode >> 8 &0xF) as usize;
        let Y: usize = (opcode >> 4 &0xF) as usize;
        let N: u8 = (opcode &0xF) as u8;
        let NN: u8 = (opcode &0xFF) as u8;
        let NNN: u16 = opcode &0xFFF;

        println!("{:04X}, {:04X}: {:0X}, {:0X}, {:0X}, {:0X}, {:02X}, {:03X}", self.pc - 2, opcode, self_type, X, Y, N, NN, NNN);
        let instruction = Instruction {
            self_type: self_type,
            x: X,
            y: Y,
            n: N,
            nn: NN,
            nnn: NNN,
            opcode: opcode
        };
        return instruction
    }

    #[allow(unused_variables, non_snake_case)]
    pub fn execute(&mut self, instruction: Instruction) { // 명령어 실행
        match instruction {
            Instruction { self_type: 0, nnn:0x0E0, .. } => self.screen.fill(false), // 화면 끄기
            Instruction { self_type:1, nnn, .. } => self.pc = nnn, 
            Instruction { self_type:6, x, nn, ..} => self.v_register[x] = nn,
            Instruction { self_type:7, x, nn,.. } => self.v_register[x] = self.v_register[x].wrapping_add(nn),
            Instruction { self_type:8, n:0, x, y,.. } => self.v_register[x] = self.v_register[y],
            Instruction { self_type:8, n:0x4, x, y, .. } => {
                let (result, overflowed) = self.v_register[x].overflowing_add(self.v_register[y]);
                self.v_register[x] = result;
                self.v_register[0xF] = overflowed as u8;
            }
            Instruction { self_type:0xA, nnn, ..} => self.i_register = nnn,
            Instruction { self_type:0xD, x, y, n, ..} => self.draw(x, y, n),
            Instruction { self_type:0xF, x, nn:0x07, .. } => self.v_register[x] = self.delay_timer,
            Instruction { self_type:0xF, x, nn:0x15, .. } => self.delay_timer = self.v_register[x],
            Instruction { self_type:0xF, x, nn:0x33, .. } => {
                self.memory[self.i_register as usize] = self.v_register[x] / 100;
                self.memory[(self.i_register + 1) as usize] = (self.v_register[x] % 100) / 10;
                self.memory[(self.i_register + 2) as usize] = self.v_register[x] % 10;
            },
            _ => panic!("알 수 없는  명령어  {:04X}", instruction.opcode)
        }
    }

    pub fn step(&mut self) {
        let opcode = self.fetch();

        self.execute(self.decode(opcode));
    }

    pub fn draw(&mut self, x:usize, y:usize, n:u8) {
        let x_coordinate = (self.v_register[x] % 64) as usize;
        let y_coordinate = (self.v_register[y] % 32) as usize;

        self.v_register[0xF] = 0;

        for r in 0..n as usize {
            let byte = self.memory[self.i_register as usize + r];
            if y_coordinate + r >= 32 {
                break;
            }
            for c in 0..8 {
                if x_coordinate + c >= 64 {
                    break;
                }
                if (byte >> (7-c)) & 1 == 1 {
                    if self.screen[(y_coordinate + r) * 64 + x_coordinate + c] {
                        self.v_register[0xF] = 1;
                    }
                    self.screen[(y_coordinate + r) * 64 + x_coordinate + c] ^= true;
                }
            }
        }
    }
}