// src/main.rs
fn mutate_and_return(mut items: Vec<i64>) -> Vec<i64> {
    {
        let __idx_raw = 0_i64;
        let __idx_norm = if __idx_raw < 0 { (items.len() as i64) + __idx_raw } else { __idx_raw };
        if __idx_norm >= 0 {
            if let Some(__elem) = items.get_mut(__idx_norm as usize) {
                *__elem = 9_i64;
            }
        }
    }
    {
        let __idx_raw = 1_i64;
        let __idx_norm = if __idx_raw < 0 { (items.len() as i64) + __idx_raw } else { __idx_raw };
        if __idx_norm >= 0 {
            if let Some(__elem) = items.get_mut(__idx_norm as usize) {
                *__elem = 10_i64;
            }
        }
    }
    items
}

fn mutate_borrowed(items: &mut Vec<i64>) -> i64 {
    {
        let __idx_raw = 0_i64;
        let __idx_norm = if __idx_raw < 0 { (items.len() as i64) + __idx_raw } else { __idx_raw };
        if __idx_norm >= 0 {
            if let Some(__elem) = items.get_mut(__idx_norm as usize) {
                *__elem = 14_i64;
            }
        }
    }
    items.len() as i64
}

fn main() {
    let values: Vec<i64> = vec![1_i64, 2_i64, 3_i64];
    let mut moved: Vec<i64> = mutate_and_return(values);
    println!("{}", ({
    let __sifr_index_list = &moved;
    let __sifr_index_i = 0_i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
}).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{}", ({
    let __sifr_index_list = &moved;
    let __sifr_index_i = 1_i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
}).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{}", mutate_borrowed(&mut moved));
}
