fn increment_all(mut values: Vec<i64>) -> Vec<i64> {
    for i in 0 as i64..values.len() as i64 {
        {
            let __assign_value = values[i as usize] + (1 as i64);
            {
                let __idx_raw = i;
                let __idx_norm = if __idx_raw < 0 {
                    (values.len() as i64) + __idx_raw
                } else {
                    __idx_raw
                };
                if __idx_norm >= 0 {
                    if let Some(__elem) = values.get_mut(__idx_norm as usize) {
                        *__elem = __assign_value;
                    }
                }
            }
        }
    }
    return values;
}

fn clear_all(mut values: Vec<i64>) -> Vec<i64> {
    for i in 0 as i64..values.len() as i64 {
        {
            let __idx_raw = i;
            let __idx_norm = if __idx_raw < 0 {
                (values.len() as i64) + __idx_raw
            } else {
                __idx_raw
            };
            if __idx_norm >= 0 {
                if let Some(__elem) = values.get_mut(__idx_norm as usize) {
                    *__elem = 0 as i64;
                }
            }
        }
    }
    return values;
}

fn main() {
    assert!(
        format!("{:?}", increment_all(vec![1 as i64, 2 as i64, 3 as i64]))
            == "[2, 3, 4]".to_string()
    );
    assert!(
        format!("{:?}", clear_all(vec![4 as i64, 5 as i64, 6 as i64])) == "[0, 0, 0]".to_string()
    );
    println!("own_mut_updates: ok");
}
