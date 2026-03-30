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
    println!("m23_3 project/test discovery parity contract demo:");
    println!("{}", helper::value());
}
