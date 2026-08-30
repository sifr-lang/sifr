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
use ::std::collections::HashMap;
use ::sifr_runtime::SifrInt;
use ::sifr_runtime::SifrRange;
fn assert_ok<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    value: Result<T, Error>,
) {
    let __sifr_try_res: Result<(), Error> = (|| {
        let out: T = value?;
        Ok(())
    })();
    if let Err(e) = __sifr_try_res {
        assert!(false);
    }
}
fn assert_err<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    value: Result<T, Error>,
) {
    let __sifr_try_res: Result<(), Error> = (|| {
        let out: T = value?;
        assert!(false);
        Ok(())
    })();
    if let Err(e) = __sifr_try_res {}
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Error {
    message: String,
}
impl Error {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}
impl ::std::error::Error for Error {}
impl From<ValueError> for Error {
    fn from(err: ValueError) -> Self {
        Self::new(err.message)
    }
}
fn negate(x: SifrInt) -> SifrInt {
    &SifrInt::from_i64(0) - &x
}
fn add(x: SifrInt, y: SifrInt) -> SifrInt {
    &x + &y
}
fn main() {
    println!("=== constructors ===");
    println!(
        "{:?}", ("sifr".to_string()).chars().map(| __sifr_char | __sifr_char.to_string())
        .collect::< Vec < _ >> ()
    );
    println!("{:?}", (SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3)));
    println!(
        "{:?}", { let mut __sifr_dict_ctor = (vec![("compiler".to_string(),
        SifrInt::from_i64(1))]).clone().into_iter().collect::< HashMap < _, _ >> ();
        __sifr_dict_ctor.extend((HashMap::from([("demo".to_string(),
        SifrInt::from_i64(2))])).clone()); __sifr_dict_ctor }
    );
    println!("=== helpers ===");
    println!(
        "{:?}", { let mut __sifr_sorted_v = (vec![SifrInt::from_i64(3),
        SifrInt::from_i64(1), SifrInt::from_i64(2)]).into_iter().collect::< Vec < _ >>
        (); __sifr_sorted_v.sort(); if false { { __sifr_sorted_v.reverse(); } };
        __sifr_sorted_v }
    );
    println!(
        "{:?}", { let mut __sifr_sorted_v = (vec![SifrInt::from_i64(3),
        SifrInt::from_i64(1), SifrInt::from_i64(2)]).into_iter().collect::< Vec < _ >>
        (); __sifr_sorted_v.sort_by(| __left, __right | { negate(__left.clone()).cmp(&
        negate(__right.clone())) }); if false { { __sifr_sorted_v.reverse(); } };
        __sifr_sorted_v }
    );
    println!(
        "{:?}", { let mut __sifr_sorted_v = (vec![SifrInt::from_i64(3),
        SifrInt::from_i64(1), SifrInt::from_i64(2)]).into_iter().collect::< Vec < _ >>
        (); __sifr_sorted_v.sort(); if true { { __sifr_sorted_v.reverse(); } };
        __sifr_sorted_v }
    );
    println!(
        "{:?}", Box::new(("sifr".to_string()).chars().map(| __sifr_char | __sifr_char
        .to_string()).rev()).collect::< Vec < _ >> ()
    );
    println!(
        "{:?}", Box::new((vec!["a".to_string(), "b".to_string()]).into_iter().enumerate()
        .map(| __pair | (SifrInt::from(__pair.0) + SifrInt::from_i64(10), __pair.1)))
        .collect::< Vec < _ >> ()
    );
    println!(
        "{:?}", Box::new((vec![SifrInt::from_i64(1), SifrInt::from_i64(2)]).into_iter()
        .zip((vec!["a".to_string(), "b".to_string()]).into_iter()).zip((vec![true,
        false]).into_iter()).map(| __zip_item | (__zip_item.0.0, __zip_item.0.1,
        __zip_item.1))).collect::< Vec < _ >> ()
    );
    println!(
        "{:?}", Box::new((vec![SifrInt::from_i64(1), SifrInt::from_i64(2),
        SifrInt::from_i64(3)]).into_iter().zip((vec![SifrInt::from_i64(4),
        SifrInt::from_i64(5), SifrInt::from_i64(6)]).into_iter()).map(| __map_item | {
        let __map_arg_0 = __map_item.0; let __map_arg_1 = __map_item.1; add(__map_arg_0,
        __map_arg_1) }).into_iter()).collect::< Vec < _ >> ()
    );
    println!(
        "{:?}", SifrRange::new_known_nonzero(SifrInt::from_i64(2), SifrInt::from_i64(9),
        SifrInt::from_i64(3)).collect::< Vec < _ >> ()
    );
    println!("=== ord/chr ===");
    println!("{}", SifrInt::from_i64(65));
    println!("B");
    let ok_text: String = "Z".to_string();
    let bad_text: String = "ZZ".to_string();
    let ok_codepoint: SifrInt = SifrInt::from_i64(67);
    let huge: SifrInt = SifrInt::from_i64(1114112);
    assert_ok(
        ({
            let __sifr_ord_chars = (ok_text).chars().collect::<Vec<char>>();
            if __sifr_ord_chars.len() == 1 {
                Ok(SifrInt::from(__sifr_ord_chars[0] as u32))
            } else {
                Err(ValueError {
                    message: "ord() expected a string of length 1".to_string(),
                })
            }
        })
            .map_err(|__sifr_error_value| ::std::convert::Into::<
                Error,
            >::into(__sifr_error_value)),
    );
    assert_err(
        ({
            let __sifr_ord_chars = (bad_text).chars().collect::<Vec<char>>();
            if __sifr_ord_chars.len() == 1 {
                Ok(SifrInt::from(__sifr_ord_chars[0] as u32))
            } else {
                Err(ValueError {
                    message: "ord() expected a string of length 1".to_string(),
                })
            }
        })
            .map_err(|__sifr_error_value| ::std::convert::Into::<
                Error,
            >::into(__sifr_error_value)),
    );
    assert_ok(
        ((ok_codepoint)
            .try_to_u32()
            .ok()
            .and_then(::std::char::from_u32)
            .map(|__sifr_chr| __sifr_chr.to_string())
            .ok_or_else(|| ValueError {
                message: "chr() arg not in range(0x110000)".to_string(),
            }))
            .map_err(|__sifr_error_value| ::std::convert::Into::<
                Error,
            >::into(__sifr_error_value)),
    );
    assert_err(
        ((huge)
            .try_to_u32()
            .ok()
            .and_then(::std::char::from_u32)
            .map(|__sifr_chr| __sifr_chr.to_string())
            .ok_or_else(|| ValueError {
                message: "chr() arg not in range(0x110000)".to_string(),
            }))
            .map_err(|__sifr_error_value| ::std::convert::Into::<
                Error,
            >::into(__sifr_error_value)),
    );
}
