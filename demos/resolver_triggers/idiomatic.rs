mod helper {
    pub fn value() -> i64 {
        18
    }
}

fn main() {
    println!("resolver_triggers explicit workspace import demo:");
    println!("{}", helper::value());
}
