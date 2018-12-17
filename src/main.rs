#![cfg_attr(feature = "test", feature(test))]
#[cfg(feature = "test")]
extern crate test;

extern crate sdl2;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::{RenderTarget,Canvas};
use serde::ser::{Serialize, Serializer, SerializeSeq};
use serde::de::{Deserialize, Deserializer, Visitor, SeqAccess};
use std::time::Duration;
use std::fs::File;
use std::io::prelude::*;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use std::ops::{Deref, DerefMut};

type Point = (i32, i32);
type PColor = (u8, u8, u8);
type Line = (Point, Point);

pub trait VecExt {
    fn scissor(&mut self, p0: Point, p1: Point);
    fn scissor_iter(&mut self, p0: Point, p1: Point);
    fn translate(&mut self, x: i32, y: i32);
    fn rotate(&mut self, a: f32);
    fn scale(&mut self, a: f32, b: f32);
    fn add(self, other:Vec<Point>) -> Vec<Point>;
}

#[derive(Serialize, Deserialize)]
struct Scene {
    objects: ObjectList,
    selected_object: usize,
    groups: ObjectList,
    camera: Point,
    scale: (f32, f32),
    rotation: f32,
}

struct ObjectList(Vec<Rc<RefCell<Object>>>);

impl ObjectList {
    fn new() -> ObjectList {
        ObjectList(Vec::new())
    }
}

impl Deref for ObjectList {
    type Target = Vec<Rc<RefCell<Object>>>;
    
    fn deref(&self) -> &Vec<Rc<RefCell<Object>>> {
        &self.0
    }
}

impl DerefMut for ObjectList {
    fn deref_mut(&mut self) -> &mut Vec<Rc<RefCell<Object>>> {
        &mut self.0
    }
}

impl Serialize for ObjectList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for e in self.0.iter() {
            seq.serialize_element(&*e.borrow())?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for ObjectList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        struct ObjectListVisitor;
        impl<'de> Visitor<'de> for ObjectListVisitor {
            type Value = ObjectList;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("objects list")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<ObjectList, V::Error> where V: SeqAccess<'de> {
                let mut objects = Vec::new();
                while let Some(x) = seq.next_element()? {
                    objects.push(x);
                }
                let objects = objects.into_iter().map(|x| Rc::new(RefCell::new(x))).collect();
                Ok(ObjectList(objects))
            }
        }

        deserializer.deserialize_seq(ObjectListVisitor)
    }
}

