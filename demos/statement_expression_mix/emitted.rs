// src/main.rs
use ::sifr_runtime::SifrInt;
fn main() {
    let mut acc: SifrInt = SifrInt::from_i64(0);
    let nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
    ];
    {
        let sifr_generated_broke: bool = false;
        #[expect(
            clippy::explicit_iter_loop,
            reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
        )]
        for n in nums.iter() {
            acc = ::std::ops::Add::add(&acc, n);
        }
        if !sifr_generated_broke {
            acc = ::std::ops::Add::add(&acc, &SifrInt::from_i64(1));
        }
    }
    let mut i: SifrInt = SifrInt::from_i64(0);
    {
        let sifr_generated_broke: bool = false;
        while i < SifrInt::from_i64(3) {
            acc = ::std::ops::Add::add(&acc, &i);
            i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
        }
        if !sifr_generated_broke {
            acc = ::std::ops::Add::add(&acc, &SifrInt::from_i64(2));
        }
    }
    let ready: bool = true;
    if ready {
        acc = ::std::ops::Add::add(&acc, &SifrInt::from_i64(10));
    } else {
        acc = ::std::ops::Add::add(&acc, &SifrInt::from_i64(100));
    }
    assert!(acc > SifrInt::from_i64(0));
    println!("acc = {acc}");
    assert_eq!(format!("acc = {acc}"), "acc = 22");
}
