mod helper {
    pub fn value() -> i64 {
        18
    }
}

fn main() {
    println!("resolver_triggers resolver trigger matrix demo:");
    println!("{}", helper::value());
}
