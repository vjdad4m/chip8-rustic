extern crate sdl2;

mod display;
mod processor;

use clap::{Arg, Command};
use std::fs;
use display::{setup_display, draw_screen_sdl};
use processor::{Chip8State, fetch_instruction, process_instruction, handle_keypress};

fn main() -> ! {
    let matches = Command::new("CHIP-8 Emulator")
        .version("1.0")
        .author("Your Name <adam@vayda.xyz>")
        .about("Emulates CHIP-8 programs")
        .arg(
            Arg::new("rom")
                .help("Sets the ROM file to use")
                .required(false)
                .index(1)
                .default_value("rom/superpong.ch8"),
        )
        .get_matches();

    let rom_path = matches.get_one::<String>("rom").unwrap();

    let fps: u64 = 1000;
    let cap_fps: bool = true;

    let (mut canvas, mut event_pump) = setup_display();

    let rom = fs::read(rom_path).unwrap_or_else(|err| {
        eprintln!("Failed to read ROM file {}: {}", rom_path, err);
        std::process::exit(1);
    });

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
