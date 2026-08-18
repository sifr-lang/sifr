// src/main.rs
fn countdown(n: i64) -> Box<dyn Iterator<Item = i64>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: ::std::vec::IntoIter<i64> = Vec::new().into_iter();
    Box::new(::std::iter::from_fn(move || {
    if !__sifr_generator_initialized {
        let mut _yields: Vec<i64> = Vec::new();
        let mut i: i64 = n;
        while i > (0_i64) {
            _yields.push(i);
            i -= 1_i64;
        }
        __sifr_generator_iter = _yields.into_iter();
        __sifr_generator_initialized = true;
    }
    __sifr_generator_iter.next()
}))
}

fn main() {
    let mut it: Box<dyn Iterator<Item = i64>> = countdown(3_i64);
    let first: Option<i64> = it.next();
    let second: Option<i64> = it.next();
    let remaining: Vec<i64> = it.collect::<Vec<_>>();
    let all_values: Vec<i64> = countdown(4_i64).collect::<Vec<_>>();
    assert!(first == Some(3_i64));
    assert!(second == Some(2_i64));
    assert!(remaining == vec![1_i64]);
    assert!(all_values == vec![4_i64, 3_i64, 2_i64, 1_i64]);
    println!("{}", (first).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{}", (second).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{:?}", remaining);
    println!("{:?}", all_values);
}
