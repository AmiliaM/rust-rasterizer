#![feature(test)]
extern crate test;
extern crate sdl2;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use std::fs::File;
use std::io::prelude::*;

type Point = (i32, i32);
type Line = (Point, Point);

pub trait VecExt {
    fn scissor(&mut self, p0: Point, p1: Point);
    fn scissor_iter(&mut self, p0: Point, p1: Point);
    fn draw(&self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>);
    fn translate(&mut self, x: i32, y: i32);
    fn rotate(&mut self, a: f32);
    fn scale(&mut self, a: f32, b: f32);
    fn add(self, other:Vec<Point>) -> Vec<Point>;
}

#[derive(Serialize, Deserialize)]
struct Scene {
    objects: Vec<Object>
}

impl Scene {
    fn new() -> Scene {
        Scene {
            objects: Vec::new(),
        }
    }
    fn draw(&self) -> Vec<Point> {
        let mut vec = Vec::new();
        for object in self.objects.iter() {
            vec.extend(object.draw());
        }
        vec
    }
    fn default_string(&mut self) -> Option<&mut Object> {
        self.objects.iter_mut().filter(|x| if let &Object { shape: Shape::Letters(_), .. } = x { true } else { false }).next()
    }
}

#[derive(Serialize, Deserialize)]
struct Object {
    shape: Shape,
    position: Point,
    scale: i32,
}

impl Object {
    fn new(shape: Shape, position: Point) -> Object {
        Object {
            shape,
            position,
            scale: 1,
        }
    }
    fn draw(&self) -> Vec<Point> {
        let mut points = self.shape.draw(self.scale);
        points.translate(self.position.0, self.position.1);
        points
    }
}

#[derive(Serialize, Deserialize)]
enum Shape {
    Circle { center: Point, width: i32, height: i32 },
    Polygon(Vec<Point>),
    Letters(String),
    Lines(Vec<Line>),
}

impl Shape {
    fn draw(&self, scale: i32) -> Vec<Point> {
        match self {
            Shape::Circle { center, width, height } => ellipse(*center, *width, *height),
            Shape::Polygon(points) => polygon(points),
            Shape::Letters(s) => {
                let mut vec = Vec::new();
                for (i, ch) in s.chars().enumerate() {
                    let mut points = Shape::for_letter(ch).draw(scale);
                    points.translate(100 * i as i32, 0);
                    vec.extend(points.into_iter());
                }
                vec
            }
            Shape::Lines(lines) => {
                let mut vec = Vec::new();
                for l in lines {
                    vec.extend(line(l.0, l.1));
                }
                vec
            }
        }
    }
    fn for_letter(c: char) -> Shape {
        let p0 = (0, 0);
        let l0 = (p0, p0);
        let s = 100;
        let p = 0;
        let c = c.to_uppercase().next().unwrap();
        let vec = match c {
            'A' => vec!(
                ((p, p+s), (p+(s/2), p)),
                ((p+(s/2), p), (p+s, p+s)),
                ((p+(s/4), p+(s/2)), (p+(3*s/4), p+(s/2)))
                ),
            'B' => vec!(((p, p), (p+1, p+s)),
                ((p, p), (p+(s/2), p+(s/4))),
                ((p+(s/2), p+(s/4)), (p, p+(s/2))),
                ((p, p+(s/2)), (p+(s/2), p+(3*s/4))),
                ((p+(s/2), p+(3*s/4)), (p, p+s))),
            'C' => vec!(((p+(s/2), p), (p, p+(s/2))),
                ((p, p+(s/2)), (p+(s/2), p+s))),
            'D' => vec!(((p, p), (p+(s/2), p+(s/2))),
                ((p+(s/2), p+(s/2)), (p, p+s)),
                ((p, p), (p+1, p+s))),
            'E' => vec!(((p, p), (p+1, p+s)),
                ((p, p), (p+s, p)),
                ((p, p+(s/2)), (p+s, p+(s/2))),
                ((p, p+s), (p+s, p+s))),
            'F' => vec![((1, 2),(2, 4)), ((1, 2),(2, 4))],
            'G' => vec!(l0),
            'H' => vec!(((p, p), (p+1, p+s)),
                ((p+s, p), (p+s-1, p+s)),
                ((p, p+(s/2)), (p+s, p+(s/2)))),
            'I' => vec!(((1, 2),(1, 2),)),
            'J' => vec!(l0),
            'K' => vec!(l0),
            'L' => vec!(l0),
            'M' => vec!(l0),
            'N' => vec!(l0),
            'O' => vec!(l0),
            'P' => vec!(l0),
            'Q' => vec!(l0),
            'R' => vec!(l0),
            'S' => vec!(l0),
            'T' => vec!(l0),
            'U' => vec!(l0),
            'V' => vec!(l0),
            'W' => vec!(l0),
            'X' => vec!(l0),
            'Y' => vec!(l0),
            'Z' => vec!(l0),
            _ => panic!("Attempted to generate unsupported letter")
        };
        Shape::Lines(vec)
    }
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

