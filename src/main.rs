extern crate sdl2;

mod display;
mod processor;

use std::fs;
use display::{setup_display, draw_screen_sdl};
use processor::{Chip8State, fetch_instruction, process_instruction, handle_keypress};

fn main() -> ! {
    let fps: u64 = 1000;
    let cap_fps: bool = true;

    let (mut canvas, mut event_pump) = setup_display();

    let rom = fs::read("rom/superpong.ch8").unwrap();
    let mut state = Chip8State::new();
    state.load_rom(&rom);

    loop {
        let opcode = fetch_instruction(&state);
        process_instruction(&mut state, opcode, &mut event_pump);

        handle_keypress(&mut state, &mut event_pump);

        draw_screen_sdl(state.gfx, &mut canvas);

        if cap_fps { std::thread::sleep(std::time::Duration::from_millis(1000 / fps)); }
    }
}
