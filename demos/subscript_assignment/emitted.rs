#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Counter {
    count: i64,
}

impl Counter {
    fn new(count: i64) -> Self {
        return Self { count: count };
    }
    fn increment(&mut self) {
        self.count += 1 as i64;
    }
}

impl std::fmt::Display for Counter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Counter(count={})", self.count);
    }
}

fn main() {
    let mut matrix: Vec<Vec<i64>> = vec![vec![0 as i64, 0 as i64, 0 as i64], vec![0 as i64, 0 as i64, 0 as i64], vec![0 as i64, 0 as i64, 0 as i64]];
    {
        let __oi_raw = 0 as i64;
        let __oi_norm = if __oi_raw < 0 { (matrix.len() as i64) + __oi_raw } else { __oi_raw };
        if __oi_norm >= 0 {
            if let Some(__row) = matrix.get_mut(__oi_norm as usize) {
                let __ii_raw = 0 as i64;
                let __ii_norm = if __ii_raw < 0 { (__row.len() as i64) + __ii_raw } else { __ii_raw };
                if __ii_norm >= 0 {
                    if let Some(__elem) = __row.get_mut(__ii_norm as usize) {
                        *__elem = 1 as i64;
                    }
                }
            }
        }
    }
    {
        let __oi_raw = 1 as i64;
        let __oi_norm = if __oi_raw < 0 { (matrix.len() as i64) + __oi_raw } else { __oi_raw };
        if __oi_norm >= 0 {
            if let Some(__row) = matrix.get_mut(__oi_norm as usize) {
                let __ii_raw = 1 as i64;
                let __ii_norm = if __ii_raw < 0 { (__row.len() as i64) + __ii_raw } else { __ii_raw };
                if __ii_norm >= 0 {
                    if let Some(__elem) = __row.get_mut(__ii_norm as usize) {
                        *__elem = 1 as i64;
                    }
                }
            }
        }
    }
    {
        let __oi_raw = 2 as i64;
        let __oi_norm = if __oi_raw < 0 { (matrix.len() as i64) + __oi_raw } else { __oi_raw };
        if __oi_norm >= 0 {
            if let Some(__row) = matrix.get_mut(__oi_norm as usize) {
                let __ii_raw = 2 as i64;
                let __ii_norm = if __ii_raw < 0 { (__row.len() as i64) + __ii_raw } else { __ii_raw };
                if __ii_norm >= 0 {
                    if let Some(__elem) = __row.get_mut(__ii_norm as usize) {
                        *__elem = 1 as i64;
                    }
                }
            }
        }
    }
    println!("{}", (({
    let __sifr_index_list = &matrix;
    let __sifr_index_i = 0 as i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).cloned()
}).as_ref().and_then(|__v| {
    let __sifr_index_list = &__v;
    let __sifr_index_i = 0 as i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
})).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{}", (({
    let __sifr_index_list = &matrix;
    let __sifr_index_i = 1 as i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).cloned()
}).as_ref().and_then(|__v| {
    let __sifr_index_list = &__v;
    let __sifr_index_i = 1 as i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
})).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{}", (({
    let __sifr_index_list = &matrix;
    let __sifr_index_i = 2 as i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).cloned()
}).as_ref().and_then(|__v| {
    let __sifr_index_list = &__v;
    let __sifr_index_i = 2 as i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
})).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    let mut scores: Vec<i64> = vec![10 as i64, 20 as i64, 30 as i64];
    {
        let __idx_raw = 0 as i64;
        let __idx_norm = if __idx_raw < 0 { (scores.len() as i64) + __idx_raw } else { __idx_raw };
        if __idx_norm >= 0 {
            if let Some(__elem) = scores.get_mut(__idx_norm as usize) {
                *__elem += 5 as i64;
            }
        }
    }
    {
        let __idx_raw = 1 as i64;
        let __idx_norm = if __idx_raw < 0 { (scores.len() as i64) + __idx_raw } else { __idx_raw };
        if __idx_norm >= 0 {
            if let Some(__elem) = scores.get_mut(__idx_norm as usize) {
                *__elem -= 3 as i64;
            }
        }
    }
    {
        let __idx_raw = 2 as i64;
        let __idx_norm = if __idx_raw < 0 { (scores.len() as i64) + __idx_raw } else { __idx_raw };
        if __idx_norm >= 0 {
            if let Some(__elem) = scores.get_mut(__idx_norm as usize) {
                *__elem *= 2 as i64;
            }
        }
    }
    let s0: Option<i64> = {
    let __sifr_index_list = &scores;
    let __sifr_index_i = 0 as i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
};
    let s1: Option<i64> = {
    let __sifr_index_list = &scores;
    let __sifr_index_i = 1 as i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
};
    let s2: Option<i64> = {
    let __sifr_index_list = &scores;
    let __sifr_index_i = 2 as i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
};
    if let Some(s0) = s0 {
        println!("{}", s0);
    }
    if let Some(s1) = s1 {
        println!("{}", s1);
    }
    if let Some(s2) = s2 {
        println!("{}", s2);
    }
    let mut c = Counter::new(0 as i64);
    c.increment();
    c.increment();
    c.increment();
    println!("{}", c.count);
}
