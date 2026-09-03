// src/main.rs
mod sifr_generated_project_nominals {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ValueError {
        pub message: String,
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
    clippy::assertions_on_constants,
    reason = "generated Rust preserves this exact typed Sifr source contract"
)]
fn assert_ok<T: Clone + 'static>(value: Result<T, Error>) {
    let sifr_generated_try_res: Result<(), Error> = (|| {
        let _out: T = value?;
        Ok(())
    })();
    if let Err(_e) = sifr_generated_try_res {
        assert!(false);
    }
}
#[expect(
    clippy::assertions_on_constants,
    reason = "generated Rust preserves this exact typed Sifr source contract"
)]
fn assert_err<T: Clone + 'static>(value: Result<T, Error>) {
    let sifr_generated_try_res: Result<(), Error> = (|| {
        let _out: T = value?;
        assert!(false);
        Ok(())
    })();
    let _ = sifr_generated_try_res;
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Error {
    message: String,
}
impl Error {
    const fn new(message: String) -> Self {
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
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn main() {
    println!("=== constructors ===");
    println!(
        "{:?}",
        "sifr"
            .to_string()
            .chars()
            .map(|sifr_generated_char| sifr_generated_char.to_string())
            .collect::<Vec<_>>()
    );
    println!(
        "{:?}",
        (
            SifrInt::from_i64(1),
            SifrInt::from_i64(2),
            SifrInt::from_i64(3)
        )
    );
    println!("{:?}", {
        let mut sifr_generated_dict_ctor = vec![("compiler".to_string(), SifrInt::from_i64(1))]
            .clone()
            .into_iter()
            .collect::<std::collections::HashMap<_, _>>();
        sifr_generated_dict_ctor.extend(
            {
                let mut sifr_generated_registry_dict_literal = ::std::collections::HashMap::new();
                sifr_generated_registry_dict_literal
                    .insert("demo".to_string(), SifrInt::from_i64(2));
                sifr_generated_registry_dict_literal
            }
            .clone(),
        );
        sifr_generated_dict_ctor
    });
    println!("=== helpers ===");
    println!("{:?}", {
        let mut sifr_generated_sorted_values = vec![
            SifrInt::from_i64(3),
            SifrInt::from_i64(1),
            SifrInt::from_i64(2),
        ]
        .into_iter()
        .collect::<Vec<_>>();
        let sifr_generated_sorted_reverse = false;
        sifr_generated_sorted_values.sort_by(
            |sifr_generated_sorted_left, sifr_generated_sorted_right| {
                if sifr_generated_sorted_reverse {
                    sifr_generated_sorted_right.cmp(&sifr_generated_sorted_left)
                } else {
                    sifr_generated_sorted_left.cmp(&sifr_generated_sorted_right)
                }
            },
        );
        sifr_generated_sorted_values
    });
    println!("{:?}", {
        let sifr_generated_sorted_values = vec![
            SifrInt::from_i64(3),
            SifrInt::from_i64(1),
            SifrInt::from_i64(2),
        ]
        .into_iter()
        .collect::<Vec<_>>();
        let sifr_generated_sorted_reverse = false;
        let mut sifr_generated_sorted_pairs = sifr_generated_sorted_values
            .into_iter()
            .map(|sifr_generated_sorted_value| {
                (
                    negate(sifr_generated_sorted_value.clone()),
                    sifr_generated_sorted_value,
                )
            })
            .collect::<Vec<_>>();
        sifr_generated_sorted_pairs.sort_by(
            |sifr_generated_sorted_left, sifr_generated_sorted_right| {
                if sifr_generated_sorted_reverse {
                    sifr_generated_sorted_right
                        .0
                        .cmp(&sifr_generated_sorted_left.0)
                } else {
                    sifr_generated_sorted_left
                        .0
                        .cmp(&sifr_generated_sorted_right.0)
                }
            },
        );
        sifr_generated_sorted_pairs
            .into_iter()
            .map(|sifr_generated_sorted_pair| sifr_generated_sorted_pair.1)
            .collect::<Vec<_>>()
    });
    println!("{:?}", {
        let mut sifr_generated_sorted_values = vec![
            SifrInt::from_i64(3),
            SifrInt::from_i64(1),
            SifrInt::from_i64(2),
        ]
        .into_iter()
        .collect::<Vec<_>>();
        let sifr_generated_sorted_reverse = true;
        sifr_generated_sorted_values.sort_by(
            |sifr_generated_sorted_left, sifr_generated_sorted_right| {
                if sifr_generated_sorted_reverse {
                    sifr_generated_sorted_right.cmp(&sifr_generated_sorted_left)
                } else {
                    sifr_generated_sorted_left.cmp(&sifr_generated_sorted_right)
                }
            },
        );
        sifr_generated_sorted_values
    });
    println!(
        "{:?}",
        Box::new(
            "sifr"
                .to_string()
                .chars()
                .map(|sifr_generated_char| sifr_generated_char.to_string())
                .rev()
        )
        .collect::<Vec<_>>()
    );
    println!(
        "{:?}",
        Box::new(
            vec!["a".to_string(), "b".to_string()]
                .into_iter()
                .enumerate()
                .map(|sifr_generated_pair| (
                    SifrInt::from(sifr_generated_pair.0) + SifrInt::from_i64(10),
                    sifr_generated_pair.1
                ))
        )
        .collect::<Vec<_>>()
    );
    println!(
        "{:?}",
        Box::new(
            vec![SifrInt::from_i64(1), SifrInt::from_i64(2)]
                .into_iter()
                .zip(vec!["a".to_string(), "b".to_string()])
                .zip(vec![true, false])
                .map(|sifr_generated_zip_item| (
                    sifr_generated_zip_item.0.0,
                    sifr_generated_zip_item.0.1,
                    sifr_generated_zip_item.1
                ))
        )
        .collect::<Vec<_>>()
    );
    println!(
        "{:?}",
        Box::new(
            vec![
                SifrInt::from_i64(1),
                SifrInt::from_i64(2),
                SifrInt::from_i64(3)
            ]
            .into_iter()
            .zip(vec![
                SifrInt::from_i64(4),
                SifrInt::from_i64(5),
                SifrInt::from_i64(6)
            ])
            .map(|sifr_generated_map_item| {
                let sifr_generated_map_arg_0 = sifr_generated_map_item.0;
                let sifr_generated_map_arg_1 = sifr_generated_map_item.1;
                add(sifr_generated_map_arg_0, sifr_generated_map_arg_1)
            })
        )
        .collect::<Vec<_>>()
    );
    println!(
        "{:?}",
        SifrRange::new_known_nonzero(
            SifrInt::from_i64(2),
            SifrInt::from_i64(9),
            SifrInt::from_i64(3)
        )
        .collect::<Vec<_>>()
    );
    println!("=== ord/chr ===");
    println!("{}", SifrInt::from_i64(65));
    println!("B");
    let ok_text: String = "Z".to_string();
    let bad_text: String = "ZZ".to_string();
    let ok_codepoint: SifrInt = SifrInt::from_i64(67);
    let huge: SifrInt = SifrInt::from_i64(1_114_112);
    assert_ok(
        {
            let sifr_generated_ord_chars = ok_text.chars().collect::<Vec<char>>();
            if let [sifr_generated_ord_char] = sifr_generated_ord_chars.as_slice() {
                Ok(SifrInt::from(*sifr_generated_ord_char as u32))
            } else {
                Err(ValueError {
                    message: "ord() expected a string of length 1".to_string(),
                })
            }
        }
        .map_err(::std::convert::Into::<Error>::into),
    );
    assert_err(
        {
            let sifr_generated_ord_chars = bad_text.chars().collect::<Vec<char>>();
            if let [sifr_generated_ord_char] = sifr_generated_ord_chars.as_slice() {
                Ok(SifrInt::from(*sifr_generated_ord_char as u32))
            } else {
                Err(ValueError {
                    message: "ord() expected a string of length 1".to_string(),
                })
            }
        }
        .map_err(::std::convert::Into::<Error>::into),
    );
    assert_ok(
        ok_codepoint
            .try_to_u32()
            .ok()
            .and_then(::std::char::from_u32)
            .map(|sifr_generated_chr| sifr_generated_chr.to_string())
            .ok_or_else(|| ValueError {
                message: "chr() arg not in range(0x110000)".to_string(),
            })
            .map_err(::std::convert::Into::<Error>::into),
    );
    assert_err(
        huge.try_to_u32()
            .ok()
            .and_then(::std::char::from_u32)
            .map(|sifr_generated_chr| sifr_generated_chr.to_string())
            .ok_or_else(|| ValueError {
                message: "chr() arg not in range(0x110000)".to_string(),
            })
            .map_err(::std::convert::Into::<Error>::into),
    );
}
