// src/main.rs
mod __sifr_project_nominals {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Error {
        pub message: String,
    }
    impl Error {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for Error {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for Error {}
}
pub use __sifr_project_nominals::Error;
use ::sifr_runtime::SifrInt;
fn normalize(n: SifrInt) -> SifrInt {
    match n {
        value if &value > &SifrInt::from_i64(0) => {
            return value;
        }
        _ => {
            return SifrInt::from_i64(0);
        }
    }
}
fn compute(values: &Vec<SifrInt>) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    {
        let _broke: bool = false;
        for value in values.iter().cloned() {
            let __sifr_try_res: Result<(), Error> = (|| {
                total = &total + &normalize((value).clone());
                Ok(())
            })();
            if let Err(__sifr_try_err) = __sifr_try_res {
                let e = __sifr_try_err.clone();
                total = &total + &SifrInt::from_i64(100);
            }
        }
        if !(_broke) {
            total = &total + &SifrInt::from_i64(1);
        }
    }
    total.clone()
}
fn main() {
    println!("loop_try_match canonical traversal layer behavior demo:");
    println!(
        "{}", compute(& vec![SifrInt::from_i64(3), SifrInt::from_i64(2), -&
        SifrInt::from_i64(1)])
    );
}
