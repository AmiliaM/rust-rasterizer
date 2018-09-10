use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::*;
use std::time::Duration;

extern crate sdl2;
fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
 
    let window = video_subsystem.window("rust-sdl2 demo", 800, 600)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
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
        //H
        canvas.draw_line(Point::new(50,100), Point::new(50,300));
        canvas.draw_line(Point::new(50,200), Point::new(150,200));
        canvas.draw_line(Point::new(150,100), Point::new(150,300));
        //E
        canvas.draw_line(Point::new(200,100), Point::new(200,300));
        canvas.draw_line(Point::new(200,200), Point::new(250,200));
        canvas.draw_line(Point::new(200,100), Point::new(250,100));
        canvas.draw_line(Point::new(200,300), Point::new(250,300));
        //L
        canvas.draw_line(Point::new(300,100), Point::new(300,300));
        //L
        canvas.draw_line(Point::new(350,100), Point::new(350,300));
        //O
        canvas.draw_line(Point::new(400,100), Point::new(400,300));
        canvas.draw_line(Point::new(400,100), Point::new(450,100));
        canvas.draw_line(Point::new(400,300), Point::new(450,300));
        canvas.draw_line(Point::new(450,100), Point::new(450,300));
        //\!
        canvas.draw_line(Point::new(500,100), Point::new(500,250));
        canvas.draw_point(Point::new(500,300));

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
