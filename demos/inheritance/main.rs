// Reference: inheritance
// Reference: inheritance
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running `target/debug/sifr emit demos/inheritance_demo.sifr`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Shape {
    name: String,
    color: String,
}

impl Shape {
    fn new(name: String, color: String) -> Self {
        Self {
            name: name,
            color: color,
        }
    }

    fn describe(&self) -> String {
        return format!("{}{}{}{}", self.name.clone(), " (".to_string(), self.color.clone(), ")".to_string());
    }

}

#[derive(Debug, Clone, PartialEq)]
struct Circle {
    shape: Shape,
    radius: f64,
}

impl Circle {
    fn new(color: String, radius: f64) -> Self {
        Self {
            shape: Shape::new("Circle".to_string(), color),
            radius: radius,
        }
    }

    fn area(&self) -> f64 {
        return (3.14159_f64 * self.radius) * self.radius;
    }

    fn describe(&self) -> String {
        return format!("{}{}{}", self.shape.name.clone(), " r=".to_string(), format!("{}", self.radius));
    }

}

#[derive(Debug, Clone, PartialEq)]
struct Rectangle {
    shape: Shape,
    width: f64,
    height: f64,
}

impl Rectangle {
    fn new(color: String, width: f64, height: f64) -> Self {
        Self {
            shape: Shape::new("Rectangle".to_string(), color),
            width: width,
            height: height,
        }
    }

    fn area(&self) -> f64 {
        return self.width * self.height;
    }

    fn describe(&self) -> String {
        return format!("{}{}{}{}{}", self.shape.name.clone(), " ".to_string(), format!("{}", self.width), "x".to_string(), format!("{}", self.height));
    }

}

#[derive(Debug, Clone, PartialEq)]
struct Temperature {
    celsius: f64,
}

impl Temperature {
    fn new(celsius: f64) -> Self {
        Self {
            celsius: celsius,
        }
    }

    fn from_fahrenheit(f: f64) -> Temperature {
        return Temperature::new(((f - 32.0_f64) * 5.0_f64) / 9.0_f64);
    }

    fn freezing() -> Temperature {
        return Temperature::new(0.0_f64);
    }

    fn to_fahrenheit(&self) -> f64 {
        return ((self.celsius * 9.0_f64) / 5.0_f64) + 32.0_f64;
    }

}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MathHelper {
}

impl MathHelper {
    fn clamp(value: f64, low: f64, high: f64) -> f64 {
        if value < low {
            return low;
        }
        if value > high {
            return high;
        }
        return value;
    }

    fn is_positive(x: f64) -> bool {
        return x > 0.0_f64;
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
