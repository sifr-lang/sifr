// src/main.rs
use ::sifr_runtime::SifrInt;

use ::sifr_runtime::SifrRange;

fn compute(limit: SifrInt) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    for n in SifrRange::new_known_nonzero(SifrInt::from_i64(0), limit.clone(), SifrInt::from_i64(1)) {
        if &n == &SifrInt::from_i64(2) {
            continue;
        }
        if &n == &SifrInt::from_i64(4) {
            break;
        }
        total = &total + &n;
    }
    total.clone()
}

fn main() {
    println!("valid_control_flow cfg validity invariants demo:");
    println!("{}", compute(SifrInt::from_i64(8)));
}
