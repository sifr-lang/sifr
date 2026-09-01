// src/main.rs
use ::sifr_runtime::SifrInt;
#[expect(
    clippy::many_single_char_names,
    reason = "generated Rust preserves this exact typed Sifr source contract"
)]
fn main() {
    let a: SifrInt = SifrInt::from_i64(10);
    let b: SifrInt = SifrInt::from_i64(12);
    println!("{}", &a & &b);
    println!("{}", &a | &b);
    println!("{}", &a ^ &b);
    println!("{}", a.clone().shl_known_valid(2_usize));
    println!("{}", a.clone().shr_known_valid(1_usize));
    let x: SifrInt = SifrInt::from_i64(42);
    println!("{}", !x);
    println!("{}", !SifrInt::from_i64(0));
    let y: SifrInt = -&SifrInt::from_i64(7);
    println!("{y}");
    println!("{}", SifrInt::from_i64(42));
    let mut flags: SifrInt = SifrInt::from_i64(0);
    flags = &flags | &SifrInt::from_i64(1);
    flags = &flags | &SifrInt::from_i64(4);
    println!("{flags}");
    flags = &flags & &SifrInt::from_i64(3);
    println!("{flags}");
    flags = &flags ^ &SifrInt::from_i64(3);
    println!("{flags}");
    flags = flags.shl_known_valid(0_usize);
    println!("{flags}");
    flags = flags.shr_known_valid(2_usize);
    println!("{flags}");
    let mut p: SifrInt = SifrInt::from_i64(0);
    let mut q: SifrInt = SifrInt::from_i64(0);
    let mut r: SifrInt = SifrInt::from_i64(0);
    r = SifrInt::from_i64(99);
    q = r.clone();
    p = q.clone();
    println!("{p}");
    println!("{q}");
    println!("{r}");
}
