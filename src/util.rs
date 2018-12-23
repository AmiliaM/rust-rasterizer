pub type Point = (i32, i32);

pub trait VecExt {
    fn scissor(&mut self, p0: Point, p1: Point);
    fn scissor_iter(&mut self, p0: Point, p1: Point);
    fn translate(&mut self, x: i32, y: i32);
    fn rotate(&mut self, a: f32);
    fn scale(&mut self, a: f32, b: f32);
    fn add(self, other:Vec<Point>) -> Vec<Point>;
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


pub fn line(p0: Point, p1: Point) -> Vec<Point> { //Starting coordinate, finishing coordinate
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

pub fn rect(p0: Point, p1: Point) -> Vec<Point> {
    let mut points = line((p0.0, p0.1), (p1.0, p0.1));
    points.extend(line((p0.0, p0.1), (p0.0, p1.1)));
    points.extend(line((p1.0, p0.1), (p1.0, p1.1)));
    points.extend(line((p0.0, p1.1), (p1.0, p1.1)));
    points
}

pub fn polygon(corners: &Vec<Point>) -> Vec<Point> {
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

pub fn ellipse(p0: Point, a: i32, b: i32) -> Vec<Point> { //Center coordinate, width, height
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
