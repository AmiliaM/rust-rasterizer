use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Point;
use std::time::Duration;


extern crate sdl2;
fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
 
    let window = video_subsystem.window("rust-sdl2 demo", 800, 600)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas: sdl2::render::Canvas<sdl2::video::Window> = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
        canvas.set_draw_color(Color::RGB(0, 255, 255));
        for y in 100..150 {
            line(100, 100, 150, y, &mut canvas);
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

fn line(x0: i32, y0: i32, x1: i32, y1: i32, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) { //Starting coordinate, finishing coordinate, reference to canvas
    //Bressenhalm line
    let dy = y1 - y0;
    let dx = x1 - x0;
    let mut error = dx / 2;
    let mut y = y0;
    for x in x0..x1 {
        error -= dy;
        canvas.draw_point(Point::new(x, y));
        if error <= 0 {
            y += 1;
            error += dx / 2;
        }
    }

}

fn ellipse(x: u32, y: u32, w: u32, h: u32) { //Center coordinate, width, height
    //Bressenhalm ellipse
}
