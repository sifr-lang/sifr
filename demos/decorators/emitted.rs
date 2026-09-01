// src/main.rs
use ::sifr_runtime::SifrInt;
fn greet(name: &str) -> String {
    {
        let mut sifr_generated_concat: String = String::with_capacity(7usize + name.len() + 1usize);
        sifr_generated_concat.push_str("Hello, ");
        sifr_generated_concat.push_str(name);
        sifr_generated_concat.push('!');
        sifr_generated_concat
    }
}
fn process(x: SifrInt) -> SifrInt {
    &x * &SifrInt::from_i64(2)
}
fn sum_all(nums: &[SifrInt]) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    for n in nums.iter().cloned() {
        total = &total + &n;
    }
    total.clone()
}
fn max_of(values: &[SifrInt]) -> SifrInt {
    let result: Option<SifrInt> = values.iter().cloned().max();
    let Some(result_value_9b51cd7cd76778c4) = result.clone() else {
        return SifrInt::from_i64(0);
    };
    result_value_9b51cd7cd76778c4
}
fn join_strings(sep: &str, parts: &[String]) -> String {
    let mut result: String = String::new();
    let mut i: SifrInt = SifrInt::from_i64(0);
    for p in parts.iter().cloned() {
        if &i > &SifrInt::from_i64(0) {
            result.push_str(sep);
        }
        result.push_str(p.as_str());
        i = &i + &SifrInt::from_i64(1);
    }
    result
}
fn main() {
    println!("{}", greet(&"World".to_string()));
    println!("{}", process(SifrInt::from_i64(21)));
    println!(
        "{}",
        sum_all(&vec![
            SifrInt::from_i64(1),
            SifrInt::from_i64(2),
            SifrInt::from_i64(3),
            SifrInt::from_i64(4),
            SifrInt::from_i64(5)
        ])
    );
    println!(
        "{}",
        sum_all(&vec![SifrInt::from_i64(10), SifrInt::from_i64(20)])
    );
    println!(
        "{}",
        max_of(&vec![
            SifrInt::from_i64(3),
            SifrInt::from_i64(7),
            SifrInt::from_i64(2),
            SifrInt::from_i64(9),
            SifrInt::from_i64(1)
        ])
    );
    println!(
        "{}",
        join_strings(
            &", ".to_string(),
            &vec![
                "Alice".to_string(),
                "Bob".to_string(),
                "Charlie".to_string()
            ]
        )
    );
}
