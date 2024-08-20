struct Chip8State {
    memory: [u8; 4096], // 4KB
    v: [u8; 16],        // V0 - VF
    i: u16,             // 12 bit register
    pc: u16,
    gfx: [u8; 64 * 32],
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; 16],
    sp: u16,
    key: [u8; 16], // 16 keys
}

fn fetch_instruction(state: &Chip8State) -> u16 {
    let pc = state.pc as usize;
    let byte1 = state.memory[pc] as u16;
    let byte2 = state.memory[pc + 1] as u16;
    byte1 << 8 | byte2
}

fn dump_memory(state: &Chip8State) {
    for i in 0..4096 {
        print!("{:02X} ", state.memory[i]);
        if i % 16 == 15 {
            println!();
        }
    }
}

fn main() -> ! {
    let rom = std::fs::read("rom/superpong.ch8").unwrap();

    let mut state = Chip8State {
        memory: [0; 4096],
        v: [0; 16],
        i: 0,
        pc: 0x200,
        gfx: [0; 64 * 32],
        delay_timer: 0,
        sound_timer: 0,
        stack: [0; 16],
        sp: 0,
        key: [0; 16],
    };

    for i in 0..rom.len() {
        state.memory[i + 0x200] = rom[i];
    }

    dump_memory(&state);

    loop {
        let opcode = fetch_instruction(&state);
        
        let inst = opcode & 0xF000;
        let address: u16 = opcode & 0x0FFF;

        println!("0x{:04X} 0x{:01X} 0x{:03X}", opcode, inst, address);

        panic!("Unexpected break statement");
    }
}
