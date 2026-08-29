// src/main.rs
use ::sifr_runtime::SifrInt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Counter {
    count: SifrInt,
}

impl Counter {
    fn new(count: SifrInt) -> Self {
        Self { count }
    }
}

impl Counter {
    fn increment(&mut self) {
        self.count += SifrInt::from_i64(1);
    }
}

impl ::std::fmt::Display for Counter {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Counter(count={})", self.count)
    }
}

fn main() {
    let mut matrix: Vec<Vec<SifrInt>> = vec![vec![SifrInt::from_i64(0), SifrInt::from_i64(0), SifrInt::from_i64(0)], vec![SifrInt::from_i64(0), SifrInt::from_i64(0), SifrInt::from_i64(0)], vec![SifrInt::from_i64(0), SifrInt::from_i64(0), SifrInt::from_i64(0)]];
    {
        let __oi_raw = SifrInt::from_i64(0);
        let __oi_norm = __oi_raw.normalize_index_or_len(matrix.len());
        if let Some(__row) = matrix.get_mut(__oi_norm) {
            let __ii_raw = SifrInt::from_i64(0);
            let __ii_norm = __ii_raw.normalize_index_or_len(__row.len());
            if let Some(__elem) = __row.get_mut(__ii_norm) {
                *__elem = SifrInt::from_i64(1);
            }
        }
    }
    {
        let __oi_raw = SifrInt::from_i64(1);
        let __oi_norm = __oi_raw.normalize_index_or_len(matrix.len());
        if let Some(__row) = matrix.get_mut(__oi_norm) {
            let __ii_raw = SifrInt::from_i64(1);
            let __ii_norm = __ii_raw.normalize_index_or_len(__row.len());
            if let Some(__elem) = __row.get_mut(__ii_norm) {
                *__elem = SifrInt::from_i64(1);
            }
        }
    }
    {
        let __oi_raw = SifrInt::from_i64(2);
        let __oi_norm = __oi_raw.normalize_index_or_len(matrix.len());
        if let Some(__row) = matrix.get_mut(__oi_norm) {
            let __ii_raw = SifrInt::from_i64(2);
            let __ii_norm = __ii_raw.normalize_index_or_len(__row.len());
            if let Some(__elem) = __row.get_mut(__ii_norm) {
                *__elem = SifrInt::from_i64(1);
            }
        }
    }
    println!("{}", ({
    let __sifr_outer_list = &matrix;
    let __sifr_outer_i = SifrInt::from_i64(0);
    let __sifr_outer_norm = __sifr_outer_i.normalize_index_or_len(__sifr_outer_list.len());
    __sifr_outer_list.get(::sifr_runtime::to_usize_proven(&(__sifr_outer_norm))).and_then(|__sifr_row| {
    let __sifr_inner_i = SifrInt::from_i64(0);
    let __sifr_inner_norm = __sifr_inner_i.normalize_index_or_len(__sifr_row.len());
    __sifr_row.get(::sifr_runtime::to_usize_proven(&(__sifr_inner_norm))).cloned()
})
}).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{}", ({
    let __sifr_outer_list = &matrix;
    let __sifr_outer_i = SifrInt::from_i64(1);
    let __sifr_outer_norm = __sifr_outer_i.normalize_index_or_len(__sifr_outer_list.len());
    __sifr_outer_list.get(::sifr_runtime::to_usize_proven(&(__sifr_outer_norm))).and_then(|__sifr_row| {
    let __sifr_inner_i = SifrInt::from_i64(1);
    let __sifr_inner_norm = __sifr_inner_i.normalize_index_or_len(__sifr_row.len());
    __sifr_row.get(::sifr_runtime::to_usize_proven(&(__sifr_inner_norm))).cloned()
})
}).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{}", ({
    let __sifr_outer_list = &matrix;
    let __sifr_outer_i = SifrInt::from_i64(2);
    let __sifr_outer_norm = __sifr_outer_i.normalize_index_or_len(__sifr_outer_list.len());
    __sifr_outer_list.get(::sifr_runtime::to_usize_proven(&(__sifr_outer_norm))).and_then(|__sifr_row| {
    let __sifr_inner_i = SifrInt::from_i64(2);
    let __sifr_inner_norm = __sifr_inner_i.normalize_index_or_len(__sifr_row.len());
    __sifr_row.get(::sifr_runtime::to_usize_proven(&(__sifr_inner_norm))).cloned()
})
}).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    let mut scores: Vec<SifrInt> = vec![SifrInt::from_i64(10), SifrInt::from_i64(20), SifrInt::from_i64(30)];
    {
        let __idx_raw = SifrInt::from_i64(0);
        let __idx_norm = __idx_raw.normalize_index_or_len(scores.len());
        if let Some(__elem) = scores.get_mut(__idx_norm) {
            *__elem += SifrInt::from_i64(5);
        }
    }
    {
        let __idx_raw = SifrInt::from_i64(1);
        let __idx_norm = __idx_raw.normalize_index_or_len(scores.len());
        if let Some(__elem) = scores.get_mut(__idx_norm) {
            *__elem -= SifrInt::from_i64(3);
        }
    }
    {
        let __idx_raw = SifrInt::from_i64(2);
        let __idx_norm = __idx_raw.normalize_index_or_len(scores.len());
        if let Some(__elem) = scores.get_mut(__idx_norm) {
            *__elem *= SifrInt::from_i64(2);
        }
    }
    let s0: Option<SifrInt> = {
    let __sifr_index_list = &scores;
    let __sifr_index_i = SifrInt::from_i64(0);
    let __sifr_index_norm = __sifr_index_i.normalize_index_or_len(__sifr_index_list.len());
    __sifr_index_list.get(__sifr_index_norm).cloned()
};
    let s1: Option<SifrInt> = {
    let __sifr_index_list = &scores;
    let __sifr_index_i = SifrInt::from_i64(1);
    let __sifr_index_norm = __sifr_index_i.normalize_index_or_len(__sifr_index_list.len());
    __sifr_index_list.get(__sifr_index_norm).cloned()
};
    let s2: Option<SifrInt> = {
    let __sifr_index_list = &scores;
    let __sifr_index_i = SifrInt::from_i64(2);
    let __sifr_index_norm = __sifr_index_i.normalize_index_or_len(__sifr_index_list.len());
    __sifr_index_list.get(__sifr_index_norm).cloned()
};
    if let Some(s0) = s0.clone() {
        println!("{}", s0);
    }
    if let Some(s1) = s1.clone() {
        println!("{}", s1);
    }
    if let Some(s2) = s2.clone() {
        println!("{}", s2);
    }
    let mut c = Counter::new(SifrInt::from_i64(0));
    c.increment();
    c.increment();
    c.increment();
    println!("{}", c.count.clone());
}
