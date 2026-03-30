mod provider {
    pub const BASE: i64 = 41;

    pub fn answer() -> i64 {
        BASE
    }
}

mod consumer {
    use super::provider::{answer, BASE};

    pub fn describe() -> i64 {
        answer() + BASE - 40
    }
}

fn main() {
    println!("{}", consumer::describe());
}
