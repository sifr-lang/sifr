mod worker {
    pub fn call() -> i64 {
        3.9_f64.floor() as i64
    }
}

use worker::call;

fn main() {
    println!("m17_2 non-main externals demo:");
    println!("{}", call());
}
