// src/main.rs
use ::sifr_runtime::SifrInt;
fn increment(x: Option<SifrInt>) -> SifrInt {
    let Some(x) = x else {
        return SifrInt::from_i64(0);
    };
    ::std::ops::Add::add(&x, &SifrInt::from_i64(1))
}
fn double(x: Option<f64>) -> f64 {
    let Some(x) = x else {
        return 0.0_f64;
    };
    x * 2.0_f64
}
#[expect(
    clippy::ref_option,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn safe_len(items: &Option<Vec<String>>) -> SifrInt {
    SifrInt::from(items.as_ref().map_or(0_usize, ::std::vec::Vec::len))
}
#[expect(
    clippy::needless_pass_by_value,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn merge_lists(a: Vec<SifrInt>, b: Vec<SifrInt>) -> Vec<SifrInt> {
    {
        let mut sifr_generated_v = a;
        sifr_generated_v.extend(b.iter().cloned());
        sifr_generated_v
    }
}
#[expect(
    clippy::approx_constant,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn main() {
    let v: Option<SifrInt> = Some(SifrInt::from_i64(10));
    println!("{}", increment(v));
    let f: Option<f64> = Some(3.14_f64);
    println!("{}", double(f));
    let names: Option<Vec<String>> = Some(vec![
        "alice".to_string(),
        "bob".to_string(),
        "charlie".to_string(),
    ]);
    println!("{}", safe_len(&names));
    let merged: Vec<SifrInt> = merge_lists(
        vec![
            SifrInt::from_i64(1),
            SifrInt::from_i64(2),
            SifrInt::from_i64(3),
        ],
        vec![
            SifrInt::from_i64(4),
            SifrInt::from_i64(5),
            SifrInt::from_i64(6),
        ],
    );
    println!("{}", SifrInt::from(merged.len()));
}
