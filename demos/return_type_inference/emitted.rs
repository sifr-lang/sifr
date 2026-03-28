fn double(x: i64) -> i64 {
    return x * (2 as i64);
}

fn greet(name: &String) -> String {
    return format!("{}{}", "hello ".to_string(), name);
}

fn is_positive(x: i64) -> bool {
    return x > (0 as i64);
}

fn log_value(x: i64) {
    println!("{}", x);
}

fn main() {
    println!("{:?}", double(21 as i64));
    println!("{:?}", greet(&"sifr".to_string()));
    println!("{:?}", is_positive(5 as i64));
    log_value(99 as i64);
}
