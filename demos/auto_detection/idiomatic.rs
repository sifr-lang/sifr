fn floor(value: f64) -> i64 {
    value.floor() as i64
}

fn main() {
    println!("auto_detection auto-detection demo:");
    println!("{}", floor(3.9));
}
