fn gen(flag: bool) -> Box<dyn Iterator<Item = i64>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: std::vec::IntoIter<i64> = Vec::new().into_iter();
    return Box::new(std::iter::from_fn(move || {
    if !__sifr_generator_initialized {
        let mut _yields: Vec<i64> = Vec::new();
        let mut i: i64 = 0 as i64;
        while i < (2 as i64) {
            if flag && (i == (0 as i64)) {
                break;
            }
            _yields.push(i);
            i += 1 as i64;
        }
        if !flag && (i == (2 as i64)) {
            _yields.push(99 as i64);
        }
        __sifr_generator_iter = _yields.into_iter();
        __sifr_generator_initialized = true;
    }
    return __sifr_generator_iter.next();
}));
}

fn main() {
    println!("generator_break_else yield/loop-path coverage demo:");
    for v in gen(false) {
        println!("{}", v);
    }
    for v in gen(true) {
        println!("{}", v);
    }
}
