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
fn negate(x: i64) -> i64 {
    (0_i64) - x
}
fn add(x: i64, y: i64) -> i64 {
    x + y
}
fn main() {
    println!("=== constructors ===");
    println!(
        "{:?}", ("sifr".to_string()).chars().map(| __sifr_char | __sifr_char.to_string())
        .collect::< Vec < _ >> ()
    );
    println!("{:?}", (1_i64, 2_i64, 3_i64));
    println!(
        "{:?}", { let mut __sifr_dict_ctor = (vec![("compiler".to_string(), 1_i64)])
        .clone().into_iter().collect::< HashMap < _, _ >> (); __sifr_dict_ctor
        .extend((HashMap::from([("demo".to_string(), 2_i64)])).clone()); __sifr_dict_ctor
        }
    );
    println!("=== helpers ===");
    println!(
        "{:?}", { let mut __sifr_sorted_v = (vec![3_i64, 1_i64, 2_i64]).into_iter()
        .collect::< Vec < _ >> (); __sifr_sorted_v.sort(); if false { { __sifr_sorted_v
        .reverse(); } }; __sifr_sorted_v }
    );
    println!(
        "{:?}", { let mut __sifr_sorted_v = (vec![3_i64, 1_i64, 2_i64]).into_iter()
        .collect::< Vec < _ >> (); __sifr_sorted_v.sort_by(| __left, __right | {
        negate(__left.clone()).cmp(& negate(__right.clone())) }); if false { {
        __sifr_sorted_v.reverse(); } }; __sifr_sorted_v }
    );
    println!(
        "{:?}", { let mut __sifr_sorted_v = (vec![3_i64, 1_i64, 2_i64]).into_iter()
        .collect::< Vec < _ >> (); __sifr_sorted_v.sort(); if true { { __sifr_sorted_v
        .reverse(); } }; __sifr_sorted_v }
    );
    println!(
        "{:?}", Box::new(("sifr".to_string()).chars().map(| __sifr_char | __sifr_char
        .to_string()).rev()).collect::< Vec < _ >> ()
    );
    println!(
        "{:?}", Box::new((vec!["a".to_string(), "b".to_string()]).into_iter().enumerate()
        .map(| __pair | ((__pair.0 as i64) + (10_i64), __pair.1))).collect::< Vec < _ >>
        ()
    );
    println!(
        "{:?}", Box::new((vec![1_i64, 2_i64]).into_iter().zip((vec!["a".to_string(), "b"
        .to_string()]).into_iter()).zip((vec![true, false]).into_iter()).map(| __zip_item
        | (__zip_item.0.0, __zip_item.0.1, __zip_item.1))).collect::< Vec < _ >> ()
    );
    println!(
        "{:?}", Box::new((vec![1_i64, 2_i64, 3_i64]).into_iter().zip((vec![4_i64, 5_i64,
        6_i64]).into_iter()).map(| __map_item | { let __map_arg_0 = __map_item.0; let
        __map_arg_1 = __map_item.1; add(__map_arg_0, __map_arg_1) }).into_iter())
        .collect::< Vec < _ >> ()
    );
    println!("{:?}", (2_i64..9_i64).step_by((3_i64) as usize).collect::< Vec < _ >> ());
    println!("=== ord/chr ===");
    println!("{}", 65_i64);
    println!("B");
    let ok_text: String = "Z".to_string();
    let bad_text: String = "ZZ".to_string();
    let ok_codepoint: i64 = 67_i64;
    let huge: i64 = 1114112_i64;
    assert_ok(
        ({
            let __sifr_ord_chars = (ok_text).chars().collect::<Vec<char>>();
            if __sifr_ord_chars.len() == 1 {
                Ok(__sifr_ord_chars[0] as i64)
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
                Ok(__sifr_ord_chars[0] as i64)
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
        (::std::char::from_u32((ok_codepoint) as u32)
            .map(|__sifr_chr| __sifr_chr.to_string())
            .ok_or_else(|| ValueError {
                message: "chr() arg not in range(0x110000)".to_string(),
            }))
            .map_err(|__sifr_error_value| ::std::convert::Into::<
                Error,
            >::into(__sifr_error_value)),
    );
    assert_err(
        (::std::char::from_u32((huge) as u32)
            .map(|__sifr_chr| __sifr_chr.to_string())
            .ok_or_else(|| ValueError {
                message: "chr() arg not in range(0x110000)".to_string(),
            }))
            .map_err(|__sifr_error_value| ::std::convert::Into::<
                Error,
            >::into(__sifr_error_value)),
    );
}
