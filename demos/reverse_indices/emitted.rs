// src/main.rs
fn reversed_values(values: &Vec<i64>) -> Vec<i64> {
    let mut out: Vec<i64> = vec![];
    for i in (-(1_i64) + (1_i64)..((values.len() as i64) - (1_i64)) + (1_i64)).rev() {
        out.push({
    let Some(__sifr_index_value) = ({
    let __sifr_index_list = &values;
    let __sifr_index_i = i;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
}) else {
        unreachable!("compiler-verified index should be in range");
    };
    __sifr_index_value
});
    }
    out
}

fn main() {
    assert!((format!("{:?}", reversed_values(&vec![4_i64, 5_i64, 6_i64])) == "[6, 5, 4]"));
    assert!((format!("{:?}", reversed_values(&vec![])) == "[]"));
    println!("reverse_indices: ok");
}
