// src/main.rs
use ::sifr_runtime::SifrInt;

fn fibonacci(n: SifrInt) -> Box<dyn Iterator<Item = SifrInt>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: ::std::vec::IntoIter<SifrInt> = Vec::new().into_iter();
    Box::new(::std::iter::from_fn(move || {
    if !__sifr_generator_initialized {
        let mut _yields: Vec<SifrInt> = Vec::new();
        let mut a: SifrInt = SifrInt::from_i64(0);
        let mut b: SifrInt = SifrInt::from_i64(1);
        let mut count: SifrInt = SifrInt::from_i64(0);
        while &count < &n {
            _yields.push(a.clone());
            let temp: SifrInt = &a + &b;
            a = b.clone();
            b = temp.clone();
            count = &count + &SifrInt::from_i64(1);
        }
        __sifr_generator_iter = _yields.into_iter();
        __sifr_generator_initialized = true;
    }
    __sifr_generator_iter.next()
}))
}

fn squares(n: SifrInt) -> Box<dyn Iterator<Item = SifrInt>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: ::std::vec::IntoIter<SifrInt> = Vec::new().into_iter();
    Box::new(::std::iter::from_fn(move || {
    if !__sifr_generator_initialized {
        let mut _yields: Vec<SifrInt> = Vec::new();
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &n {
            _yields.push(&i * &i);
            i = &i + &SifrInt::from_i64(1);
        }
        __sifr_generator_iter = _yields.into_iter();
        __sifr_generator_initialized = true;
    }
    __sifr_generator_iter.next()
}))
}

fn evens(limit: SifrInt) -> Box<dyn Iterator<Item = SifrInt>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: ::std::vec::IntoIter<SifrInt> = Vec::new().into_iter();
    Box::new(::std::iter::from_fn(move || {
    if !__sifr_generator_initialized {
        let mut _yields: Vec<SifrInt> = Vec::new();
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &limit {
            if (&i.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == &SifrInt::from_i64(0)) {
                _yields.push(i.clone());
            }
            i = &i + &SifrInt::from_i64(1);
        }
        __sifr_generator_iter = _yields.into_iter();
        __sifr_generator_initialized = true;
    }
    __sifr_generator_iter.next()
}))
}

fn count_up(n: SifrInt) -> Box<dyn Iterator<Item = SifrInt>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: ::std::vec::IntoIter<SifrInt> = Vec::new().into_iter();
    Box::new(::std::iter::from_fn(move || {
    if !__sifr_generator_initialized {
        let mut _yields: Vec<SifrInt> = Vec::new();
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &n {
            _yields.push(i.clone());
            i = &i + &SifrInt::from_i64(1);
        }
        __sifr_generator_iter = _yields.into_iter();
        __sifr_generator_initialized = true;
    }
    __sifr_generator_iter.next()
}))
}

fn format_int_list(values: &Vec<SifrInt>) -> String {
    if &SifrInt::from(values.len()) == &SifrInt::from_i64(0) {
        return "[]".to_string();
    }
    let mut formatted: String = "[".to_string();
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &SifrInt::from(values.len())) {
        formatted.push_str((format!("{}", {
    let __sifr_index_value_option = {
    let __sifr_index_list = &values;
    let __sifr_index_i = i.clone();
    let __sifr_index_norm = __sifr_index_i.normalize_index_or_len(__sifr_index_list.len());
    __sifr_index_list.get(__sifr_index_norm).cloned()
};
    __sifr_index_value_option.as_slice()[0_usize].clone()
})).as_str());
        if (&(&i + &SifrInt::from_i64(1)) < &SifrInt::from(values.len())) {
            formatted.push_str(", ");
        }
        i = &i + &SifrInt::from_i64(1);
    }
    formatted.push(']');
    formatted
}

fn main() {
    let mut output: Vec<String> = vec![];
    output.push("=== Fibonacci (lazy for loop) ===".to_string());
    for fib in fibonacci(SifrInt::from_i64(8)) {
        output.push(format!("{}", fib));
    }
    output.push("=== Squares (collected) ===".to_string());
    let sq: Vec<SifrInt> = squares(SifrInt::from_i64(5)).collect::<Vec<_>>();
    output.push(format_int_list(&sq));
    output.push("=== Evens (conditional yield) ===".to_string());
    for e in evens(SifrInt::from_i64(10)) {
        output.push(format!("{}", e));
    }
    output.push("=== Count (lazy) ===".to_string());
    for c in count_up(SifrInt::from_i64(3)) {
        output.push(format!("{}", c));
    }
    output.push("=== Count (collected) ===".to_string());
    let nums: Vec<SifrInt> = count_up(SifrInt::from_i64(5)).collect::<Vec<_>>();
    output.push(format_int_list(&nums));
    assert!(output == vec!["=== Fibonacci (lazy for loop) ===".to_string(), "0".to_string(), "1".to_string(), "1".to_string(), "2".to_string(), "3".to_string(), "5".to_string(), "8".to_string(), "13".to_string(), "=== Squares (collected) ===".to_string(), "[0, 1, 4, 9, 16]".to_string(), "=== Evens (conditional yield) ===".to_string(), "0".to_string(), "2".to_string(), "4".to_string(), "6".to_string(), "8".to_string(), "=== Count (lazy) ===".to_string(), "0".to_string(), "1".to_string(), "2".to_string(), "=== Count (collected) ===".to_string(), "[0, 1, 2, 3, 4]".to_string()]);
    println!("Lazy iterator demo output:");
    for item in output.iter().cloned() {
        println!("{}", item);
    }
}
