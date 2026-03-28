mod sifr_math {
    pub fn floor(value: f64) -> i64 {
        value.floor() as i64
    }
}

fn main() {
    println!("m18_2 auto-detection demo:");
    println!("{}", sifr_math::floor(3.9));
}
