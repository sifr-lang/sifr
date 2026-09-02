// src/main.rs
use ::sifr_runtime::SifrInt;
use ::sifr_runtime::SifrRange;
fn main() {
    let mut total: SifrInt = SifrInt::from_i64(0);
    for i in SifrRange::new_known_nonzero(
        SifrInt::from_i64(1),
        SifrInt::from_i64(6),
        SifrInt::from_i64(1),
    ) {
        if &i.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == &SifrInt::from_i64(0) {
            total = &total + &i;
        } else {
            total = &total + &(&i * &SifrInt::from_i64(2));
        }
    }
    let verdict: String = if &total > &SifrInt::from_i64(10) {
        "high".to_string()
    } else {
        "low".to_string()
    };
    println!("total = {total}");
    assert_eq!(format!("total = {total}"), "total = 24");
    println!("verdict = {verdict}");
    assert_eq!(format!("verdict = {verdict}"), "verdict = high");
}
