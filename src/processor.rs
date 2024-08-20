use sdl2::{event::Event, keyboard::Keycode, EventPump};

pub struct Chip8State {
    pub memory: [u8; 4096], // 4KB
    pub v: [u8; 16], // V0 - VF
    pub i: u16, // 12 bit register
    pub pc: u16,
    pub gfx: [u8; 64 * 32],
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub stack: [u16; 16],
    pub sp: u16,
    pub key: [u8; 16], // 16 keys
}

impl Chip8State {
    pub fn new() -> Self {
        Self {
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
        }
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        for i in 0..rom.len() {
            self.memory[i + 0x200] = rom[i];
        }
    }
}

pub fn fetch_instruction(state: &Chip8State) -> u16 {
    let pc = state.pc as usize;
    let byte1 = state.memory[pc] as u16;
    let byte2 = state.memory[pc + 1] as u16;
    byte1 << 8 | byte2
}

pub fn process_instruction(state: &mut Chip8State, opcode: u16, event_pump: &mut EventPump) {
    let inst = opcode & 0xF000;
    let address: u16 = opcode & 0x0FFF;
    let x: u16 = (opcode & 0x0F00) >> 8;
    let y: u16 = (opcode & 0x00F0) >> 4;
    let n: u16 = opcode & 0x000F;
    let kk: u16 = opcode & 0x00FF;

    match inst {
        0x0000 => {
            match opcode {
                // clear
                0x00E0 => {
                    state.gfx = [0; 64 * 32];
                    state.pc += 2;
                }
                // return
                0x00EE =>  {
                    state.sp -= 1;
                    state.pc = state.stack[state.sp as usize];
                    state.pc += 2;
                }
                // 0x0N 0xNN
                _ => {
                    state.pc = address;
                }
            }
        }
        // jump NNN
        0x1000 => {
            state.pc = address;
        }
        // :call NNN
        0x2000 => {
            state.stack[state.sp as usize] = state.pc;
            state.sp += 1;
            state.pc = address;
        }
        // if vX != NN then
        0x3000 => {
            if state.v[x as usize] == kk as u8 { state.pc += 4; }
            else { state.pc += 2; }
        }
        // if vX == NN then
        0x4000 => {
            if state.v[x as usize] != kk as u8 { state.pc += 4; }
            else { state.pc += 2; }
        }
        // if vX != vY then
        0x5000 => {
            if state.v[x as usize] == state.v[y as usize] { state.pc += 4; }
            else { state.pc += 2; }
        }
        // vX := NN
        0x6000 => {
            state.v[x as usize] = kk as u8;
            state.pc += 2;
        }
        // vX += NN
        0x7000 => {
            state.v[x as usize] = (state.v[x as usize] as u16 + kk) as u8;
            state.pc += 2;
        }
        0x8000 => {
            match n {
                // vX := vY
                0x0 => {
                    state.v[x as usize] = state.v[y as usize];
                    state.pc += 2;
                }
                // vX += vY
                0x4 => {
                    let sum = state.v[x as usize] as u16 + state.v[y as usize] as u16;
                    state.v[0xF] = if sum > 0xFF { 1 } else { 0 };
                    state.v[x as usize] = sum as u8;
                    state.pc += 2;
                }
                // vX -= vY
                0x5 => {
                    if state.v[x as usize] > state.v[y as usize] {
                        state.v[0xF] = 1;
                    } else {
                        state.v[0xF] = 0;
                    }
                    state.v[x as usize] = state.v[x as usize].wrapping_sub(state.v[y as usize]);
                    state.pc += 2;
                }
                // vX =- vY
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
        // if vX == vY then
        0x9000 => {
            if state.v[x as usize] != state.v[y as usize] { state.pc += 4; }
            else { state.pc += 2; }
        }
        // i := NNN
        0xA000 => {
            state.i = address;
            state.pc += 2;
        }
        // jump0 NNN
        0xB000 => {
            state.pc = address + state.v[0] as u16;
        }
        // vX := random NN
        0xC000 => {
            state.v[x as usize] = rand::random::<u8>() & kk as u8;
            state.pc += 2;
        }
        // sprite vX vY N
        0xD000 => {
            let x = state.v[x as usize] as usize;
            let y = state.v[y as usize] as usize;
            let height = n as usize;
            state.v[0xF] = 0;
            for yline in 0..height {
                let pixel = state.memory[state.i as usize + yline];
                for xline in 0..8 {
                    if (pixel & (0x80 >> xline)) != 0 {
                        let pos = (x + xline + ((y + yline) * 64)) % (64 * 32);
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
                // if vX -key then
                0x9E => {
                    if state.key[state.v[x as usize] as usize] == 1 {
                        state.pc += 4;
                    } else {
                        state.pc += 2;
                    }
                }
                // if vX key then
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
                // plane X
                0x01 => {
                    // IGNORE
                    state.pc += 2;
                }
                // vX := delay
                0x07 => {
                    state.v[x as usize] = state.delay_timer;
                    state.pc += 2;
                }
                // vX := key
                0x0A => {
                    let key = get_latest_keypress(event_pump);
                    state.v[x as usize] = key;
                    state.pc += 2;
                }
                // delay := vX
                0x15 => {
                    state.delay_timer = state.v[x as usize];
                    state.pc += 2;
                }
                // buzzer := vX
                0x18 => {
                    state.sound_timer = state.v[x as usize];
                    state.pc += 2;
                }
                // i += vX
                0x1E => {
                    state.i += state.v[x as usize] as u16;
                    state.pc += 2;
                }
                // i := hex vX
                0x29 => {
                    state.i = state.v[x as usize] as u16 * 5;
                    state.pc += 2;
                }
                // bcd vX
                0x33 => {
                    state.memory[state.i as usize] = state.v[x as usize] / 100;
                    state.memory[state.i as usize + 1] = (state.v[x as usize] / 10) % 10;
                    state.memory[state.i as usize + 2] = state.v[x as usize] % 10;
                    state.pc += 2;
                }
                // save vX
                0x55 => {
                    for i in 0..x+1 {
                        state.memory[(state.i + i as u16) as usize] = state.v[i as usize];
                    }
                    state.i += x + 1;
                    state.pc += 2;
                }
                // load vX
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

    // Update timers
    if state.delay_timer > 0 {
        state.delay_timer -= 1;
    }
    if state.sound_timer > 0 {
        state.sound_timer -= 1;
    }
}

pub fn get_latest_keypress(event_pump: &mut EventPump) -> u8 {
    loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => std::process::exit(0),
                Event::KeyDown { keycode: Some(keycode), .. } => {
                    match keycode {
                        Keycode::Num1 => return 0x1,
                        Keycode::Num2 => return 0x2,
                        Keycode::Num3 => return 0x3,
                        Keycode::Num4 => return 0xC,
                        Keycode::Q => return 0x4,
                        Keycode::W => return 0x5,
                        Keycode::E => return 0x6,
                        Keycode::R => return 0xD,
                        Keycode::A => return 0x7,
                        Keycode::S => return 0x8,
                        Keycode::D => return 0x9,
                        Keycode::F => return 0xE,
                        Keycode::Z => return 0xA,
                        Keycode::X => return 0x0,
                        Keycode::C => return 0xB,
                        Keycode::V => return 0xF,
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}

pub fn handle_keypress(state: &mut Chip8State, event_pump: &mut EventPump) {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. } => std::process::exit(0),
            Event::KeyDown { keycode: Some(keycode), .. } => {
                match keycode {
                    Keycode::Num1 => state.key[0x1] = 1,
                    Keycode::Num2 => state.key[0x2] = 1,
                    Keycode::Num3 => state.key[0x3] = 1,
                    Keycode::Num4 => state.key[0xC] = 1,
                    Keycode::Q => state.key[0x4] = 1,
                    Keycode::W => state.key[0x5] = 1,
                    Keycode::E => state.key[0x6] = 1,
                    Keycode::R => state.key[0xD] = 1,
                    Keycode::A => state.key[0x7] = 1,
                    Keycode::S => state.key[0x8] = 1,
                    Keycode::D => state.key[0x9] = 1,
                    Keycode::F => state.key[0xE] = 1,
                    Keycode::Z => state.key[0xA] = 1,
                    Keycode::X => state.key[0x0] = 1,
                    Keycode::C => state.key[0xB] = 1,
                    Keycode::V => state.key[0xF] = 1,
                    _ => {}
                }
            }
            Event::KeyUp { keycode: Some(keycode), .. } => {
                match keycode {
                    Keycode::Num1 => state.key[0x1] = 0,
                    Keycode::Num2 => state.key[0x2] = 0,
                    Keycode::Num3 => state.key[0x3] = 0,
                    Keycode::Num4 => state.key[0xC] = 0,
                    Keycode::Q => state.key[0x4] = 0,
                    Keycode::W => state.key[0x5] = 0,
                    Keycode::E => state.key[0x6] = 0,
                    Keycode::R => state.key[0xD] = 0,
                    Keycode::A => state.key[0x7] = 0,
                    Keycode::S => state.key[0x8] = 0,
                    Keycode::D => state.key[0x9] = 0,
                    Keycode::F => state.key[0xE] = 0,
                    Keycode::Z => state.key[0xA] = 0,
                    Keycode::X => state.key[0x0] = 0,
                    Keycode::C => state.key[0xB] = 0,
                    Keycode::V => state.key[0xF] = 0,
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
