// src/main.rs
use ::sifr_runtime::SifrInt;

fn shadow_parameter(mut value: SifrInt) -> SifrInt {
    value = &value + &SifrInt::from_i64(2);
    value.clone()
}

fn choose_label(flag: bool) -> String {
    let mut label: String = "cold".to_string();
    if flag {
        label = "warm".to_string();
    }
    label
}

fn main() {
    assert!((&shadow_parameter(SifrInt::from_i64(5)) == &SifrInt::from_i64(7)));
    assert!((choose_label(true) == "warm"));
    assert!((choose_label(false) == "cold"));
    println!("local_shadowing: ok");
}
