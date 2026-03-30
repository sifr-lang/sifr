mod helper {
    pub fn area_like(r: f64) -> f64 {
        std::f64::consts::PI * r
    }
}

fn main() {
    println!("m22_2 project-aware check parity demo:");
    println!("{}", helper::area_like(3.0));
}
