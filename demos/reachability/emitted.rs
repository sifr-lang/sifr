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
fn classify(flag: bool) -> SifrInt {
    let __sifr_try_res: Result<SifrInt, ValueError> = (|| {
        if flag {
            return Ok(SifrInt::from_i64(5));
        }
        Err(ValueError::new("bad value".to_string()))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return SifrInt::from_i64(77);
        }
    }
}
fn main() {
    println!("reachability canonical flow truth queries demo:");
    println!("{}", classify(true));
    println!("{}", classify(false));
}
