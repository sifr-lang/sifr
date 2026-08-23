// src/main.rs
fn r#gen(flag: bool) -> Box<dyn Iterator<Item = i64>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: ::std::vec::IntoIter<i64> = Vec::new().into_iter();
    Box::new(::std::iter::from_fn(move || {
    if !__sifr_generator_initialized {
        let mut _yields: Vec<i64> = Vec::new();
        let mut i: i64 = 0_i64;
        while i < (2_i64) {
            if flag && (i == (0_i64)) {
                break;
            }
            _yields.push(i);
            i += 1_i64;
        }
        if !flag && (i == (2_i64)) {
            _yields.push(99_i64);
        }
        __sifr_generator_iter = _yields.into_iter();
        __sifr_generator_initialized = true;
    }
    __sifr_generator_iter.next()
}))
}

fn main() {
    println!("generator_break_else yield/loop-path coverage demo:");
    for v in r#gen(false) {
        println!("{}", v);
    }
    for v in r#gen(true) {
        println!("{}", v);
    }
}
