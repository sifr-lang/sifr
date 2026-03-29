const PI: f64 = 3.14159;

#[derive(Clone)]
struct Shape {
    name: String,
    color: String,
}

impl Shape {
    fn new(name: impl Into<String>, color: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            color: color.into(),
        }
    }

    fn describe(&self) -> String {
        format!("{} ({})", self.name, self.color)
    }
}

struct Circle {
    shape: Shape,
    radius: f64,
}

impl Circle {
    fn new(color: &str, radius: f64) -> Self {
        Self {
            shape: Shape::new("Circle", color),
            radius,
        }
    }

    fn area(&self) -> f64 {
        PI * self.radius * self.radius
    }

    fn describe(&self) -> String {
        format!("{} r={}", self.shape.name, self.radius)
    }
}

struct Rectangle {
    shape: Shape,
    width: f64,
    height: f64,
}

impl Rectangle {
    fn new(color: &str, width: f64, height: f64) -> Self {
        Self {
            shape: Shape::new("Rectangle", color),
            width,
            height,
        }
    }

    fn area(&self) -> f64 {
        self.width * self.height
    }

    fn describe(&self) -> String {
        format!("{} {}x{}", self.shape.name, self.width, self.height)
    }
}

struct Temperature {
    celsius: f64,
}

impl Temperature {
    fn new(celsius: f64) -> Self {
        Self { celsius }
    }

    fn from_fahrenheit(fahrenheit: f64) -> Self {
        Self::new((fahrenheit - 32.0) * 5.0 / 9.0)
    }

    fn freezing() -> Self {
        Self::new(0.0)
    }

    fn to_fahrenheit(&self) -> f64 {
        self.celsius * 9.0 / 5.0 + 32.0
    }
}

struct MathHelper;

impl MathHelper {
    fn clamp(value: f64, low: f64, high: f64) -> f64 {
        value.max(low).min(high)
    }

    fn is_positive(value: f64) -> bool {
        value > 0.0
    }
}

fn main() {
    let circle = Circle::new("red", 5.0);
    let rectangle = Rectangle::new("blue", 3.0, 4.0);

    println!("{}", circle.describe());
    println!("{}", circle.area());
    println!("{}", circle.shape.color);

    println!("{}", rectangle.describe());
    println!("{}", rectangle.area());
    println!("{}", rectangle.shape.color);

    let boiling = Temperature::new(100.0);
    println!("{}", boiling.to_fahrenheit());

    let body = Temperature::from_fahrenheit(98.6);
    println!("{}", body.celsius);

    let zero = Temperature::freezing();
    println!("{}", zero.celsius);

    println!("{}", MathHelper::clamp(15.0, 0.0, 10.0));
    println!("{}", MathHelper::clamp(-5.0, 0.0, 10.0));
    println!("{}", MathHelper::is_positive(42.0));

    let _ = circle.shape.describe();
}
