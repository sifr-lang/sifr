mod helper {
    const BASE: i64 = 5;

    pub fn value(x: i64) -> i64 {
        BASE + 2 + x
    }
}

fn main() {
    println!("mode_consistency parity regression matrix demo:");
    println!("{}", helper::value(1));
}