    let mut scene = Scene::new();
    let letter = Object::new(Shape::for_letter('A'), (100, 100));
    let poly = Object::new(Shape::Polygon(vec![(100, 100), (200, 200), (100, 200)]), (50, 50));
    let st = Object::new(Shape::Letters("abcd".to_string()), (50, 50));

    scene.objects.extend(vec!(poly,letter,st));

    video_subsystem.text_input().start();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::Backspace), .. } => {
                    if let Some(&mut Object { shape: Shape::Letters(ref mut s), .. } ) = scene.default_string() {
                        let l = { s.len() };
                        if l <= 0 {
                            continue;
                        }
                        s.remove(l-1);
                    }
                },
                Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
                    let serial = serde_json::to_string(&scene).unwrap();
                    let mut file = File::create("saved_drawing.json").unwrap();
                    file.write_all(&serial.as_bytes()).unwrap();
                }
                Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
                    let mut file = File::open("saved_drawing.json").unwrap();
                    let mut contents = String::new();
                    file.read_to_string(&mut contents).unwrap();
                    scene = serde_json::from_str(&contents).unwrap();
                    
                }
                Event::TextInput { text: text, .. } => {
                    println!("{}", text);
                    if let Some(&mut Object { shape: Shape::Letters(ref mut s), .. } ) = scene.default_string() {
                        *s += &text;
                    }
                },
                _ => {}
            }
        }
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.set_draw_color(Color::RGB(0, 255, 255));

        let mut points: Vec<Point> = Vec::new();
        
        points.extend(scene.draw().into_iter());
        points.draw(&mut canvas);
        
        canvas.present();
        /*let error = ::sdl2::get_error();
        if error != "" {
            println!("{}", error);
        }*/
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

impl VecExt for Vec<Point> { 
    fn scissor(&mut self, p0: Point, p1: Point) {
        self.sort_by_key(|x| x.0);
        let x0 = match self.binary_search_by_key(&(p0.0-1), |x| x.0) {
                    Ok(e) => e, 
                    Err(e) => e
                };
        let x1 = match self.binary_search_by_key(&(p1.0+1), |x| x.0) {
                    Ok(e) => e, 
                    Err(e) => e
                };
        self.truncate(x1);
        *self = self.split_off(x0);
        self.sort_by_key(|x| x.1);
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
        self.sort_by_key(|x| x.0);
        let x: Vec<_> = self.drain(..)
                            .skip_while(|x| x.0 < p0.0)
                            .take_while(|x| x.0 < p1.0)
                            .collect();
        self.extend(x);
        self.sort_by_key(|x| x.1);
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
    fn add(self, other:Vec<Point>) -> Vec<Point> {
        let mut mlem = self;
        mlem.extend(other);
        return mlem
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
    let mut points: Vec<Point> = vec!((p1.1, p1.0), (p0.1, p0.0));
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