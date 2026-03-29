const PI: f64 = 3.14159;
const MAX_RETRIES: i64 = 3;
const APP_NAME: &str = "sifr";
const DEBUG: bool = true;

#[derive(Clone, Copy, Debug)]
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
}

fn circle_area(radius: f64) -> f64 {
    PI * radius * radius
}

fn get_config() -> String {
    format!("{APP_NAME} (debug={DEBUG}, retries={MAX_RETRIES})")
}

fn main() {
    println!("{}", circle_area(5.0));
    println!("{}", get_config());
    println!("{PI}");
    println!("{MAX_RETRIES}");

    let t = Temperature::new(100.0);
    println!("{}", t.celsius);

    let t2 = Temperature::from_fahrenheit(212.0);
    println!("{}", t2.celsius);
}
