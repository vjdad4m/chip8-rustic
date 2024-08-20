extern crate sdl2;

use sdl2::event::{self, Event};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::{EventPump, Sdl};

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

fn _draw_screen_console(gfx: [u8; 64 * 32]) {
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

fn draw_screen_sdl(gfx: [u8; 64 * 32], canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for y in 0..32 {
        for x in 0..64 {
            if gfx[y * 64 + x] != 0 {
                let _ = canvas.fill_rect(Rect::new(x as i32 * 10, y as i32 * 10, 10, 10));
            }
        }
    }
    canvas.present();
}

fn get_keypress(event_pump: &mut EventPump) -> u8 {
    loop {
        for event in event_pump.poll_iter() {
            match event {
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
                },
                _ => {}
            }
        }
    }
}

fn main() -> ! {
    let fps: u64 = 480;
    let cap_fps: bool = true;

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let window = video_subsystem.window("CHIP-8 Emulator", 640, 320)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

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
                    0x0 => {
                        state.v[x as usize] = state.v[y as usize];
                        state.pc += 2;
                    }
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
                    0x9E => {
                        if state.key[state.v[x as usize] as usize] == 1 {
                            state.pc += 4;
                        } else {
                            state.pc += 2;
                        }
                    }
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
                        let key = get_keypress(&mut event_pump);
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

        if state.delay_timer > 0 {
            state.delay_timer -= 1;
        }

        draw_screen_sdl(state.gfx, &mut canvas);

        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit {..} => std::process::exit(0),
                sdl2::event::Event::KeyDown { keycode: Some(keycode), .. } => {
                    match keycode {
                        sdl2::keyboard::Keycode::Num1 => state.key[0x1] = 1,
                        sdl2::keyboard::Keycode::Num2 => state.key[0x2] = 1,
                        sdl2::keyboard::Keycode::Num3 => state.key[0x3] = 1,
                        sdl2::keyboard::Keycode::Num4 => state.key[0xC] = 1,
                        sdl2::keyboard::Keycode::Q => state.key[0x4] = 1,
                        sdl2::keyboard::Keycode::W => state.key[0x5] = 1,
                        sdl2::keyboard::Keycode::E => state.key[0x6] = 1,
                        sdl2::keyboard::Keycode::R => state.key[0xD] = 1,
                        sdl2::keyboard::Keycode::A => state.key[0x7] = 1,
                        sdl2::keyboard::Keycode::S => state.key[0x8] = 1,
                        sdl2::keyboard::Keycode::D => state.key[0x9] = 1,
                        sdl2::keyboard::Keycode::F => state.key[0xE] = 1,
                        sdl2::keyboard::Keycode::Z => state.key[0xA] = 1,
                        sdl2::keyboard::Keycode::X => state.key[0x0] = 1,
                        sdl2::keyboard::Keycode::C => state.key[0xB] = 1,
                        sdl2::keyboard::Keycode::V => state.key[0xF] = 1,
                        _ => {}
                    }
                },
                sdl2::event::Event::KeyUp { keycode: Some(keycode), .. } => {
                    match keycode {
                        sdl2::keyboard::Keycode::Num1 => state.key[0x1] = 0,
                        sdl2::keyboard::Keycode::Num2 => state.key[0x2] = 0,
                        sdl2::keyboard::Keycode::Num3 => state.key[0x3] = 0,
                        sdl2::keyboard::Keycode::Num4 => state.key[0xC] = 0,
                        sdl2::keyboard::Keycode::Q => state.key[0x4] = 0,
                        sdl2::keyboard::Keycode::W => state.key[0x5] = 0,
                        sdl2::keyboard::Keycode::E => state.key[0x6] = 0,
                        sdl2::keyboard::Keycode::R => state.key[0xD] = 0,
                        sdl2::keyboard::Keycode::A => state.key[0x7] = 0,
                        sdl2::keyboard::Keycode::S => state.key[0x8] = 0,
                        sdl2::keyboard::Keycode::D => state.key[0x9] = 0,
                        sdl2::keyboard::Keycode::F => state.key[0xE] = 0,
                        sdl2::keyboard::Keycode::Z => state.key[0xA] = 0,
                        sdl2::keyboard::Keycode::X => state.key[0x0] = 0,
                        sdl2::keyboard::Keycode::C => state.key[0xB] = 0,
                        sdl2::keyboard::Keycode::V => state.key[0xF] = 0,
                        _ => {}
                    }
                },
                _ => {}
            }
        }

        if cap_fps { std::thread::sleep(std::time::Duration::from_millis(1000 / fps)) };
    }
}
