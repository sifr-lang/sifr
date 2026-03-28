const PI: f64 = 3.14159 as f64;

const MAX_RETRIES: i64 = 3 as i64;

fn __const_APP_NAME() -> String {
    return "sifr".to_string().to_string();
}

const DEBUG: bool = true;

#[derive(Debug, Clone, PartialEq)]
struct Temperature {
    celsius: f64,
}

impl Temperature {
    fn new(celsius: f64) -> Self {
        return Self { celsius: celsius };
    }
    fn from_fahrenheit(f: f64) -> Temperature {
        return Temperature::new(((f - (32.0 as f64)) * (5.0 as f64)) / (9.0 as f64));
    }
}

impl std::fmt::Display for Temperature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Temperature(celsius={})", self.celsius);
    }
}

fn circle_area(r: f64) -> f64 {
    return (PI * r) * r;
}

fn get_config() -> String {
    return format!(
        "{} (debug={}, retries={})",
        __const_APP_NAME(),
        DEBUG,
        MAX_RETRIES
    );
}

fn main() {
    println!("{}", circle_area(5.0 as f64));
    println!("{}", get_config());
    println!("{}", PI);
    println!("{}", MAX_RETRIES);
    let t: Temperature = Temperature::new(100.0 as f64);
    println!("{}", t.celsius);
    let t2: Temperature = Temperature::from_fahrenheit(212.0 as f64);
    println!("{}", t2.celsius);
}
