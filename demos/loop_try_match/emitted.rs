// src/main.rs
mod sifr_generated_project_nominals {}
use ::sifr_runtime::SifrInt;
fn normalize(n: SifrInt) -> SifrInt {
    match n {
        value if &value > &SifrInt::from_i64(0) => value,
        _ => SifrInt::from_i64(0),
    }
}
fn compute(values: &[SifrInt]) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    {
        let sifr_generated_broke: bool = false;
        for value in values.iter().cloned() {
            total = &total + &normalize(value.clone());
        }
        if !sifr_generated_broke {
            total = &total + &SifrInt::from_i64(1);
        }
    }
    total.clone()
}
fn main() {
    println!("loop_try_match canonical traversal layer behavior demo:");
    println!(
        "{}",
        compute(&vec![
            SifrInt::from_i64(3),
            SifrInt::from_i64(2),
            -&SifrInt::from_i64(1)
        ])
    );
}
