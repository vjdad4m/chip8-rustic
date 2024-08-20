use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::Sdl;
use sdl2::EventPump;

pub fn setup_display() -> (Canvas<Window>, EventPump) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("CHIP-8 Emulator", 640, 320)
        .position_centered()
        .build()
        .unwrap();
    let canvas = window.into_canvas().build().unwrap();
    let event_pump = sdl_context.event_pump().unwrap();
    (canvas, event_pump)
}

pub fn draw_screen_sdl(gfx: [u8; 64 * 32], canvas: &mut Canvas<Window>) {
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
