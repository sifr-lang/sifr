// src/main.rs
use ::sifr_runtime::SifrInt;
#[expect(
    clippy::needless_pass_by_value,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn greater_than_two(x: SifrInt) -> bool {
    x > SifrInt::from_i64(2)
}
fn main() {
    let nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(5),
        SifrInt::from_i64(1),
        SifrInt::from_i64(3),
        SifrInt::from_i64(4),
    ];
    let flags: Vec<bool> = vec![false, true, false];
    println!("{}", Box::new(flags.iter().copied()).any(|x| x));
    println!(
        "{:?}",
        Box::new(
            nums.iter()
                .filter(|&sifr_generated_filter_item| {
                    let sifr_generated_filter_value = sifr_generated_filter_item.clone();
                    greater_than_two(sifr_generated_filter_value)
                })
                .cloned()
        )
        .collect::<Vec<_>>()
    );
    println!("{:?}", {
        let mut sifr_generated_sorted_values = Box::new(nums.iter().cloned()).collect::<Vec<_>>();
        sifr_generated_sorted_values.sort_by(
            |sifr_generated_sorted_left, sifr_generated_sorted_right| {
                sifr_generated_sorted_left.cmp(sifr_generated_sorted_right)
            },
        );
        sifr_generated_sorted_values
    });
}
