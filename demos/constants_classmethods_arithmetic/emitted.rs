// src/main.rs
const PI: f64 = 3.14159_f64;

const MAX_RETRIES: i64 = 3_i64;

fn __const_APP_NAME() -> String {
    "sifr".to_string().to_string()
}

const DEBUG: bool = true;

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

impl ::std::fmt::Display for Temperature {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Temperature(celsius={})", self.celsius)
    }
}

fn circle_area(r: f64) -> f64 {
    (PI * r) * r
}

fn get_config() -> String {
    format!("{} (debug={}, retries={})", __const_APP_NAME(), DEBUG, MAX_RETRIES)
}

fn main() {
    println!("{}", circle_area(5.0_f64));
    println!("{}", get_config());
    println!("{}", PI);
    println!("{}", MAX_RETRIES);
    let t: Temperature = Temperature::new(100.0_f64);
    println!("{}", t.celsius);
    let t2: Temperature = Temperature::from_fahrenheit(212.0_f64);
    println!("{}", t2.celsius);
}
