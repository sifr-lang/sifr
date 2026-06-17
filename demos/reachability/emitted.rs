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

impl std::error::Error for ValueError {
}

fn classify(flag: bool) -> i64 {
    let __sifr_try_res: Result<i64, ValueError> = (|| {
    if flag {
        return Ok(5 as i64);
    }
    return Err(ValueError::new("bad value".to_string()));
    unreachable!("sifr try/except return capture fell through");
})();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        },
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return 77 as i64;
        },
    }
}

fn main() {
    println!("reachability canonical flow truth queries demo:");
    println!("{}", classify(true));
    println!("{}", classify(false));
}
