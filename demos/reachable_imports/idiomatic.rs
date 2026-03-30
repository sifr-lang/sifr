mod helper {
    pub fn compute(x: i64) -> i64 {
        x * 7
    }
}

fn main() {
    println!("m23_1 import-closure discovery demo:");
    println!("{}", helper::compute(6));
}
