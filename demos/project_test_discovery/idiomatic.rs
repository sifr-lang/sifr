mod shared {
    pub const BASE: i64 = 42;
}

mod helper {
    use super::shared::BASE;

    pub fn value() -> i64 {
        BASE
    }
}

fn main() {
    println!("project_test_discovery project/test discovery parity behavior demo:");
    println!("{}", helper::value());
}
