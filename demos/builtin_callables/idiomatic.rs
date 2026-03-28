use std::collections::HashMap;

// --- stdlib: sifr.test ---
fn assert_ok<T: Clone + std::fmt::Display + PartialOrd + 'static>(value: Result<T, Error>) {
    let __sifr_try_res: Result<(), Error> = (|| {
        let out: T = value?;
        return Ok(());
    })();
    if let Err(e) = __sifr_try_res {
        assert!(false);
    }
}
fn assert_err<T: Clone + std::fmt::Display + PartialOrd + 'static>(value: Result<T, Error>) {
    let __sifr_try_res: Result<(), Error> = (|| {
        let out: T = value?;
        assert!(false);
        return Ok(());
    })();
    if let Err(e) = __sifr_try_res {}
}

#[derive(Debug, Clone)]
struct Error {
    message: String,
}

impl Error {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone)]
struct ValueError {
    message: String,
}

impl ValueError {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}

impl std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for ValueError {}

fn negate(x: i64) -> i64 {
    return (0 as i64) - x;
}

fn add(x: i64, y: i64) -> i64 {
    return x + y;
}

fn main() {
    println!("=== constructors ===");
    println!(
        "{:?}",
        ("sifr".to_string())
            .chars()
            .map(|__sifr_char| __sifr_char.to_string())
            .collect::<Vec<_>>()
    );
    println!("{:?}", (1 as i64, 2 as i64, 3 as i64));
    println!("{:?}", {
        let mut __sifr_dict_ctor = (vec![("compiler".to_string(), 1 as i64)])
            .clone()
            .into_iter()
            .collect::<HashMap<_, _>>();
        __sifr_dict_ctor.extend((HashMap::from([("demo".to_string(), 2 as i64)])).clone());
        __sifr_dict_ctor
    });
    println!("=== helpers ===");
    println!("{:?}", {
        let mut __sifr_sorted_v = (vec![3 as i64, 1 as i64, 2 as i64])
            .into_iter()
            .collect::<Vec<_>>();
        __sifr_sorted_v.sort();
        if false {
            {
                __sifr_sorted_v.reverse();
            }
        };
        __sifr_sorted_v
    });
    println!("{:?}", {
        let mut __sifr_sorted_v = (vec![3 as i64, 1 as i64, 2 as i64])
            .into_iter()
            .collect::<Vec<_>>();
        __sifr_sorted_v.sort_by(|__left, __right| {
            let __left_key = __left.clone();
            let __right_key = __right.clone();
            return negate(__left_key).cmp(&negate(__right_key));
        });
        if false {
            {
                __sifr_sorted_v.reverse();
            }
        };
        __sifr_sorted_v
    });
    println!("{:?}", {
        let mut __sifr_sorted_v = (vec![3 as i64, 1 as i64, 2 as i64])
            .into_iter()
            .collect::<Vec<_>>();
        __sifr_sorted_v.sort();
        if true {
            {
                __sifr_sorted_v.reverse();
            }
        };
        __sifr_sorted_v
    });
    println!(
        "{:?}",
        Box::new(
            ("sifr".to_string())
                .chars()
                .map(|__sifr_char| __sifr_char.to_string())
                .rev()
        )
        .collect::<Vec<_>>()
    );
    println!(
        "{:?}",
        Box::new(
            (vec!["a".to_string(), "b".to_string()])
                .into_iter()
                .enumerate()
                .map(|__pair| ((__pair.0 as i64) + (10 as i64), __pair.1))
        )
        .collect::<Vec<_>>()
    );
    println!(
        "{:?}",
        Box::new(
            (vec![1 as i64, 2 as i64])
                .into_iter()
                .zip((vec!["a".to_string(), "b".to_string()]).into_iter())
                .zip((vec![true, false]).into_iter())
                .map(|__zip_item| (__zip_item.0 .0, __zip_item.0 .1, __zip_item.1))
        )
        .collect::<Vec<_>>()
    );
    println!(
        "{:?}",
        Box::new(
            (vec![1 as i64, 2 as i64, 3 as i64])
                .into_iter()
                .zip((vec![4 as i64, 5 as i64, 6 as i64]).into_iter())
                .map(|__map_item| {
                    let __map_arg_0 = __map_item.0;
                    let __map_arg_1 = __map_item.1;
                    return add(__map_arg_0, __map_arg_1);
                })
                .into_iter()
        )
        .collect::<Vec<_>>()
    );
    println!(
        "{:?}",
        (2 as i64..9 as i64)
            .step_by((3 as i64) as usize)
            .collect::<Vec<_>>()
    );
    println!("=== ord/chr ===");
    println!("{}", 65 as i64);
    println!("B");
    let ok_text: String = "Z".to_string();
    let bad_text: String = "ZZ".to_string();
    let ok_codepoint: i64 = 67 as i64;
    let huge: i64 = 1114112 as i64;
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
        .map_err(|__e| Error::new(__e.to_string())),
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
        .map_err(|__e| Error::new(__e.to_string())),
    );
    assert_ok(
        (std::char::from_u32((ok_codepoint) as u32)
            .map(|__sifr_chr| __sifr_chr.to_string())
            .ok_or_else(|| ValueError {
                message: "chr() arg not in range(0x110000)".to_string(),
            }))
        .map_err(|__e| Error::new(__e.to_string())),
    );
    assert_err(
        (std::char::from_u32((huge) as u32)
            .map(|__sifr_chr| __sifr_chr.to_string())
            .ok_or_else(|| ValueError {
                message: "chr() arg not in range(0x110000)".to_string(),
            }))
        .map_err(|__e| Error::new(__e.to_string())),
    );
}
