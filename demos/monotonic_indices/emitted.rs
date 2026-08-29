// src/main.rs
use ::sifr_runtime::SifrInt;

use ::sifr_runtime::SifrRange;

fn active_indices(flags: &Vec<bool>) -> Vec<SifrInt> {
    let mut out: Vec<SifrInt> = vec![];
    for index in SifrRange::new_known_nonzero(SifrInt::from_i64(0), SifrInt::from(flags.len()), SifrInt::from_i64(1)) {
        if flags[::sifr_runtime::to_usize_proven(&(index))] {
            out.push(index.clone());
        }
    }
    out
}

fn main() {
    assert!((format!("{:?}", active_indices(&vec![true, false, true, true])) == "[0, 2, 3]"));
    assert!((format!("{:?}", active_indices(&vec![false, false])) == "[]"));
    println!("monotonic_indices: ok");
}
