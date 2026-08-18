// src/main.rs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Counter {
    count: i64,
}

impl Counter {
    fn new(count: i64) -> Self {
        Self { count }
    }
}

impl Counter {
    fn increment(&mut self) {
        self.count += 1_i64;
    }
}

impl ::std::fmt::Display for Counter {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Counter(count={})", self.count)
    }
}

fn main() {
    let mut matrix: Vec<Vec<i64>> = vec![vec![0_i64, 0_i64, 0_i64], vec![0_i64, 0_i64, 0_i64], vec![0_i64, 0_i64, 0_i64]];
    {
        let __oi_raw = 0_i64;
        let __oi_norm = if __oi_raw < 0 { (matrix.len() as i64) + __oi_raw } else { __oi_raw };
        if __oi_norm >= 0 {
            if let Some(__row) = matrix.get_mut(__oi_norm as usize) {
                let __ii_raw = 0_i64;
                let __ii_norm = if __ii_raw < 0 { (__row.len() as i64) + __ii_raw } else { __ii_raw };
                if __ii_norm >= 0 {
                    if let Some(__elem) = __row.get_mut(__ii_norm as usize) {
                        *__elem = 1_i64;
                    }
                }
            }
        }
    }
    {
        let __oi_raw = 1_i64;
        let __oi_norm = if __oi_raw < 0 { (matrix.len() as i64) + __oi_raw } else { __oi_raw };
        if __oi_norm >= 0 {
            if let Some(__row) = matrix.get_mut(__oi_norm as usize) {
                let __ii_raw = 1_i64;
                let __ii_norm = if __ii_raw < 0 { (__row.len() as i64) + __ii_raw } else { __ii_raw };
                if __ii_norm >= 0 {
                    if let Some(__elem) = __row.get_mut(__ii_norm as usize) {
                        *__elem = 1_i64;
                    }
                }
            }
        }
    }
    {
        let __oi_raw = 2_i64;
        let __oi_norm = if __oi_raw < 0 { (matrix.len() as i64) + __oi_raw } else { __oi_raw };
        if __oi_norm >= 0 {
            if let Some(__row) = matrix.get_mut(__oi_norm as usize) {
                let __ii_raw = 2_i64;
                let __ii_norm = if __ii_raw < 0 { (__row.len() as i64) + __ii_raw } else { __ii_raw };
                if __ii_norm >= 0 {
                    if let Some(__elem) = __row.get_mut(__ii_norm as usize) {
                        *__elem = 1_i64;
                    }
                }
            }
        }
    }
    println!("{}", ({
    let __sifr_outer_list = &matrix;
    let __sifr_outer_i = 0_i64;
    let __sifr_outer_norm = if __sifr_outer_i < 0 { (__sifr_outer_list.len() as i64) + __sifr_outer_i } else { __sifr_outer_i };
    __sifr_outer_list.get(__sifr_outer_norm as usize).and_then(|__sifr_row| {
    let __sifr_inner_i = 0_i64;
    let __sifr_inner_norm = if __sifr_inner_i < 0 { (__sifr_row.len() as i64) + __sifr_inner_i } else { __sifr_inner_i };
    __sifr_row.get(__sifr_inner_norm as usize).copied()
})
}).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{}", ({
    let __sifr_outer_list = &matrix;
    let __sifr_outer_i = 1_i64;
    let __sifr_outer_norm = if __sifr_outer_i < 0 { (__sifr_outer_list.len() as i64) + __sifr_outer_i } else { __sifr_outer_i };
    __sifr_outer_list.get(__sifr_outer_norm as usize).and_then(|__sifr_row| {
    let __sifr_inner_i = 1_i64;
    let __sifr_inner_norm = if __sifr_inner_i < 0 { (__sifr_row.len() as i64) + __sifr_inner_i } else { __sifr_inner_i };
    __sifr_row.get(__sifr_inner_norm as usize).copied()
})
}).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{}", ({
    let __sifr_outer_list = &matrix;
    let __sifr_outer_i = 2_i64;
    let __sifr_outer_norm = if __sifr_outer_i < 0 { (__sifr_outer_list.len() as i64) + __sifr_outer_i } else { __sifr_outer_i };
    __sifr_outer_list.get(__sifr_outer_norm as usize).and_then(|__sifr_row| {
    let __sifr_inner_i = 2_i64;
    let __sifr_inner_norm = if __sifr_inner_i < 0 { (__sifr_row.len() as i64) + __sifr_inner_i } else { __sifr_inner_i };
    __sifr_row.get(__sifr_inner_norm as usize).copied()
})
}).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    let mut scores: Vec<i64> = vec![10_i64, 20_i64, 30_i64];
    {
        let __idx_raw = 0_i64;
        let __idx_norm = if __idx_raw < 0 { (scores.len() as i64) + __idx_raw } else { __idx_raw };
        if __idx_norm >= 0 {
            if let Some(__elem) = scores.get_mut(__idx_norm as usize) {
                *__elem += 5_i64;
            }
        }
    }
    {
        let __idx_raw = 1_i64;
        let __idx_norm = if __idx_raw < 0 { (scores.len() as i64) + __idx_raw } else { __idx_raw };
        if __idx_norm >= 0 {
            if let Some(__elem) = scores.get_mut(__idx_norm as usize) {
                *__elem -= 3_i64;
            }
        }
    }
    {
        let __idx_raw = 2_i64;
        let __idx_norm = if __idx_raw < 0 { (scores.len() as i64) + __idx_raw } else { __idx_raw };
        if __idx_norm >= 0 {
            if let Some(__elem) = scores.get_mut(__idx_norm as usize) {
                *__elem *= 2_i64;
            }
        }
    }
    let s0: Option<i64> = {
    let __sifr_index_list = &scores;
    let __sifr_index_i = 0_i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
};
    let s1: Option<i64> = {
    let __sifr_index_list = &scores;
    let __sifr_index_i = 1_i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
};
    let s2: Option<i64> = {
    let __sifr_index_list = &scores;
    let __sifr_index_i = 2_i64;
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
    let mut c = Counter::new(0_i64);
    c.increment();
    c.increment();
    c.increment();
    println!("{}", c.count);
}
