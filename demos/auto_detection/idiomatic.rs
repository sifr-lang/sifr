fn floor(value: f64) -> i64 {
    value.floor() as i64
}

fn main() {
    println!("auto_detection structural workspace demo:");
    println!("{}", floor(3.9));
}
