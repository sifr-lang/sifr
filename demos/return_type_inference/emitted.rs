// src/main.rs
fn double(x: i64) -> i64 {
    x * (2_i64)
}

fn greet(name: &String) -> String {
    {
    let mut __sifr_concat: String = String::with_capacity(6usize + name.len());
    __sifr_concat.push_str("hello ");
    __sifr_concat.push_str((name).as_str());
    __sifr_concat
}
}

fn is_positive(x: i64) -> bool {
    x > (0_i64)
}

fn log_value(x: i64) {
    println!("{}", x);
}

fn main() {
    println!("{}", double(21_i64));
    println!("{}", greet(&"sifr".to_string()));
    println!("{}", is_positive(5_i64));
    log_value(99_i64);
}
