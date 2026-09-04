// src/main.rs
use ::sifr_runtime::SifrInt;
fn main() {
    let pairs: Vec<(SifrInt, SifrInt)> = vec![
        (SifrInt::from_i64(2), SifrInt::from_i64(5)),
        (SifrInt::from_i64(4), SifrInt::from_i64(7)),
    ];
    let mut totals: Vec<SifrInt> = Vec::new();
    for pair in pairs.iter().cloned() {
        totals.push(pair.0 + pair.1);
    }
    println!("{totals:?}");
    let mixed: Vec<Box<dyn ::std::any::Any>> = Vec::new();
    let mut count: SifrInt = SifrInt::from_i64(0);
    #[expect(
        clippy::explicit_iter_loop,
        reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
    )]
    for _ in mixed.iter() {
        count = ::std::ops::Add::add(&count, &SifrInt::from_i64(1));
    }
    println!("{count}");
    println!("clone_generic_cloning_hardening_demo: pass");
}
