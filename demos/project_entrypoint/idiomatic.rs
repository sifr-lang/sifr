mod helper {
    pub fn adjusted(value: i64) -> i64 {
        value + 2
    }
}

fn main() {
    println!("m22_1 canonical frontend entry path demo:");
    println!("{}", helper::adjusted(5));
}
