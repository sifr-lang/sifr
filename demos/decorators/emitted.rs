// src/main.rs
use ::sifr_runtime::SifrInt;
fn greet(name: &str) -> String {
    {
        let mut sifr_generated_concat: String =
            String::with_capacity(7usize.saturating_add(name.len()).saturating_add(1usize));
        sifr_generated_concat.push_str("Hello, ");
        sifr_generated_concat.push_str(name);
        sifr_generated_concat.push('!');
        sifr_generated_concat
    }
}
#[expect(
    clippy::needless_pass_by_value,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn process(x: SifrInt) -> SifrInt {
    ::std::ops::Mul::mul(&x, &SifrInt::from_i64(2))
}
fn sum_all(nums: &[SifrInt]) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    #[expect(
        clippy::explicit_iter_loop,
        reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
    )]
    for n in nums.iter() {
        total = ::std::ops::Add::add(&total, n);
    }
    total
}
fn max_of(values: &[SifrInt]) -> SifrInt {
    let result: Option<SifrInt> = values.iter().cloned().max();
    let Some(result_value_9b51cd7cd76778c4) = result else {
        return SifrInt::from_i64(0);
    };
    result_value_9b51cd7cd76778c4
}
fn join_strings(sep: &str, parts: &[String]) -> String {
    let mut result: String = String::new();
    let mut i: SifrInt = SifrInt::from_i64(0);
    #[expect(
        clippy::explicit_iter_loop,
        reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
    )]
    for p in parts.iter() {
        if i > SifrInt::from_i64(0) {
            result.push_str(sep);
        }
        result.push_str(p.as_str());
        i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
    }
    result
}
fn main() {
    println!("{}", greet("World"));
    println!("{}", process(SifrInt::from_i64(21)));
    println!(
        "{}",
        sum_all(&[
            SifrInt::from_i64(1),
            SifrInt::from_i64(2),
            SifrInt::from_i64(3),
            SifrInt::from_i64(4),
            SifrInt::from_i64(5)
        ])
    );
    println!(
        "{}",
        sum_all(&[SifrInt::from_i64(10), SifrInt::from_i64(20)])
    );
    println!(
        "{}",
        max_of(&[
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
            ", ",
            &[
                "Alice".to_string(),
                "Bob".to_string(),
                "Charlie".to_string()
            ]
        )
    );
}
