// src/main.rs
use ::sifr_runtime::SifrInt;
fn pick_value(maybe: Option<&SifrInt>) -> SifrInt {
    let maybe: Option<SifrInt> = maybe.cloned();
    let Some(maybe_value_531ab7a4be7bf10b) = maybe else {
        return SifrInt::from_i64(0);
    };
    maybe_value_531ab7a4be7bf10b
}
fn main() {
    println!("early_return_paths cfg integration behavior demo:");
    println!("{}", pick_value(Some(&SifrInt::from_i64(41))));
    println!("{}", pick_value(None));
}
