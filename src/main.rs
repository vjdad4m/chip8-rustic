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

fn get_input() -> u8 {
    use std::collections::HashMap;
    
    let key_mapping: HashMap<String, u8> = [
        ("0".to_string(), 0x00),
        ("1".to_string(), 0x01),
        ("2".to_string(), 0x02),
        ("3".to_string(), 0x03),
        ("4".to_string(), 0x04),
        ("5".to_string(), 0x05),
        ("6".to_string(), 0x06),
        ("7".to_string(), 0x07),
        ("8".to_string(), 0x08),
        ("9".to_string(), 0x09),
        ("a".to_string(), 0x0A),
        ("b".to_string(), 0x0B),
        ("c".to_string(), 0x0C),
        ("d".to_string(), 0x0D),
        ("e".to_string(), 0x0E),
        ("f".to_string(), 0x0F),
    ].iter().cloned().collect();

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let input = input.trim().to_lowercase();
    let key = key_mapping.get(&input);
    match key {
        Some(k) => *k,
        None => 0
    }
}

fn draw_screen(gfx: [u8; 64 * 32]) {
    for y in 0..32 {
        for x in 0..64 {
            let pixel = gfx[y * 64 + x];
            if pixel == 0 {
                print!(" ");
            } else {
                print!("█");
            }
        }
        println!();
    }
}

fn main() -> ! {
    let fps: u64 = 60;
    let cap_fps: bool = false;

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
        let x: u16 = (opcode & 0x0F00) >> 8;
        let y: u16 = (opcode & 0x00F0) >> 4;
        let n: u16 = opcode & 0x000F;
        let kk: u16 = opcode & 0x00FF;

        println!("0x{:04X} 0x{:01X} 0x{:03X}", opcode, inst, address);

        match inst {
            0x0000 => {
                match opcode {
                    0x00E0 => {
                        state.gfx = [0; 64 * 32];
                        state.pc += 2;
                    }
                    0x00EE =>  {
                        state.sp -= 1;
                        state.pc = state.stack[state.sp as usize];
                        state.pc += 2;
                    }
                    _ => {
                        panic!("Unknown instruction: 0x{:04X} (0x{:4X})", inst, opcode);
                    }
                }
            }
            0x1000 => {
                state.pc = address;
            }
            0x2000 => {
                state.stack[state.sp as usize] = state.pc;
                state.sp += 1;
                state.pc = address;
            }
            0x3000 => {
                if state.v[x as usize] == kk as u8 { state.pc += 4; }
                else { state.pc += 2; }
            }
            0x4000 => {
                if state.v[x as usize] != kk as u8 { state.pc += 4; }
                else { state.pc += 2; }
            }
            0x6000 => {
                state.v[x as usize] = kk as u8;
                state.pc += 2;
            }
            0x7000 => {
                state.v[x as usize] = (state.v[x as usize] as u16 + kk) as u8;
                state.pc += 2;
            }
            0x8000 => {
                match n {
                    0x4 => {
                        let sum = state.v[x as usize] as u16 + state.v[y as usize] as u16;
                        state.v[0xF] = if sum > 0xFF { 1 } else { 0 };
                        state.v[x as usize] = sum as u8;
                        state.pc += 2;
                    }
                    0x5 => {
                        if state.v[x as usize] > state.v[y as usize] {
                            state.v[0xF] = 1;
                        } else {
                            state.v[0xF] = 0;
                        }
                        state.v[x as usize] = state.v[x as usize].wrapping_sub(state.v[y as usize]);
                        state.pc += 2;
                    }
                    0x7 => {
                        if state.v[y as usize] > state.v[x as usize] {
                            state.v[0xF] = 1;
                        } else {
                            state.v[0xF] = 0;
                        }
                        state.v[x as usize] = state.v[y as usize].wrapping_sub(state.v[x as usize]);
                        state.pc += 2;
                    }
                    _ => {
                        panic!("Unknown instruction: 0x{:04X} (0x{:4X})", inst, opcode);
                    }
                }
            }
            0xA000 => {
                state.i = address;
                state.pc += 2;
            }
            0xC000 => {
                state.v[x as usize] = rand::random::<u8>() & kk as u8;
                state.pc += 2;
            }
            0xD000 => {
                let x = state.v[x as usize] as usize;
                let y = state.v[y as usize] as usize;
                let height = n as usize;
                state.v[0xF] = 0;
                for yline in 0..height {
                    let pixel = state.memory[state.i as usize + yline];
                    for xline in 0..8 {
                        if (pixel & (0x80 >> xline)) != 0 {
                            let pos = x + xline + ((y + yline) * 64);
                            if state.gfx[pos] == 1 {
                                state.v[0xF] = 1;
                            }
                            state.gfx[pos] ^= 1;
                        }
                    }
                }
                state.pc += 2;
            }
            0xE000 => {
                match kk {
                    0xA1 => {
                        if state.key[state.v[x as usize] as usize] == 0 {
                            state.pc += 4;
                        } else {
                            state.pc += 2;
                        }
                    }
                    _ => {
                        panic!("Unknown instruction: 0x{:04X} (0x{:4X})", inst, opcode);
                    }
                }
            }
            0xF000 => {
                match kk {
                    0x07 => {
                        state.v[x as usize] = state.delay_timer;
                        state.pc += 2;
                    }
                    0x0A => {
                        let key = get_input();
                        println!("Key: 0x{:02X}", key);
                        state.v[x as usize] = key;
                        state.pc += 2;
                    }
                    0x15 => {
                        state.delay_timer = state.v[x as usize];
                        state.pc += 2;
                    }
                    0x1E => {
                        state.i += state.v[x as usize] as u16;
                        state.pc += 2;
                    }
                    0x29 => {
                        state.i = state.v[x as usize] as u16 * 5;
                        state.pc += 2;
                    }
                    0x33 => {
                        state.memory[state.i as usize] = state.v[x as usize] / 100;
                        state.memory[state.i as usize + 1] = (state.v[x as usize] / 10) % 10;
                        state.memory[state.i as usize + 2] = state.v[x as usize] % 10;
                        state.pc += 2;
                    }
                    0x65 => {
                        for i in 0..x+1 {
                            state.v[i as usize] = state.memory[(state.i + i as u16) as usize];
                        }
                        state.i += x + 1;
                        state.pc += 2;
                    }
                    _ => {
                        panic!("Unknown instruction: 0x{:04X} (0x{:4X})", inst, opcode);
                    }
                }
            }
            _ => {
                panic!("Unknown instruction: 0x{:04X} (0x{:4X})", inst, opcode);
            }
        }

        draw_screen(state.gfx);

        if cap_fps { std::thread::sleep(std::time::Duration::from_millis(1000 / fps)) };
    }
}
