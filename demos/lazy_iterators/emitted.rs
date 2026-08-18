// src/main.rs
fn fibonacci(n: i64) -> Box<dyn Iterator<Item = i64>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: ::std::vec::IntoIter<i64> = Vec::new().into_iter();
    Box::new(::std::iter::from_fn(move || {
    if !__sifr_generator_initialized {
        let mut _yields: Vec<i64> = Vec::new();
        let mut a: i64 = 0_i64;
        let mut b: i64 = 1_i64;
        let mut count: i64 = 0_i64;
        while count < n {
            _yields.push(a);
            let temp: i64 = a + b;
            a = b;
            b = temp;
            count += 1_i64;
        }
        __sifr_generator_iter = _yields.into_iter();
        __sifr_generator_initialized = true;
    }
    __sifr_generator_iter.next()
}))
}

fn squares(n: i64) -> Box<dyn Iterator<Item = i64>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: ::std::vec::IntoIter<i64> = Vec::new().into_iter();
    Box::new(::std::iter::from_fn(move || {
    if !__sifr_generator_initialized {
        let mut _yields: Vec<i64> = Vec::new();
        let mut i: i64 = 0_i64;
        while i < n {
            _yields.push(i * i);
            i += 1_i64;
        }
        __sifr_generator_iter = _yields.into_iter();
        __sifr_generator_initialized = true;
    }
    __sifr_generator_iter.next()
}))
}

fn evens(limit: i64) -> Box<dyn Iterator<Item = i64>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: ::std::vec::IntoIter<i64> = Vec::new().into_iter();
    Box::new(::std::iter::from_fn(move || {
    if !__sifr_generator_initialized {
        let mut _yields: Vec<i64> = Vec::new();
        let mut i: i64 = 0_i64;
        while i < limit {
            if (i % (2_i64)) == (0_i64) {
                _yields.push(i);
            }
            i += 1_i64;
        }
        __sifr_generator_iter = _yields.into_iter();
        __sifr_generator_initialized = true;
    }
    __sifr_generator_iter.next()
}))
}

fn count_up(n: i64) -> Box<dyn Iterator<Item = i64>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: ::std::vec::IntoIter<i64> = Vec::new().into_iter();
    Box::new(::std::iter::from_fn(move || {
    if !__sifr_generator_initialized {
        let mut _yields: Vec<i64> = Vec::new();
        let mut i: i64 = 0_i64;
        while i < n {
            _yields.push(i);
            i += 1_i64;
        }
        __sifr_generator_iter = _yields.into_iter();
        __sifr_generator_initialized = true;
    }
    __sifr_generator_iter.next()
}))
}

fn format_int_list(values: &Vec<i64>) -> String {
    if (values.len() as i64) == (0_i64) {
        return "[]".to_string();
    }
    let mut formatted: String = "[".to_string();
    let mut i: i64 = 0_i64;
    while (i < (values.len() as i64)) {
        formatted.push_str((format!("{}", {
    let Some(__sifr_index_value) = ({
    let __sifr_index_list = &values;
    let __sifr_index_i = i;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
}) else {
        unreachable!("compiler-verified index should be in range");
    };
    __sifr_index_value
})).as_str());
        if ((i + (1_i64)) < (values.len() as i64)) {
            formatted.push_str(", ");
        }
        i += 1_i64;
    }
    formatted.push(']');
    formatted
}

fn main() {
    let mut output: Vec<String> = vec![];
    output.push("=== Fibonacci (lazy for loop) ===".to_string());
    for fib in fibonacci(8_i64) {
        output.push(format!("{}", fib));
    }
    output.push("=== Squares (collected) ===".to_string());
    let sq: Vec<i64> = squares(5_i64).collect::<Vec<_>>();
    output.push(format_int_list(&sq));
    output.push("=== Evens (conditional yield) ===".to_string());
    for e in evens(10_i64) {
        output.push(format!("{}", e));
    }
    output.push("=== Count (lazy) ===".to_string());
    for c in count_up(3_i64) {
        output.push(format!("{}", c));
    }
    output.push("=== Count (collected) ===".to_string());
    let nums: Vec<i64> = count_up(5_i64).collect::<Vec<_>>();
    output.push(format_int_list(&nums));
    assert!(output == vec!["=== Fibonacci (lazy for loop) ===".to_string(), "0".to_string(), "1".to_string(), "1".to_string(), "2".to_string(), "3".to_string(), "5".to_string(), "8".to_string(), "13".to_string(), "=== Squares (collected) ===".to_string(), "[0, 1, 4, 9, 16]".to_string(), "=== Evens (conditional yield) ===".to_string(), "0".to_string(), "2".to_string(), "4".to_string(), "6".to_string(), "8".to_string(), "=== Count (lazy) ===".to_string(), "0".to_string(), "1".to_string(), "2".to_string(), "=== Count (collected) ===".to_string(), "[0, 1, 2, 3, 4]".to_string()]);
    println!("Lazy iterator demo output:");
    for item in output.iter().cloned() {
        println!("{}", item);
    }
}
