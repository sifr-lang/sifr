fn double(x: i64) -> i64 {
    x * 2
}

fn greet(name: &str) -> String {
    format!("hello {name}")
}

fn is_positive(x: i64) -> bool {
    x > 0
}

fn log_value(x: i64) {
    println!("{}", x);
}

fn main() {
    println!("{}", double(21));
    println!("{:?}", greet("sifr"));
    println!("{}", is_positive(5));
    log_value(99);
}
