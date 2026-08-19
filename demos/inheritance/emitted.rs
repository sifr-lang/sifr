// src/main.rs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Shape {
    name: String,
    color: String,
}

impl Shape {
    fn new(name: String, color: String) -> Self {
        let __sifr_field_init_0: String = name;
        let __sifr_field_init_1: String = color;
        Self { name: __sifr_field_init_0, color: __sifr_field_init_1 }
    }
}

impl Shape {
    fn describe(&self) -> String {
        {
    let mut __sifr_concat: String = String::with_capacity(((0usize + 2usize) + 0usize) + 1usize);
    __sifr_concat.push_str((self.name.clone()).as_str());
    __sifr_concat.push_str(" (");
    __sifr_concat.push_str((self.color.clone()).as_str());
    __sifr_concat.push(')');
    __sifr_concat
}
    }
}

impl ::std::fmt::Display for Shape {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Shape(name={}, color={})", self.name, self.color)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Circle {
    shape: Shape,
    radius: f64,
}

impl ::std::ops::Deref for Circle {
    type Target = Shape;
    fn deref(&self) -> &Self::Target {
        &self.shape
    }
}

impl ::std::ops::DerefMut for Circle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.shape
    }
}

impl ::std::convert::From<Circle> for Shape {
    fn from(value: Circle) -> Self {
        value.shape
    }
}

impl Circle {
    fn new(color: String, radius: f64) -> Self {
        let __sifr_parent = Shape::new("Circle".to_string(), color);
        let __sifr_field_init_0: f64 = radius;
        Self { shape: __sifr_parent, radius: __sifr_field_init_0 }
    }
}

impl Circle {
    fn area(&self) -> f64 {
        ((3.14159_f64) * self.radius) * self.radius
    }
}

impl Circle {
    fn describe(&self) -> String {
        {
    let mut __sifr_concat: String = String::with_capacity((0usize + 3usize) + 0usize);
    __sifr_concat.push_str((self.shape.name.clone()).as_str());
    __sifr_concat.push_str(" r=");
    __sifr_concat.push_str((format!("{}", self.radius)).as_str());
    __sifr_concat
}
    }
}

impl ::std::fmt::Display for Circle {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Circle(shape={}, radius={})", self.shape, self.radius)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Rectangle {
    shape: Shape,
    width: f64,
    height: f64,
}

impl ::std::ops::Deref for Rectangle {
    type Target = Shape;
    fn deref(&self) -> &Self::Target {
        &self.shape
    }
}

impl ::std::ops::DerefMut for Rectangle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.shape
    }
}

impl ::std::convert::From<Rectangle> for Shape {
    fn from(value: Rectangle) -> Self {
        value.shape
    }
}

impl Rectangle {
    fn new(color: String, width: f64, height: f64) -> Self {
        let __sifr_parent = Shape::new("Rectangle".to_string(), color);
        let __sifr_field_init_0: f64 = width;
        let __sifr_field_init_1: f64 = height;
        Self { shape: __sifr_parent, width: __sifr_field_init_0, height: __sifr_field_init_1 }
    }
}

impl Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }
}

impl Rectangle {
    fn describe(&self) -> String {
        {
    let mut __sifr_concat: String = String::with_capacity((((0usize + 1usize) + 0usize) + 1usize) + 0usize);
    __sifr_concat.push_str((self.shape.name.clone()).as_str());
    __sifr_concat.push(' ');
    __sifr_concat.push_str((format!("{}", self.width)).as_str());
    __sifr_concat.push('x');
    __sifr_concat.push_str((format!("{}", self.height)).as_str());
    __sifr_concat
}
    }
}

impl ::std::fmt::Display for Rectangle {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Rectangle(shape={}, width={}, height={})", self.shape, self.width, self.height)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Temperature {
    celsius: f64,
}

impl Temperature {
    fn new(celsius: f64) -> Self {
        let __sifr_field_init_0: f64 = celsius;
        Self { celsius: __sifr_field_init_0 }
    }
}

impl Temperature {
    fn from_fahrenheit(f: f64) -> Temperature {
        Temperature::new(((f - (32.0_f64)) * (5.0_f64)) / (9.0_f64))
    }
}

impl Temperature {
    fn freezing() -> Temperature {
        Temperature::new(0.0_f64)
    }
}

impl Temperature {
    fn to_fahrenheit(&self) -> f64 {
        ((self.celsius * (9.0_f64)) / (5.0_f64)) + (32.0_f64)
    }
}

impl ::std::fmt::Display for Temperature {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Temperature(celsius={})", self.celsius)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MathHelper {
}

impl MathHelper {
    fn new() -> Self {
        Self {  }
    }
}

impl MathHelper {
    fn clamp(value: f64, low: f64, high: f64) -> f64 {
        if value < low {
            return low;
        }
        if value > high {
            return high;
        }
        value
    }
}

impl MathHelper {
    fn is_positive(x: f64) -> bool {
        x > (0.0_f64)
    }
}

fn main() {
    let c: Circle = Circle::new("red".to_string(), 5.0_f64);
    let r: Rectangle = Rectangle::new("blue".to_string(), 3.0_f64, 4.0_f64);
    println!("{}", c.describe());
    println!("{}", c.area());
    println!("{}", c.shape.color.clone());
    println!("{}", r.describe());
    println!("{}", r.area());
    println!("{}", r.shape.color.clone());
    let boiling: Temperature = Temperature::new(100.0_f64);
    println!("{}", boiling.to_fahrenheit());
    let body: Temperature = Temperature::from_fahrenheit(98.6_f64);
    println!("{}", body.celsius);
    let zero: Temperature = Temperature::freezing();
    println!("{}", zero.celsius);
    println!("{}", MathHelper::clamp(15.0_f64, 0.0_f64, 10.0_f64));
    println!("{}", MathHelper::clamp(-(5.0_f64), 0.0_f64, 10.0_f64));
    println!("{}", MathHelper::is_positive(42.0_f64));
}
