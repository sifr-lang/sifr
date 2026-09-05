// src/main.rs
mod sifr_generated_project_nominals {}
use ::sifr_runtime::SifrInt;
fn normalize(n: &SifrInt) -> SifrInt {
    match (*n).clone() {
        value if value > SifrInt::from_i64(0) => value,
        _ => SifrInt::from_i64(0),
    }
}
fn compute(values: &[SifrInt]) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    {
        let sifr_generated_broke: bool = false;
        #[expect(
            clippy::explicit_iter_loop,
            reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
        )]
        for value in values.iter() {
            total = ::std::ops::Add::add(&total, &normalize(value));
        }
        if !sifr_generated_broke {
            total = ::std::ops::Add::add(&total, &SifrInt::from_i64(1));
        }
    }
    total
}
fn main() {
    println!("loop_try_match canonical traversal layer behavior demo:");
    println!(
        "{}",
        compute(&[
            SifrInt::from_i64(3),
            SifrInt::from_i64(2),
            ::std::ops::Neg::neg(&SifrInt::from_i64(1))
        ])
    );
}
