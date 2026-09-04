// src/main.rs
mod sifr_generated_project_nominals {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ValueError {
        pub message: String,
    }
    impl ValueError {
        #[must_use]
        pub const fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for ValueError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ValueError {}
}
use ::sifr_runtime::SifrInt;
use ::sifr_runtime::SifrRange;
pub use sifr_generated_project_nominals::ValueError;
#[expect(
    clippy::needless_pass_by_value,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn evaluate(seed: SifrInt) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    for n in SifrRange::new_known_nonzero(SifrInt::from_i64(0), seed.clone(), SifrInt::from_i64(1))
    {
        if n == SifrInt::from_i64(1) {
            continue;
        }
        if n == SifrInt::from_i64(6) {
            break;
        }
        if n.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == SifrInt::from_i64(0) {
            total = ::std::ops::Add::add(&total, &n);
        } else {
            total = ::std::ops::Add::add(&total, &SifrInt::from_i64(1));
        }
    }
    total
}
#[expect(
    clippy::needless_pass_by_value,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn safe(seed: SifrInt) -> SifrInt {
    let sifr_generated_try_res: Result<SifrInt, ValueError> = (|| {
        let value: SifrInt = evaluate(seed.clone());
        if value > SifrInt::from_i64(3) {
            return Ok(value);
        }
        Err(ValueError::new("too small".to_string()))
    })();
    sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
        let _ = sifr_generated_try_err;
        SifrInt::from_i64(42)
    })
}
const fn unreachable_tail() -> SifrInt {
    SifrInt::from_i64(9)
}
fn main() {
    println!("cfg flow activation regression matrix demo:");
    println!("{}", safe(SifrInt::from_i64(8)));
    println!("{}", safe(SifrInt::from_i64(3)));
    println!("{}", unreachable_tail());
}
