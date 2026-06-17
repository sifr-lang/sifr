mod helper {
    pub fn compute(x: i64) -> i64 {
        x * 7
    }
}

fn main() {
    println!("reachable_imports import-closure discovery demo:");
    println!("{}", helper::compute(6));
}
