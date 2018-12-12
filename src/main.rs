#![feature(test)]
extern crate test;
extern crate sdl2;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

type Point = (i32, i32);

pub trait VecExt {
    fn scissor(&mut self, p0: Point, p1: Point);
    fn scissor_iter(&mut self, p0: Point, p1: Point);
    fn draw(&self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>);
    fn translate(&mut self, x: i32, y: i32);
    fn rotate(&mut self, a: f32);
    fn scale(&mut self, a: f32, b: f32);
}



fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("rust-sdl2 demo", 1200, 1200)
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

        let mut points: Vec<Point> = Vec::new();
        
        /*
        for y in 100..300 {
            points.extend(line((200, 200), (300, y)));
            points.extend(line((200, 200), (100, y)));
        }
        for x in 100..300 {
            points.extend(line((200, 200), (x, 300)));
            points.extend(line((200, 200), (x, 100)));
        }*/
        /*
        for y in 100..500 {
            points.extend(line((100, y), (500, y)));
        }

        //println!("{}", points.len());
        //points.extend(rect(200, 200, 300, 300));
        //points.extend(ellipse(200, 200, 200, 200));

        //points.extend(line((200, 200), (300, 300)));

        points.scissor_iter((200, 200), (400, 400));
        */
        for y in 100..500 {
            points.extend(line((100, y), (500, y)));
        }

        //points.extend(polygon(&vec!((100 ,100), (200, 200), (300, 500), (200, 600))));

        //points.scissor((0, 0), (800, 800));

        points.scale(2.0, 2.0);
        points.draw(&mut canvas);

        canvas.present();
        let error = ::sdl2::get_error();
        if error != "" {
            println!("{}", error);
        }
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

impl VecExt for Vec<Point> { //(i32, i32)
    fn scissor(&mut self, p0: Point, p1: Point) {
        //sort list by x coordinate
        self.sort_by_key(|x| x.0);
        //delete every item with x < p0.0 or x > p1.0
        //binary search for x = p0.0 (x needs to = the first occurance of p0.0 in order to include all correct values)
        let x0 = match self.binary_search_by_key(&(p0.0-1), |x| x.0) {
                    Ok(e) => e, 
                    Err(e) => e
                };
        //binary search for x = p1.0
        let x1 = match self.binary_search_by_key(&(p1.0+1), |x| x.0) {
                    Ok(e) => e, 
                    Err(e) => e
                };
        //make a slice of x where x = p0.0..x where x = p1.0
        self.truncate(x1);
        *self = self.split_off(x0);
        //sort list by y coordinate
        self.sort_by_key(|x| x.1);
        //delete every item with y < p01 or y > p1.1
        let y0 = match self.binary_search_by_key(&(p0.1-1), |x| x.1) {
                    Ok(e) => e, 
                    Err(e) => e
                };
        let y1 = match self.binary_search_by_key(&(p1.1+1), |x| x.1)  {
                    Ok(e) => e, 
                    Err(e) => e
                };
        self.truncate(y1);
        *self = self.split_off(y0);
        self.shrink_to_fit();
    }
    fn scissor_iter(&mut self, p0: Point, p1: Point) { 
        //sort list by x coordinate
        self.sort_by_key(|x| x.0);
        //delete every item with x < p0.0 or x > p1.0
        let x: Vec<_> = self.drain(..)
                            .skip_while(|x| x.0 < p0.0)
                            .take_while(|x| x.0 < p1.0)
                            .collect();
        self.extend(x);
        //sort list by y coordinate
        self.sort_by_key(|x| x.1);
        //delete every item with y < p0.1 or y > p1.1
        let x: Vec<_> = self.drain(..)
                            .skip_while(|x| x.1 < p0.1)
                            .take_while(|x| x.1 < p1.1)
                            .collect();
        self.extend(x);
        self.shrink_to_fit();
    }
    fn draw(&self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
        for p in self{
            match canvas.draw_point(*p) {
                Ok(_) => {},
                Err(e) => println!("Error: {}", e)
            }
        }
    }
    fn translate(&mut self, x: i32, y: i32) {
        for p in self {
            p.0 += x;
            p.1 += y;
        }
    }
    fn rotate(&mut self, a: f32) {
        
    }
    fn scale(&mut self, a: f32, b: f32) {
        assert!(a >= 0.0 && b >= 0.0);
        for p in self {
            p.0 = (p.0 as f32 * a) as i32;
            p.1 = (p.1 as f32 * b) as i32;
        }
    }
}


