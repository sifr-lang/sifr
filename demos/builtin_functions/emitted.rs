// src/main.rs
use ::sifr_runtime::SifrInt;

use ::sifr_runtime::SifrRange;

fn main() {
    println!("max(3, 7) = {}", ::std::cmp::max(SifrInt::from_i64(3), SifrInt::from_i64(7)));
    assert!((format!("{}", format!("max(3, 7) = {}", ::std::cmp::max(SifrInt::from_i64(3), SifrInt::from_i64(7)))) == "max(3, 7) = 7"));
    println!("min(3, 7) = {}", ::std::cmp::min(SifrInt::from_i64(3), SifrInt::from_i64(7)));
    assert!((format!("{}", format!("min(3, 7) = {}", ::std::cmp::min(SifrInt::from_i64(3), SifrInt::from_i64(7)))) == "min(3, 7) = 3"));
    println!("pow(2, 10) = {}", SifrInt::from_i64(2).pow_known_valid(&SifrInt::from_i64(10)));
    assert!((format!("{}", format!("pow(2, 10) = {}", SifrInt::from_i64(2).pow_known_valid(&SifrInt::from_i64(10)))) == "pow(2, 10) = 1024"));
    let mut result: String = "".to_string();
    for i in SifrRange::new_known_nonzero(SifrInt::from_i64(0), SifrInt::from_i64(10), SifrInt::from_i64(2)) {
        if (&SifrInt::from(result.chars().count()) > &SifrInt::from_i64(0)) {
            result.push(' ');
        }
        result.push_str((format!("{}", i)).as_str());
    }
    println!("{}", result);
    assert!((format!("{}", result) == "0 2 4 6 8"));
}
