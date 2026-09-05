// src/main.rs
use ::sifr_runtime::SifrInt;
#[expect(
    clippy::many_single_char_names,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn main() {
    let a: SifrInt = SifrInt::from_i64(10);
    let b: SifrInt = SifrInt::from_i64(12);
    println!("{}", ::std::ops::BitAnd::bitand(&a, &b));
    println!("{}", ::std::ops::BitOr::bitor(&a, &b));
    println!("{}", ::std::ops::BitXor::bitxor(&a, &b));
    println!("{}", a.shl_known_valid(2_usize));
    println!("{}", a.shr_known_valid(1_usize));
    let x: SifrInt = SifrInt::from_i64(42);
    println!("{}", !x);
    println!("{}", !SifrInt::from_i64(0));
    let y: SifrInt = ::std::ops::Neg::neg(&SifrInt::from_i64(7));
    println!("{y}");
    println!("{}", SifrInt::from_i64(42));
    let mut flags: SifrInt = SifrInt::from_i64(0);
    flags = ::std::ops::BitOr::bitor(&flags, &SifrInt::from_i64(1));
    flags = ::std::ops::BitOr::bitor(&flags, &SifrInt::from_i64(4));
    println!("{flags}");
    flags = ::std::ops::BitAnd::bitand(&flags, &SifrInt::from_i64(3));
    println!("{flags}");
    flags = ::std::ops::BitXor::bitxor(&flags, &SifrInt::from_i64(3));
    println!("{flags}");
    flags = flags.shl_known_valid(0_usize);
    println!("{flags}");
    flags = flags.shr_known_valid(2_usize);
    println!("{flags}");
    let r: SifrInt = SifrInt::from_i64(99);
    let q: SifrInt = r.clone();
    let p: SifrInt = q.clone();
    println!("{p}");
    println!("{q}");
    println!("{r}");
}
