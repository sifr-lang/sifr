mod helper {
    pub fn doubled(x: i64) -> i64 {
        x * 2
    }
}

fn main() {
    println!("m22_3 cross-mode diagnostic and exit contract demo:");
    println!("{}", helper::doubled(21));
}
