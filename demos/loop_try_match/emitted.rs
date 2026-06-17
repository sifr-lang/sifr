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

impl std::error::Error for Error {
}

fn normalize(n: i64) -> i64 {
    match n {
        value if value > (0 as i64) => {
            return value;
        },
        _ => {
            return 0 as i64;
        },
    }
}

fn compute(values: &Vec<i64>) -> i64 {
    let mut total: i64 = 0 as i64;
    {
        let mut _broke: bool = false;
        for value in values.iter().copied() {
            let __sifr_try_res: Result<(), Error> = (|| {
    total = total + normalize(value);
    return Ok(());
})();
            if let Err(__sifr_try_err) = __sifr_try_res {
                let e = __sifr_try_err.clone();
                total = total + (100 as i64);
            }
        }
        if !(_broke) {
            total = total + (1 as i64);
        }
    }
    return total;
}

fn main() {
    println!("loop_try_match canonical traversal layer behavior demo:");
    println!("{}", compute(&vec![3 as i64, 2 as i64, -(1 as i64)]));
}
