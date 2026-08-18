// src/main.rs
fn main() {
    let mut items: Vec<i64> = vec![1_i64, 2_i64, 3_i64];
    {
        let __idx_raw = -(1_i64);
        let __idx_norm = if __idx_raw < 0 { (items.len() as i64) + __idx_raw } else { __idx_raw };
        if __idx_norm >= 0 {
            if let Some(__elem) = items.get_mut(__idx_norm as usize) {
                *__elem = 9_i64;
            }
        }
    }
    {
        let __idx_raw = -(2_i64);
        let __idx_norm = if __idx_raw < 0 { (items.len() as i64) + __idx_raw } else { __idx_raw };
        if __idx_norm >= 0 {
            if let Some(__elem) = items.get_mut(__idx_norm as usize) {
                *__elem += 5_i64;
            }
        }
    }
    {
        let __idx_raw = -(1_i64);
        let __idx_norm = if __idx_raw < 0 { (items.len() as i64) + __idx_raw } else { __idx_raw };
        if (__idx_norm >= 0) && ((__idx_norm as usize) < items.len()) {
            let _ = items.remove(__idx_norm as usize);
        }
    }
    {
        let __idx_raw = -(5_i64);
        let __idx_norm = if __idx_raw < 0 { (items.len() as i64) + __idx_raw } else { __idx_raw };
        if (__idx_norm >= 0) && ((__idx_norm as usize) < items.len()) {
            let _ = items.remove(__idx_norm as usize);
        }
    }
    println!("indexing_rules indexing and semantics parity fixes demo:");
    println!("{:?}", items);
}
