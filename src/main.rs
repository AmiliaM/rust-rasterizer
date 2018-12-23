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

use std::time::Duration;
use std::fs::File;
use std::io::prelude::*;

mod util;
mod scene;

use scene::{Scene,Object,Shape};

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