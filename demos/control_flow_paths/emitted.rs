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

fn evaluate(seed: i64) -> i64 {
    let mut total: i64 = 0 as i64;
    for n in 0 as i64..seed {
        if n == (1 as i64) {
            continue;
        }
        if n == (6 as i64) {
            break;
        }
        if (n % (2 as i64)) == (0 as i64) {
            total += n;
        } else {
            total += 1 as i64;
        }
    }
    return total;
}

fn safe(seed: i64) -> i64 {
    let __sifr_try_res: Result<i64, ValueError> = (|| {
    let value: i64 = evaluate(seed);
    if value > (3 as i64) {
        return Ok(value);
    }
    return Err(ValueError::new("too small".to_string()));
    unreachable!("sifr try/except return capture fell through");
})();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        },
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return 42 as i64;
        },
    }
}

fn unreachable_tail() -> i64 {
    return 9 as i64;
}

fn test_cfg_flow_matrix() {
    assert!(safe(8 as i64) == (8 as i64));
    assert!(safe(3 as i64) == (42 as i64));
    assert!(unreachable_tail() == (9 as i64));
}

fn main() {
    println!("cfg flow activation regression matrix demo:");
    println!("{}", safe(8 as i64));
    println!("{}", safe(3 as i64));
    println!("{}", unreachable_tail());
}
