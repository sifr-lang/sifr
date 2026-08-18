// src/main.rs
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

impl ::std::error::Error for Error {
}

fn normalize(n: i64) -> i64 {
    match n {
        value if value > (0_i64) => {
            return value;
        },
        _ => {
            return 0_i64;
        },
    }
}

fn compute(values: &Vec<i64>) -> i64 {
    let mut total: i64 = 0_i64;
    {
        let _broke: bool = false;
        for value in values.iter().copied() {
            let __sifr_try_res: Result<(), Error> = (|| {
    total += normalize(value);
    Ok(())
})();
            if let Err(__sifr_try_err) = __sifr_try_res {
                let e = __sifr_try_err.clone();
                total += 100_i64;
            }
        }
        if !(_broke) {
            total += 1_i64;
        }
    }
    total
}

fn main() {
    println!("loop_try_match canonical traversal layer behavior demo:");
    println!("{}", compute(&vec![3_i64, 2_i64, -(1_i64)]));
}
