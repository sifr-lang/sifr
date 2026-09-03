// src/main.rs
use ::sifr_runtime::SifrInt;
fn pick_value(maybe: Option<SifrInt>) -> SifrInt {
    let Some(maybe) = maybe.clone() else {
        return SifrInt::from_i64(0);
    };
    maybe
}
fn main() {
    println!("early_return_paths cfg integration behavior demo:");
    println!("{}", pick_value(Some(SifrInt::from_i64(41))));
    println!("{}", pick_value(None));
}
