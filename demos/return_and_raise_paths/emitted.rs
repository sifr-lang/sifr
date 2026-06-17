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

fn classify(n: i64) -> i64 {
    let __sifr_try_res: Result<i64, ValueError> = (|| {
    if n > (0 as i64) {
        return Ok(n);
    } else {
        return Err(ValueError::new("non-positive".to_string()));
    }
    unreachable!("sifr try/except return capture fell through");
})();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        },
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return 99 as i64;
        },
    }
}

fn main() {
    println!("return_and_raise_paths control-flow effect query unification demo:");
    println!("{}", classify(7 as i64));
    println!("{}", classify(0 as i64));
}
