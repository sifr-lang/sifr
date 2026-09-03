// src/main.rs
use ::sifr_runtime::SifrInt;
fn double(x: SifrInt) -> SifrInt {
    &x * &SifrInt::from_i64(2)
}
fn greet(name: &str) -> String {
    {
        let mut sifr_generated_concat: String = String::with_capacity(6usize + name.len());
        sifr_generated_concat.push_str("hello ");
        sifr_generated_concat.push_str(name);
        sifr_generated_concat
    }
}
fn is_positive(x: SifrInt) -> bool {
    &x > &SifrInt::from_i64(0)
}
fn log_value(x: SifrInt) {
    println!("{x}");
}
fn main() {
    println!("{}", double(SifrInt::from_i64(21)));
    println!("{}", greet(&"sifr".to_string()));
    println!("{}", is_positive(SifrInt::from_i64(5)));
    log_value(SifrInt::from_i64(99));
}
