// src/main.rs
use ::sifr_runtime::SifrInt;
fn sort_values(mut values: Vec<SifrInt>) -> Vec<SifrInt> {
    values.sort();
    values
}
fn main() {
    assert_eq!(
        format!(
            "{:?}",
            sort_values(vec![
                SifrInt::from_i64(5),
                SifrInt::from_i64(1),
                SifrInt::from_i64(4),
                SifrInt::from_i64(2)
            ])
        ),
        "[1, 2, 4, 5]"
    );
    assert_eq!(
        format!("{:?}", sort_values(vec![SifrInt::from_i64(3)])),
        "[3]"
    );
    println!("mut_sort: ok");
}
