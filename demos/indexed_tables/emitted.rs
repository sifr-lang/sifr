// src/main.rs
fn write_indices(size: i64) -> Vec<i64> {
    let mut out: Vec<i64> = {
    let mut __sifr_list_comp = vec![];
    for i in 0_i64..size {
        __sifr_list_comp.push(0_i64);
    }
    __sifr_list_comp
};
    for i in 0_i64..out.len() as i64 {
        {
            let __idx_raw = i;
            let __idx_norm = if __idx_raw < 0 { (out.len() as i64) + __idx_raw } else { __idx_raw };
            if __idx_norm >= 0 {
                if let Some(__elem) = out.get_mut(__idx_norm as usize) {
                    *__elem = i + (1_i64);
                }
            }
        }
    }
    out
}

fn main() {
    assert!((format!("{:?}", write_indices(4_i64)) == "[1, 2, 3, 4]"));
    assert!((format!("{:?}", write_indices(0_i64)) == "[]"));
    println!("indexed_tables: ok");
}
