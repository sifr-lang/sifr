// src/main.rs
use ::sifr_runtime::SifrInt;
fn main() {
    let pairs: Vec<(SifrInt, SifrInt)> = vec![
        (SifrInt::from_i64(2), SifrInt::from_i64(5)),
        (SifrInt::from_i64(4), SifrInt::from_i64(7)),
    ];
    let mut totals: Vec<SifrInt> = Vec::new();
    for pair in pairs.iter().cloned() {
        totals.push(&pair.0.clone() + &pair.1.clone());
    }
    println!("{totals:?}");
    let mixed: Vec<Box<dyn ::std::any::Any>> = Vec::new();
    let mut count: SifrInt = SifrInt::from_i64(0);
    for _value in mixed.iter() {
        count = &count + &SifrInt::from_i64(1);
    }
    println!("{count}");
    println!("clone_generic_cloning_hardening_demo: pass");
}
