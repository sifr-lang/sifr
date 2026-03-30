mod helper {
    pub fn value() -> i64 {
        44
    }
}

fn main() {
    println!("m23_4 invocation-scoped temp workspace isolation demo:");
    println!("{}", helper::value());
}
