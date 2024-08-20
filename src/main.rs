extern crate sdl2;

mod display;
mod processor;

use clap::{Arg, Command};
use display::{draw_screen_sdl, setup_display};
use processor::{fetch_instruction, handle_keypress, process_instruction, Chip8State};
use std::fs;
use std::time::{Duration, Instant};

fn main() -> ! {
    let matches = Command::new("CHIP-8 Emulator")
        .version("1.0")
        .author("Adam Vajda <adam@vayda.xyz>")
        .about("Emulates CHIP-8 programs")
        .arg(
            Arg::new("rom")
                .help("Sets the ROM file to use")
                .required(false)
                .index(1)
                .default_value("rom/superpong.ch8"),
        )
        .arg(
            Arg::new("fps")
                .help("Sets the frames per second")
                .long("fps")
                .default_value("1000")
                .value_parser(clap::value_parser!(u64)),
        )
        .arg(
            Arg::new("cap_fps")
                .help("Caps the frames per second")
                .long("cap-fps")
                .default_value("true")
                .value_parser(clap::value_parser!(bool)),
        )
        .get_matches();

    let rom_path = matches.get_one::<String>("rom").unwrap();

    let fps: u64 = *matches.get_one::<u64>("fps").unwrap();
    let cap_fps: bool = *matches.get_one::<bool>("cap_fps").unwrap();

    let (mut canvas, mut event_pump) = setup_display();
    let frame_duration = Duration::from_secs_f64(1.0 / fps as f64);

    let rom = fs::read(rom_path).unwrap_or_else(|err| {
        eprintln!("Failed to read ROM file {}: {}", rom_path, err);
        std::process::exit(1);
    });

    let mut state = Chip8State::new();
    state.load_rom(&rom);

    loop {
        let frame_start = Instant::now();

        let opcode = fetch_instruction(&state);
        process_instruction(&mut state, opcode, &mut event_pump);

        handle_keypress(&mut state, &mut event_pump);

        draw_screen_sdl(state.gfx, &mut canvas);

        if cap_fps {
            let elapsed = frame_start.elapsed();
            if elapsed < frame_duration {
                std::thread::sleep(frame_duration - elapsed);
            }
        }
    }
}
