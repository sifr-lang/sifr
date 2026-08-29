// src/main.rs
use ::sifr_runtime::SifrInt;

use ::sifr_runtime::SifrRange;

fn reversed_values(values: &Vec<SifrInt>) -> Vec<SifrInt> {
    let mut out: Vec<SifrInt> = vec![];
    for i in SifrRange::new_known_nonzero(&SifrInt::from(values.len()) - &SifrInt::from_i64(1), -(SifrInt::from_i64(1)), -(SifrInt::from_i64(1))) {
        out.push({
    let Some(__sifr_index_value) = ({
    let __sifr_index_list = &values;
    let __sifr_index_i = i.clone();
    let __sifr_index_norm = __sifr_index_i.normalize_index_or_len(__sifr_index_list.len());
    __sifr_index_list.get(__sifr_index_norm).cloned()
}) else {
        unreachable!("compiler-verified index should be in range");
    };
    __sifr_index_value
});
    }
    out
}

fn main() {
    assert!((format!("{:?}", reversed_values(&vec![SifrInt::from_i64(4), SifrInt::from_i64(5), SifrInt::from_i64(6)])) == "[6, 5, 4]"));
    assert!((format!("{:?}", reversed_values(&vec![])) == "[]"));
    println!("reverse_indices: ok");
}
