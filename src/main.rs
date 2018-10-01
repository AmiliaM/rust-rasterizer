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
    let mut canvas: sdl2::render::Canvas<sdl2::video::Window> = window.into_canvas().accelerated().present_vsync().build().unwrap();
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
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.set_draw_color(Color::RGB(0, 255, 255));
        for y in 0..200 {
            line(100, 100, 200, y, &mut canvas);
        }
        //rect(200, 200, 300, 300, &mut canvas);
        //ellipse(200, 200, 100, 100, &mut canvas);
        canvas.present();
        let error = ::sdl2::get_error();
        if error != "" {
            println!("{}", error);
        }
        //::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}



fn line(x0: i32, y0: i32, x1: i32, y1: i32, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) { //Starting coordinate, finishing coordinate, reference to canvas
    //Bressenhalm line
    let dy = y1 - y0;
    let dx = x1 - x0;
    let mut error = dx / 2;
    let mut y = y0;
    /*let mut y_step;
    if dx > 0 {
        y_step = 1;
    }
    else {
        y_step = -1;
    }*/

    for x in x0..x1 {
        error -= dy;
        canvas.draw_point(Point::new(x, y));
        if error <= 0 {
            y += 1;
            error += dx / 2;
        }
    }
}

fn rect(x0: i32, y0: i32, x1: i32, y1: i32, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
    line(x0, y0, x1, y0, canvas);
    line(x0, y0, x0, y1, canvas);
    line(x1, y0, x1, y1, canvas);
    line(x0, y1, x1, y1, canvas);
}

fn ellipse_point(x: i32, y: i32, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
    canvas.draw_point(Point::new(x, y));
    canvas.draw_point(Point::new(-x, y));
    canvas.draw_point(Point::new(x, -y));
    canvas.draw_point(Point::new(-x, -y));

    canvas.draw_point(Point::new(y, x));
    canvas.draw_point(Point::new(-y, x));
    canvas.draw_point(Point::new(y, -x));
    canvas.draw_point(Point::new(-y, -x));
}

fn ellipse(x_: i32, y_: i32, a: i32, b: i32, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) { //Center coordinate, width, height
    //Bressenhalm ellipse
    //let x_ = x;
    //let y_ = y;
    let mut x = 0;
    let mut y = b;
    let mut d1 = (b.pow(2)) - ((a.pow(2))*b) + (a.pow(2))/4;
    ellipse_point(x + x_, y + y_, canvas);
    while (a.pow(2))*y >(b.pow(2))*(x+1){ //y- 0.5
        if d1 < 0 {
            d1 += (b.pow(2))*(2*x+3);
        }
        else {
            d1 += (b.pow(2))*(2*x+3)+(a.pow(2))*(-2*y+2);
            y-=1;
        }
        x+=1;
        ellipse_point(x + x_, y + y_, canvas);
    }

}
