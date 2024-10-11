
#![procedural::magic_macro]
//!Create a module called Point that inside has a struct Point with the fields x: f32 , y: f32 .
//!Create the following methods
//! - new that initializes the Point
//! - distance that borrow a Point and returns the distance between the two points
//!   
//! 
//! Create then another module called line that has a struct `Line` with the
//! fields `start: Point`, `end: Point`, `m: f32` and `q: f32`
//! - you have to implement the new method that takes two points and calculates the slope
//!   and the intercept of the line m and q
//! - contains that borrow a p: Point and returns a Result<_, String> . The function
//!   should check if the Line contains the borrowed point
//! 
#![dependency("oorandom=\"11.1\"")]
mod point{
    pub struct Point {
        pub x: f32,
        pub y: f32,
    }
    
    impl Point {
        pub fn new(x: f32, y: f32) -> Self {
            Point { x, y }
        }
    
        pub fn distance(&self, other: &Point) -> f32 {
            let x = (self.x - other.x).powi(2);
            let y = (self.y - other.y).powi(2);
            (x + y).sqrt()
        }
    }    
}
mod line{
    use super::point::Point;

    pub struct Line {
        start: Point,
        end: Point,
        m: f32,
        q: f32,
    }

    impl Line {
        pub fn new(start: Point, end: Point) -> Self {
            let m = (end.y - start.y) / (end.x - start.x);
            let q = end.y - start.y - m * (end.x - start.x);
            Line { start, end, m, q }
        }
        pub fn contains(&self, point: &Point) -> Result<(), &str> {
            let res = self.m * point.x + self.q;
            if point.y == res {
                Ok(())
            } else {
                Err("Not contained")
            }
        }
    }

}

#[runtest(1.0)]
/// tests if the Point::new function works as expected
fn test_new_point(){
    let point = point::Point::new(1.0, 2.0);
}

#[runtest(1.0)]
/// tests if the structure Point is correctly defined
fn test_point(){
    let point = point::Point{
        x: 1.0,
        y: 1.0,
    };
}

#[runtest(1.0, point::Point::distance)]
#[overwrite(impl Point::new in point)]
/// tests if the distance metric is correctly defined.
fn test_distance_point(){
    let point = point::Point::new(1.0, 2.0);
    let point2 = point::Point::new(4.0, 6.0);
    assert_eq!(point.distance(&point2), 5.0);
}

#[runtest(1.0, point::Point::distance)]
#[overwrite(impl Point::new in point)]
/// tests if the distance metric is correctly defined.
fn test_new_point_on_random_points(){
    let mut rng = oorandom::Rand32::new(10);
    for i in 0..1000{
        let point = point::Point::new(rng.rand_float()*100.0, rng.rand_float()*100.0);
        let point2 = point::Point::new(rng.rand_float()*100.0, rng.rand_float()*100.0);
        let dist = ((point.x-point2.x)*(point.x-point2.x)+(point.y-point2.y)*(point.y-point2.y)).sqrt();
        let user_dist = point.distance(&point2);
        if (user_dist-dist).abs()> 0.00001{

            panic!("Error, expected distance {}, but got {}\n The input for point were ({},{}), ({},{}):", dist, user_dist, point.x, point.y, point2.x, point2.y);
        }
    }
}
