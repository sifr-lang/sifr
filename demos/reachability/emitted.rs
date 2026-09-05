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
pub use sifr_generated_project_nominals::ValueError;
fn classify(flag: bool) -> SifrInt {
    let sifr_generated_try_res: Result<SifrInt, ValueError> = (|| {
        if flag {
            return Ok(SifrInt::from_i64(5));
        }
        Err(ValueError::new("bad value".to_string()))
    })();
    sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
        let _e = sifr_generated_try_err;
        SifrInt::from_i64(77)
    })
}
fn main() {
    println!("reachability canonical flow truth queries demo:");
    println!("{}", classify(true));
    println!("{}", classify(false));
}
