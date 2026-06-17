mod helper {
    pub fn msg() -> &'static str {
        "aligned"
    }
}

fn main() {
    println!("run_and_build run/build alignment demo:");
    println!("{}", helper::msg());
}
