mod worker {
    pub fn call() -> i64 {
        3.9_f64.floor() as i64
    }
}

use worker::call;

fn main() {
    println!("external_modules non-main externals demo:");
    println!("{}", call());
}
