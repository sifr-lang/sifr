// src/main.rs
use ::sifr_runtime::SifrInt;

fn safe_add_one(x: Option<SifrInt>) -> SifrInt {
    let Some(x) = x.clone() else {
        return SifrInt::from_i64(0);
    };
    &x + &SifrInt::from_i64(1)
}

fn main() {
    println!("optional_arithmetic optional arithmetic soundness demo:");
    println!("{}", safe_add_one(Some(SifrInt::from_i64(5))));
    println!("{}", safe_add_one(None));
    let total: Option<SifrInt> = Some(SifrInt::from_i64(9));
    let count: Option<SifrInt> = Some(SifrInt::from_i64(3));
    if let Some(total) = total.clone() {
        if let Some(count) = count.clone() {
            println!("{}", (total).to_f64_proven_exact() / (count).to_f64_proven_exact());
        }
    }
    let missing_total: Option<SifrInt> = None;
    if missing_total.is_none() {
        println!("{}", SifrInt::from_i64(0));
    }
}
