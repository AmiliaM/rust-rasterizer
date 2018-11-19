extern crate sdl2;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

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

        let mut points: Vec<(i32, i32)> = Vec::new();
        
        /*
        for y in 100..300 {
            //points.extend(line((200, 200), (300, y)));
            //points.extend(line((200, 200), (100, y)));
        }
        for x in 100..300 {
            //points.extend(line((200, 200), (x, 300)));
            //points.extend(line((200, 200), (x, 100)));
        }*/

        //points.extend(rect(200, 200, 300, 300));
        //points.extend(ellipse(200, 200, 200, 200));

        //points.extend(line((200, 200), (300, 300)));

        draw_points(points, &mut canvas);

        canvas.present();
        let error = ::sdl2::get_error();
        if error != "" {
            println!("{}", error);
        }
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

fn draw_points(points: Vec<(i32, i32)>, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
    for p in points {
        match canvas.draw_point(p) {
            Ok(_) => {},
            Err(e) => println!("Error: {}", e)
        }
    }
}

fn line(p0: (i32, i32), p1: (i32, i32)) -> Vec<(i32, i32)> { //Starting coordinate, finishing coordinate
    let dx = p1.0 - p0.0;
    let dy = p1.1 - p0.1;
    let m = {
            if dy == 0 {
                0
            }
            else {
                dx/dy
            }
    };
    let mut points: Vec<(i32, i32)> = vec!((p0.0, p0.1), (p1.0, p1.1));
    let mut y = p0.1;
    for x in p0.0..p1.0 {
        y+= m;
        points.push((x, (y as f32 + 0.5).floor() as i32));
    }
    points
}

fn rect(p0: (i32, i32), p1: (i32, i32)) -> Vec<(i32, i32)> {
    let mut points = line((p0.0, p0.1), (p1.0, p0.1));
    points.extend(line((p0.0, p0.1), (p0.0, p1.1)));
    points.extend(line((p1.0, p0.1), (p1.0, p1.1)));
    points.extend(line((p0.0, p1.1), (p1.0, p1.1)));
    points
}

fn ellipse_points(x: i32, y: i32, p0: (i32, i32)) -> Vec<(i32, i32)> {
    let mut points: Vec<(i32, i32)> = Vec::new();
    points.push((x + p0.0, y + p0.1));
    points.push((-x + p0.0, y + p0.1));
    points.push((x + p0.0, -y + p0.1));
    points.push((-x + p0.0, -y + p0.1));
    points
}
fn ellipse_points2(x: i32, y: i32, p0: (i32, i32)) -> Vec<(i32, i32)> {
    let mut points: Vec<(i32, i32)> = Vec::new();
    points.push((y + p0.0, x + p0.1));
    points.push((-y + p0.0, x + p0.1));
    points.push((y + p0.0, -x + p0.1));
    points.push((-y + p0.0, -x + p0.1));
    points
}

fn ellipse(p0: (i32, i32), a: i32, b: i32) -> Vec<(i32, i32)> { //Center coordinate, width, height
    let mut x = 0;
    let mut y = b;
    let mut d1 = (b.pow(2)) - ((a.pow(2))*b) + (a.pow(2))/4;
    let mut points = ellipse_points(x, y, p0);
    while (a.pow(2))*y >(b.pow(2))*(x+1){ //y- 0.5
        if d1 < 0 {
            d1 += (b.pow(2))*(2*x+3);
        }
        else {
            d1 += (b.pow(2))*(2*x+3)+(a.pow(2))*(-2*y+2);
            y-=1;
        }
        x+=1;
        points.extend(ellipse_points(x, y, p0));
    }

    let mut x = 0;
    let mut y = b;
    let mut d1 = (b.pow(2)) - ((a.pow(2))*b) + (a.pow(2))/4;
    points.extend(ellipse_points2(x, y, p0));
    while (a.pow(2))*y >(b.pow(2))*(x+1){ //y- 0.5
        if d1 < 0 {
            d1 += (b.pow(2))*(2*x+3);
        }
        else {
            d1 += (b.pow(2))*(2*x+3)+(a.pow(2))*(-2*y+2);
            y-=1;
        }
        x+=1;
        points.extend(ellipse_points2(x, y, p0));
    }
    points
}
