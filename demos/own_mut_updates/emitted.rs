// src/main.rs
fn increment_all(mut values: Vec<i64>) -> Vec<i64> {
    for i in 0_i64..values.len() as i64 {
        {
            let __assign_value = values[i as usize] + (1_i64);
            {
                let __idx_raw = i;
                let __idx_norm = if __idx_raw < 0 { (values.len() as i64) + __idx_raw } else { __idx_raw };
                if __idx_norm >= 0 {
                    if let Some(__elem) = values.get_mut(__idx_norm as usize) {
                        *__elem = __assign_value;
                    }
                }
            }
        }
    }
    values
}

fn clear_all(mut values: Vec<i64>) -> Vec<i64> {
    for i in 0_i64..values.len() as i64 {
        {
            let __idx_raw = i;
            let __idx_norm = if __idx_raw < 0 { (values.len() as i64) + __idx_raw } else { __idx_raw };
            if __idx_norm >= 0 {
                if let Some(__elem) = values.get_mut(__idx_norm as usize) {
                    *__elem = 0_i64;
                }
            }
        }
    }
    values
}

fn main() {
    assert!((format!("{:?}", increment_all(vec![1_i64, 2_i64, 3_i64])) == "[2, 3, 4]"));
    assert!((format!("{:?}", clear_all(vec![4_i64, 5_i64, 6_i64])) == "[0, 0, 0]"));
    println!("own_mut_updates: ok");
}
