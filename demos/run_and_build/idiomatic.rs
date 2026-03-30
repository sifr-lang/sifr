mod helper {
    pub fn msg() -> &'static str {
        "aligned"
    }
}

fn main() {
    println!("m18_1 run/build alignment demo:");
    println!("{}", helper::msg());
}
