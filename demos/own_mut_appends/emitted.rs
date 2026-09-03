// src/main.rs
use ::sifr_runtime::SifrInt;
fn append_zero(mut values: Vec<SifrInt>) -> Vec<SifrInt> {
    values.push(SifrInt::from_i64(0));
    values
}
fn append_marker(mut words: Vec<String>) -> Vec<String> {
    words.push("done".to_string());
    words
}
fn main() {
    assert_eq!(
        format!(
            "{:?}",
            append_zero(vec![
                SifrInt::from_i64(2),
                SifrInt::from_i64(3),
                SifrInt::from_i64(4)
            ])
        ),
        "[2, 3, 4, 0]"
    );
    assert_eq!(
        format!(
            "{:?}",
            append_marker(vec!["compile".to_string(), "check".to_string()])
        ),
        "[\"compile\", \"check\", \"done\"]"
    );
    println!("own_mut_appends: ok");
}