impl Scene {
    fn new() -> Scene {
        let mut s = Scene {
            objects: ObjectList::new(),
            selected_object: 0,
            groups: ObjectList::new(),
            camera: (0, 0),
            rotation: 0.,
            scale: (1., 1.),
        };
        for _ in 0..10 {
            let o = Object::new(Shape::Group(ObjectList::new()), (0, 0));
            s.groups.push(o.clone());
            s.objects.push(o);
        }
        s
    }
    fn draw<T>(&self, canvas: &mut Canvas<T>) where T: RenderTarget {
        for (i, object) in self.objects.iter().enumerate() {
            let object = object.borrow();
            canvas.set_draw_color(Color::RGB(object.color.0, object.color.1, object.color.2));
            if self.selected_object == i {
                canvas.set_draw_color(Color::RGB(255, 255, 0));
            }
            let mut points = object.draw();
            points.translate(-self.camera.0, -self.camera.1);
            points.scale(self.scale.0, self.scale.1);
            points.rotate(self.rotation);
            for point in points.into_iter() {
                canvas.draw_point(point).unwrap();
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Object {
    shape: Shape,
    position: Point,
    scale: (f32, f32),
    rotation: f32,
    color: PColor,
}

impl Object {
    fn new(shape: Shape, position: Point) -> Rc<RefCell<Object>> {
        Rc::new(RefCell::new(Object {
            shape,
            position,
            scale: (1., 1.),
            color: (0, 255, 255),
            rotation: 0.,
        }))
    }
    fn with_color(shape: Shape, position: Point, color: PColor) -> Rc<RefCell<Object>> {
        Rc::new(RefCell::new(Object {
            shape,
            position,
            scale: (1., 1.),
            rotation: 0.,
            color,
        }))
    }
    fn draw(&self) -> Vec<Point> {
        let mut points = self.shape.draw(self.scale);
        points.scale(self.scale.0, self.scale.1);
        points.rotate(self.rotation);
        points.translate(self.position.0, self.position.1);
        points
    }
}

#[derive(Serialize, Deserialize)]
enum Shape {
    Circle { width: i32, height: i32 },
    Rect(Point, Point),
    Polygon(Vec<Point>),
    Letters(String),
    Lines(Vec<Line>),
    Group(ObjectList),
}

impl Shape {
    fn draw(&self, scale: (f32, f32)) -> Vec<Point> {
        match self {
            Shape::Circle { width, height } => ellipse( (0, 0), *width, *height),
            Shape::Rect(p0, p1) => rect(*p0, *p1),
            Shape::Polygon(points) => polygon(points),
            Shape::Letters(s) => {
                let mut vec = Vec::new();
                for (i, ch) in s.chars().enumerate() {
                    let mut points = Shape::for_letter(ch).draw(scale);
                    points.translate((100 * i as i32)%1100, 100 * ((100 * i as i32)/1100));
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
            },
            Shape::Group(objs) => {
                let mut vec = Vec::new();
                for o in objs.0.iter() {
                    vec.extend(o.borrow().draw().into_iter());
                }
                vec
            }
        }
    }
    fn for_letter(c: char) -> Shape {
        let s = 50;
        let c = c.to_uppercase().next().unwrap();
        let vec = match c {
            'A' => vec![
                ((0, s), (s/2, 0)),
                ((s/2, 0), (s, s)),
                ((s/4, s/2), (3*s/4, s/2))],
            'B' => vec![
                ((0, 0), (1, s)),
                ((0, 0), (s/2, s/4)),
                ((s/2, s/4), (0, s/2)),
                ((0, s/2), (s/2, 3*s/4)),
                ((s/2, 3*s/4), (0, s))],
            'C' => vec![
                ((s/2, 0), (0, s/2)),
                ((0, s/2), (s/2, s))],
            'D' => vec![
                ((0, 0), (s/2, s/2)),
                ((s/2, s/2), (0, s)),
                ((0, 0), (1, s))],
            'E' => vec![
                ((0, 0), (1, s)),
                ((0, 0), (s, 0)),
                ((0, s/2), (s, s/2)),
                ((0, s), (s, s))],
            'F' => vec![
                ((0, 0), (1, s)),
                ((0, 0), (s, 0)),
                ((0, s/2), (s, s/2))],
            'G' => vec![
                ((s/2, 0), (0, s/2)),
                ((0, s/2), (s/2, s)),
                ((s/2, s), (s, s/2)),
                ((s-1, s/2), (s/2, s/2))],
            'H' => vec![
                ((0, 0), (1, s)),
                ((s, 0), (s-1, s)),
                ((0, s/2), (s, s/2))],
            'I' => vec![((0, 0), (1, s))],
            'J' => vec![
                ((s, 0), (s-1, s)),
                ((s, s), (0, s)),
                ((0, s), (1, s/2))],
            'K' => vec![
                ((0,0), (1, s)),
                ((0, s/2), (s, 0)),
                ((0, s/2), (s, s))],
            'L' => vec![
                ((0, 0), (1, s)),
                ((1, s), (s, s))],
            'M' => vec![
                ((0, 0), (1, s)),
                ((0, 0), (s/2, s/2)),
                ((s/2, s/2), (s, 0)),
                ((s, 0), (s-1, s))],
            'N' => vec![
                ((0, 0), (1, s)),
                ((0, 0), (s, s)),
                ((s, s), (s-1, 0))],
            'O' => vec![
                ((s/2, 0), (0, s/2)),
                ((s/2, 0), (s, s/2)),
                ((s, s/2), (s/2, s)),
                ((0, s/2), (s/2, s))],
            'P' => vec![
                ((0, 0), (1, s)),
                ((0, 0), (s/2, s/4)),
                ((s/2, s/4), (0, s/2))],
            'Q' => vec![
                ((s/2, 0), (0, s/2)),
                ((s/2, 0), (s, s/2)),
                ((s, s/2), (s/2, s)),
                ((0, s/2), (s/2, s)),
                ((s/2, s/2), (s, s))],
            'R' => vec![
                ((0, 0), (1, s)),
                ((0, 0), (s/2, s/4)),
                ((s/2, s/4), (0, s/2)),
                ((0, s/2), (s/2, s))],
            'S' => vec![
                ((s, 0), (0, s/3)),
                ((0, s/3), (s, 2*s/3)),
                ((s, 2*s/3), (0, s))],
            'T' => vec![
                ((s/2, 0), ((s/2)+1, s)),
                ((0, 0), (s, 0))],
            'U' => vec![
                ((0, 0), (1, s)),
                ((1, s), (s-1, s)),
                ((s-1, s), (s, 0))],
            'V' => vec![
                ((0, 0), (s/2, s)),
                ((s/2, s), (s, 0))],
            'W' => vec![
                ((0, 0), (s/3, s)),
                ((s/3, s), (s/2, 0)),
                ((s/2, 0), (2*s/3, s)),
                ((2*s/3, s), (s, 0))],
            'X' => vec![
                ((0, 0), (s, s)),
                ((s, 0), (0, s))],
            'Y' => vec![
                ((0, 0), (s/2, s/2)),
                ((s, 0), (s/2, s/2)),
                ((s/2, s/2), ((s/2)-1, s))],
            'Z' => vec![
                ((0, 0), (s, 0)),
                ((s, 0), (0, s)),
                ((0, s), (s, s))],
            '0' => vec![
                ((s/2, 0), (0, s/2)),
                ((s/2, 0), (s, s/2)),
                ((s, s/2), (s/2, s)),
                ((0, s/2), (s/2, s))],
            '1' => vec![((0, 0), (1, s))],
            '2' => vec![
                ((0, 0), (s, 0)),
                ((s, 0), (0, s)),
                ((0, s), (s, s))],
            '3' => vec![
                ((s, 0), (s-1, s)),
                ((0, 0), (s, 0)),
                ((0, s/2), (s, s/2)),
                ((0, s), (s, s))],
            '4' => vec![
                ((s, 0), (0, s/2)),
                ((0, s/2), (s, s/2)),
                ((s, 0), (s-1, s))],
            '5' => vec![
                ((s, 0), (0, s/3)),
                ((0, s/3), (s, 2*s/3)),
                ((s, 2*s/3), (0, s))],
            '6' => vec![
                ((0, 0), (1, s)),
                ((0, s), (s/2, 3*s/4)),
                ((s/2, 3*s/4), (0, s/2))],
            '7' => vec![
                ((0, 0), (s, 0)),
                ((s, 0), (s/2, s))],
            '8' => vec![
                ((1, 0), (0, s)),
                ((0, 0), (s, 0)),
                ((0, s/2), (s, s/2)),
                ((s-1, 0), (s, s)),
                ((0, s), (s, s))],
            '9' => vec![
                ((s, 0), (s-1, s)),
                ((s, 0), (s/2, s/4)),
                ((s/2, s/4), (s, s/2))],
            ' ' => vec!(),
            _ => {vec![]}
        };
        Shape::Lines(vec)
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("rusterizer", 1200, 1200)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas: sdl2::render::Canvas<sdl2::video::Window> = window.into_canvas().accelerated().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut scene = Scene::new();
    //let letter = Object::new(Shape::for_letter('A'), (100, 100));
    //let poly = Object::new(Shape::Polygon(vec![(100, 100), (200, 200), (100, 200)]), (50, 50));
    let mut command_st = Object::new(Shape::Letters("".to_string()), (50, 50));
    let blel = Object::new(Shape::Circle { height: 50, width: 100 }, (300, 300) );

    scene.objects.extend(vec!(command_st.clone(), blel));

    video_subsystem.text_input().start();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::Backspace), .. } => {
                    let mut st = command_st.borrow_mut();
                    if let Shape::Letters(ref mut s) = st.shape {
                        let l = { s.len() };
                        if l == 0 {
                            continue;
                        }
                        s.remove(l-1);
                    }
                },
                Event::KeyDown { keycode: Some(Keycode::Return), .. } => {
                    let mut st = command_st.borrow_mut();
                    if let Shape::Letters(ref mut s) = st.shape {
                        let mut invalid = false;
                        {
                            let parts: Vec<_> = s.split(" ").collect();
                            if parts.len() < 1 {
                                continue;
                            }
                            match parts[0] {
                                "ellipse" if parts.len() == 3 => {
                                    let x = parts[1].parse().unwrap();
                                    let y = parts[2].parse().unwrap();
                                    scene.objects.extend(vec!(Object::with_color(Shape::Circle { height: x, width: y }, (100, 100), (255, 0, 0))));
                                },
                                "rect" if parts.len() == 5 => {
                                    let x0 = parts[1].parse().unwrap();
                                    let y0 = parts[2].parse().unwrap();
                                    let x1 = parts[3].parse().unwrap();
                                    let y1 = parts[4].parse().unwrap();
                                    scene.objects.extend(vec!(Object::with_color(Shape::Rect((x0, y0), (x1, y1)), (100, 100), (255, 0, 0))));
                                },
                                _ => {
                                    invalid = true;
                                }
                            }
                        }
                        if invalid {
                            { s.clear(); }
                            *s += "invalid";
                        }
                    }
                },
                Event::KeyDown { keycode: Some(Keycode::Down), keymod, .. } if keymod.contains(sdl2::keyboard::LCTRLMOD) => {
                    scene.camera.1 += 5;
                }
                Event::KeyDown { keycode: Some(Keycode::Up), keymod, .. } if keymod.contains(sdl2::keyboard::LCTRLMOD) => {
                    scene.camera.1 -= 5;
                }
                Event::KeyDown { keycode: Some(Keycode::Left), keymod, .. } if keymod.contains(sdl2::keyboard::LCTRLMOD) => {
                    scene.camera.0 -= 5;
                }
                Event::KeyDown { keycode: Some(Keycode::Right), keymod, .. } if keymod.contains(sdl2::keyboard::LCTRLMOD) => {
                    scene.camera.0 += 5;
                }


                Event::KeyDown { keycode: Some(Keycode::Down), keymod, .. } if keymod.contains(sdl2::keyboard::LSHIFTMOD) => {
                    scene.objects[scene.selected_object].borrow_mut().position.1 += 5;
                }
                Event::KeyDown { keycode: Some(Keycode::Up), keymod, .. } if keymod.contains(sdl2::keyboard::LSHIFTMOD) => {
                    scene.objects[scene.selected_object].borrow_mut().position.1 -= 5;
                }
                Event::KeyDown { keycode: Some(Keycode::Left), keymod, .. } if keymod.contains(sdl2::keyboard::LSHIFTMOD) => {
                    scene.objects[scene.selected_object].borrow_mut().position.0 -= 5;
                }
                Event::KeyDown { keycode: Some(Keycode::Right), keymod, .. } if keymod.contains(sdl2::keyboard::LSHIFTMOD) => {
                    scene.objects[scene.selected_object].borrow_mut().position.0 += 5;
                }


                Event::KeyDown { keycode: Some(Keycode::Equals), keymod, .. } if keymod.contains(sdl2::keyboard::LCTRLMOD) => {
                    scene.scale.0 += 0.1;
                    scene.scale.1 += 0.1;
                }
                Event::KeyDown { keycode: Some(Keycode::Minus), keymod, .. } if keymod.contains(sdl2::keyboard::LCTRLMOD) => {
                    scene.scale.0 -= 0.1;
                    scene.scale.1 -= 0.1;
                }

                Event::KeyDown { keycode: Some(Keycode::Equals), .. } => {
                    scene.objects[scene.selected_object].borrow_mut().scale.0 += 0.1;
                    scene.objects[scene.selected_object].borrow_mut().scale.1 += 0.1;
                }
                Event::KeyDown { keycode: Some(Keycode::Minus), .. } => {
                    scene.objects[scene.selected_object].borrow_mut().scale.0 -= 0.1;
                    scene.objects[scene.selected_object].borrow_mut().scale.1 -= 0.1;
                }

                Event::KeyDown { keycode: Some(Keycode::LeftBracket), keymod, .. } if keymod.contains(sdl2::keyboard::LCTRLMOD) => {
                    scene.rotation -= 3.;
                }
                Event::KeyDown { keycode: Some(Keycode::RightBracket), keymod, .. } if keymod.contains(sdl2::keyboard::LCTRLMOD) => {
                    scene.rotation += 3.;
                }

                Event::KeyDown { keycode: Some(Keycode::LeftBracket), .. } => {
                    scene.objects[scene.selected_object].borrow_mut().rotation -= 3.;
                }
                Event::KeyDown { keycode: Some(Keycode::RightBracket), .. } => {
                    scene.objects[scene.selected_object].borrow_mut().rotation += 3.;
                }


                Event::KeyDown { keycode: Some(Keycode::X), keymod, .. } if keymod.contains(sdl2::keyboard::LCTRLMOD) => {
                    let mut locs = Vec::new();
                    if let Shape::Group(ref mut objs) = scene.objects[scene.selected_object].borrow_mut().shape {
                        locs.extend(objs.drain(..));
                    }
                    scene.objects.extend(locs.into_iter());
                }

                Event::KeyDown { keycode: Some(Keycode::Num0), keymod, .. } |
                Event::KeyDown { keycode: Some(Keycode::Num1), keymod, .. } |
                Event::KeyDown { keycode: Some(Keycode::Num2), keymod, .. } |
                Event::KeyDown { keycode: Some(Keycode::Num3), keymod, .. } |
                Event::KeyDown { keycode: Some(Keycode::Num4), keymod, .. } |
                Event::KeyDown { keycode: Some(Keycode::Num5), keymod, .. } |
                Event::KeyDown { keycode: Some(Keycode::Num6), keymod, .. } |
                Event::KeyDown { keycode: Some(Keycode::Num7), keymod, .. } |
                Event::KeyDown { keycode: Some(Keycode::Num8), keymod, .. } |
                Event::KeyDown { keycode: Some(Keycode::Num9), keymod, .. } if keymod.contains(sdl2::keyboard::LSHIFTMOD) => {
                    if let Event::KeyDown { keycode: Some(x), .. } = event {
                        let num = match x {
                            Keycode::Num0 => 0,
                            Keycode::Num1 => 1,
                            Keycode::Num2 => 2,
                            Keycode::Num3 => 3,
                            Keycode::Num4 => 4,
                            Keycode::Num5 => 5,
                            Keycode::Num6 => 6,
                            Keycode::Num7 => 7,
                            Keycode::Num8 => 8,
                            Keycode::Num9 => 9,
                            _ => unreachable!(),
                        };
                        if let Shape::Group(ref mut objs) = scene.groups[num].borrow_mut().shape {
                            objs.push(scene.objects[scene.selected_object].clone());
                            scene.objects.remove(scene.selected_object);
                            scene.selected_object = 0;
                        }
                    }
                }
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
                    command_st = scene.objects.iter().cloned().filter(|x| if let Shape::Letters(_) = x.borrow().shape { true } else { false }).next().unwrap();
                }
                Event::KeyDown { keycode: Some(Keycode::Tab), .. } => {
                    scene.selected_object += 1;
                    if scene.selected_object >= scene.objects.len() {
                        scene.selected_object = 0;
                    }
                    while let Shape::Group(ref objs) = scene.objects[scene.selected_object].borrow().shape {
                        if objs.len() == 0 {
                            scene.selected_object += 1;
                            if scene.selected_object >= scene.objects.len() {
                                scene.selected_object = 0;
                            }
                        } else {
                            break;
                        }
                    }

                    println!("Selected object: {}", scene.selected_object);
                }
                Event::TextInput { text, .. } => {
                    let mut st = command_st.borrow_mut();
                    if let Shape::Letters(ref mut s) = st.shape {
                        *s += &text;
                    }
                },
                _ => {}
            }
        }
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        //st.position.0 +=1;
        canvas.set_draw_color(Color::RGB(0, 255, 255));

        scene.draw(&mut canvas);
        
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
    fn translate(&mut self, x: i32, y: i32) {
        for p in self {
            p.0 += x;
            p.1 += y;
        }
    }
    fn rotate(&mut self, a: f32) {
        let s = a.sin();
        let c = a.cos();

        for p in self {
            let x = p.0 as f32;
            let y = p.1 as f32;
            p.0 = (x * c + y * s) as i32;
            p.1 = (x * s + y * c) as i32;
        }
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

fn polygon(corners: &Vec<Point>) -> Vec<Point> {
    let mut points: Vec<Point> = Vec::new();
    for x in corners.windows(2) {
        points.extend(line(x[0], x[1]));
    }
    points.extend(line(corners[0], corners[corners.len() - 1]));
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
    let mut d2 = b.pow(2) * ((x).pow(2)) + a.pow(2) * ((y - 1).pow(2)) - (a.pow(2) * b.pow(2));
    while y > 0 {
        if d2 < 0 {
            d2 += b.pow(2) * (2 * x + 2) + a.pow(2) * (-2 * y + 3);
            x += 1;
        } else {
            d2 += a.pow(2) * (-2 * y + 3);
        }
        y -= 1;
        points.extend(ellipse_points(x, y, p0));
    }
    points
}

#[cfg(all(test, feature = "test"))]
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