fn line(p0: Point, p1: Point) -> Vec<Point> { //Starting coordinate, finishing coordinate
    let dx = p1.0 - p0.0;
    let dy = p1.1 - p0.1;
    if dx < 0 {
        return line(p1, p0);
    }
    let m = {
            if dx == 0 {
                0.0
            }
            else {
                dy as f32/dx as f32
            }
    };
    if m > 1.0 || m < -1.0 {
        return line2((p1.1, p1.0), (p0.1, p0.0));
    }
    let mut points: Vec<Point> = vec!((p0.0, p0.1), (p1.0, p1.1));
    let mut y = p0.1 as f32;
    for x in p0.0..p1.0 {
        y += m;
        points.push((x, (y + 0.5).floor() as i32));
    }
    points
}

fn line2(p0: Point, p1: Point) -> Vec<Point> {
    let dx = p1.0 - p0.0;
    let dy = p1.1 - p0.1;
    if dx < 0 {
        return line2(p1, p0);
    }
    let m = {
            if dx == 0 {
                0.0
            }
            else {
                dy as f32/dx as f32
            }
    };
    let mut points: Vec<Point> = vec!((p1.0, p1.1), (p0.0, p0.1));
    let mut y = p0.1 as f32;
    for x in p0.0..p1.0 {
        y += m;
        points.push(((y + 0.5).floor() as i32, x));
    }
    points
}

fn rect(p0: Point, p1: Point) -> Vec<Point> {
    let mut points = line((p0.0, p0.1), (p1.0, p0.1));
    points.extend(line((p0.0, p0.1), (p0.0, p1.1)));
    points.extend(line((p1.0, p0.1), (p1.0, p1.1)));
    points.extend(line((p0.0, p1.1), (p1.0, p1.1)));
    points
}

fn ellipse_points(x: i32, y: i32, p0: Point) -> Vec<Point> {
    let mut points: Vec<Point> = Vec::new();
    points.push((x + p0.0, y + p0.1));
    points.push((-x + p0.0, y + p0.1));
    points.push((x + p0.0, -y + p0.1));
    points.push((-x + p0.0, -y + p0.1));
    points
}
fn ellipse_points2(x: i32, y: i32, p0: Point) -> Vec<Point> {
    let mut points: Vec<Point> = Vec::new();
    points.push((y + p0.0, x + p0.1));
    points.push((-y + p0.0, x + p0.1));
    points.push((y + p0.0, -x + p0.1));
    points.push((-y + p0.0, -x + p0.1));
    points
}

fn polygon(corners: &Vec<Point>) -> Vec<Point> {
    let mut points: Vec<Point> = Vec::new();
    for x in corners.windows(2) {
        points.extend(line(x[0], x[1]));
    }
    points.extend(line(corners[0], corners[corners.len() - 1]));
    points
}

fn ellipse(p0: Point, a: i32, b: i32) -> Vec<Point> { //Center coordinate, width, height
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



#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;
    #[bench]
    fn thing(b: &mut Bencher) {
        let mut points: Vec<Point> = Vec::new();
        for y in 0..1000 {
            points.extend(line((0, y), (1000, y)));
        }
        b.iter(|| points.clone().scissor((200, 200), (400, 400)));
    }
    #[bench]
    fn thing_iter(b: &mut Bencher) {
        let mut points: Vec<Point> = Vec::new();
        for y in 0..1000 {
            points.extend(line((0, y), (1000, y)));
        }
        b.iter(|| points.clone().scissor_iter((200, 200), (400, 400)));
    }
}