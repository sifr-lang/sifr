// src/main.rs
use ::sifr_runtime::SifrInt;
fn shadow_parameter(mut value: SifrInt) -> SifrInt {
    value = &value + &SifrInt::from_i64(2);
    value.clone()
}
fn choose_label(flag: bool) -> String {
    if flag {
        "warm".to_string()
    } else {
        "cold".to_string()
    }
}
fn main() {
    assert_eq!(
        &shadow_parameter(SifrInt::from_i64(5)),
        &SifrInt::from_i64(7)
    );
    assert_eq!(choose_label(true), "warm");
    assert_eq!(choose_label(false), "cold");
    println!("local_shadowing: ok");
}
