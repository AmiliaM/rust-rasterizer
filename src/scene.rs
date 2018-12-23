use serde::ser::{Serialize, Serializer, SerializeSeq};
use serde::de::{Deserialize, Deserializer, Visitor, SeqAccess};

use std::cell::RefCell;
use std::rc::Rc;
use std::ops::{Deref, DerefMut};
use std::fmt;

use sdl2::render::{RenderTarget,Canvas};
use sdl2::pixels::Color;

use util::{VecExt,line,rect,ellipse,polygon,Point};

type PColor = (u8, u8, u8);
type Line = (Point, Point);

#[derive(Serialize, Deserialize)]
pub struct Scene {
    pub objects: ObjectList,
    pub selected_object: usize,
    pub groups: ObjectList,
    pub camera: Point,
    pub scale: (f32, f32),
    pub rotation: f32,
}

pub struct ObjectList(Vec<Rc<RefCell<Object>>>);

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
    pub fn new() -> Scene {
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
    pub fn draw<T>(&self, canvas: &mut Canvas<T>) where T: RenderTarget {
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
pub struct Object {
    pub shape: Shape,
    pub position: Point,
    pub scale: (f32, f32),
    pub rotation: f32,
    color: PColor,
}

impl Object {
    pub fn new(shape: Shape, position: Point) -> Rc<RefCell<Object>> {
        Rc::new(RefCell::new(Object {
            shape,
            position,
            scale: (1., 1.),
            color: (0, 255, 255),
            rotation: 0.,
        }))
    }
    pub fn with_color(shape: Shape, position: Point, color: PColor) -> Rc<RefCell<Object>> {
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
pub enum Shape {
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
