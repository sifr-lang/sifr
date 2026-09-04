// src/main.rs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Shape {
    name: String,
    color: String,
}
impl Shape {
    const fn new(name: String, color: String) -> Self {
        let sifr_generated_field_value_c4bcadba8e631b86_6e616d65: String = name;
        let sifr_generated_field_value_77f5c18e246c6638_636f6c6f72: String = color;
        Self {
            name: sifr_generated_field_value_c4bcadba8e631b86_6e616d65,
            color: sifr_generated_field_value_77f5c18e246c6638_636f6c6f72,
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
        let sifr_generated_parent = Shape::new("Circle".to_string(), color);
        let sifr_generated_field_value_a293b946d5782cf3_726164697573: f64 = radius;
        Self {
            shape: sifr_generated_parent,
            radius: sifr_generated_field_value_a293b946d5782cf3_726164697573,
        }
    }
}
impl Circle {
    #[expect(
        clippy::approx_constant,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    fn area(&self) -> f64 {
        3.14159_f64 * self.radius * self.radius
    }
}
impl Circle {
    fn describe(&self) -> String {
        {
            let mut sifr_generated_concat: String =
                String::with_capacity(0usize.saturating_add(3usize).saturating_add(0usize));
            sifr_generated_concat.push_str(self.shape.name.clone().as_str());
            sifr_generated_concat.push_str(" r=");
            sifr_generated_concat.push_str(self.radius.to_string().as_str());
            sifr_generated_concat
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
        let sifr_generated_parent = Shape::new("Rectangle".to_string(), color);
        let sifr_generated_field_value_dbdacd932fd1e9bf_7769647468: f64 = width;
        let sifr_generated_field_value_17720bf67d347222_686569676874: f64 = height;
        Self {
            shape: sifr_generated_parent,
            width: sifr_generated_field_value_dbdacd932fd1e9bf_7769647468,
            height: sifr_generated_field_value_17720bf67d347222_686569676874,
        }
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
            let mut sifr_generated_concat: String = String::with_capacity(
                0usize
                    .saturating_add(1usize)
                    .saturating_add(0usize)
                    .saturating_add(1usize)
                    .saturating_add(0usize),
            );
            sifr_generated_concat.push_str(self.shape.name.clone().as_str());
            sifr_generated_concat.push(' ');
            sifr_generated_concat.push_str(self.width.to_string().as_str());
            sifr_generated_concat.push('x');
            sifr_generated_concat.push_str(self.height.to_string().as_str());
            sifr_generated_concat
        }
    }
}
impl ::std::fmt::Display for Rectangle {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(
            f,
            "Rectangle(shape={}, width={}, height={})",
            self.shape, self.width, self.height
        )
    }
}
#[derive(Debug, Clone, PartialEq)]
struct Temperature {
    celsius: f64,
}
impl Temperature {
    const fn new(celsius: f64) -> Self {
        let sifr_generated_field_value_69a867ea0a4ed8a3_63656c73697573: f64 = celsius;
        Self {
            celsius: sifr_generated_field_value_69a867ea0a4ed8a3_63656c73697573,
        }
    }
}
impl Temperature {
    fn from_fahrenheit(f: f64) -> Self {
        Self::new((f - 32.0_f64) * 5.0_f64 / 9.0_f64)
    }
}
impl Temperature {
    const fn freezing() -> Self {
        Self::new(0.0_f64)
    }
}
impl Temperature {
    fn to_fahrenheit(&self) -> f64 {
        self.celsius * 9.0_f64 / 5.0_f64 + 32.0_f64
    }
}
impl ::std::fmt::Display for Temperature {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Temperature(celsius={})", self.celsius)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MathHelper {}
impl MathHelper {
    const fn new() -> Self {
        Self {}
    }
}
impl ::std::default::Default for MathHelper {
    fn default() -> Self {
        Self::new()
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
        x > 0.0_f64
    }
}
fn main() {
    let c: Circle = Circle::new("red".to_string(), 5.0_f64);
    let r: Rectangle = Rectangle::new("blue".to_string(), 3.0_f64, 4.0_f64);
    println!("{}", c.describe());
    println!("{}", c.area());
    println!("{}", c.shape.color);
    println!("{}", r.describe());
    println!("{}", r.area());
    println!("{}", r.shape.color);
    let boiling: Temperature = Temperature::new(100.0_f64);
    println!("{}", boiling.to_fahrenheit());
    let body: Temperature = Temperature::from_fahrenheit(98.6_f64);
    println!("{}", body.celsius);
    let zero: Temperature = Temperature::freezing();
    println!("{}", zero.celsius);
    println!("{}", MathHelper::clamp(15.0_f64, 0.0_f64, 10.0_f64));
    println!("{}", MathHelper::clamp(-5.0_f64, 0.0_f64, 10.0_f64));
    println!("{}", MathHelper::is_positive(42.0_f64));
}
