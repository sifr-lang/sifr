mod helper {
    pub fn evaluate(n: i64) -> i64 {
        if n > 0 {
            if n > 10 {
                n
            } else {
                n + 10
            }
        } else {
            45
        }
    }
}

use helper::evaluate;

fn main() {
    println!("hir analysis consolidation regression matrix demo:");
    println!("{}", evaluate(10));
    println!("{}", evaluate(0));
}
