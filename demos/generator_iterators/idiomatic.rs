fn gen_pairs(limit: i64) -> Box<dyn Iterator<Item = i64>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: std::vec::IntoIter<i64> = Vec::new().into_iter();
    return Box::new(std::iter::from_fn(move || {
        if !__sifr_generator_initialized {
            let mut _yields: Vec<i64> = Vec::new();
            let mut i: i64 = 0 as i64;
            while i < limit {
                _yields.push(i);
                i = i + (1 as i64);
                if i < limit {
                    _yields.push(i);
                    i = i + (1 as i64);
                }
            }
            __sifr_generator_iter = _yields.into_iter();
            __sifr_generator_initialized = true;
        }
        return __sifr_generator_iter.next();
    }));
}

fn gen_even(xs: &Vec<i64>) -> Box<dyn Iterator<Item = i64>> {
    let xs = xs.clone();
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: std::vec::IntoIter<i64> = Vec::new().into_iter();
    return Box::new(std::iter::from_fn(move || {
        if !__sifr_generator_initialized {
            let mut _yields: Vec<i64> = Vec::new();
            for x in xs.iter().copied() {
                if (x % (2 as i64)) == (0 as i64) {
                    _yields.push(x);
                }
            }
            __sifr_generator_iter = _yields.into_iter();
            __sifr_generator_initialized = true;
        }
        return __sifr_generator_iter.next();
    }));
}

fn main() {
    let xs: Vec<i64> = vec![1 as i64, 2 as i64, 3 as i64, 4 as i64, 5 as i64];
    let mut squares: Box<dyn Iterator<Item = i64>> = Box::new(xs.iter().copied().filter_map(|x| {
        if (x % (2 as i64)) == (0 as i64) {
            Some(x * x)
        } else {
            None
        }
    }));
    println!("{:?}", squares.collect::<Vec<_>>());
    println!("{:?}", gen_pairs(5 as i64).collect::<Vec<_>>());
    println!("{:?}", gen_even(&xs).collect::<Vec<_>>());
}
