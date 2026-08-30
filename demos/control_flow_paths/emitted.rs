// src/main.rs
mod __sifr_project_nominals {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ValueError {
        pub message: String,
    }
    impl ValueError {
        pub fn new(message: String) -> Self {
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
pub use __sifr_project_nominals::ValueError;
use ::sifr_runtime::SifrInt;
use ::sifr_runtime::SifrRange;
fn evaluate(seed: SifrInt) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    for n in SifrRange::new_known_nonzero(
        SifrInt::from_i64(0),
        seed.clone(),
        SifrInt::from_i64(1),
    ) {
        if &n == &SifrInt::from_i64(1) {
            continue;
        }
        if &n == &SifrInt::from_i64(6) {
            break;
        }
        if (&n.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == &SifrInt::from_i64(0)) {
            total = &total + &n;
        } else {
            total = &total + &SifrInt::from_i64(1);
        }
    }
    total.clone()
}
fn safe(seed: SifrInt) -> SifrInt {
    let __sifr_try_res: Result<SifrInt, ValueError> = (|| {
        let value: SifrInt = evaluate((seed).clone());
        if &value > &SifrInt::from_i64(3) {
            return Ok(value);
        }
        Err(ValueError::new("too small".to_string()))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return SifrInt::from_i64(42);
        }
    }
}
fn unreachable_tail() -> SifrInt {
    SifrInt::from_i64(9)
}
fn test_cfg_flow_matrix() {
    assert!((& safe(SifrInt::from_i64(8)) == & SifrInt::from_i64(8)));
    assert!((& safe(SifrInt::from_i64(3)) == & SifrInt::from_i64(42)));
    assert!((& unreachable_tail() == & SifrInt::from_i64(9)));
}
fn main() {
    println!("cfg flow activation regression matrix demo:");
    println!("{}", safe(SifrInt::from_i64(8)));
    println!("{}", safe(SifrInt::from_i64(3)));
    println!("{}", unreachable_tail());
}
