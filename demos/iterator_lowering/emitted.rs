// src/main.rs
use ::sifr_runtime::SifrInt;
#[expect(
    clippy::needless_pass_by_value,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn inc(x: SifrInt) -> SifrInt {
    ::std::ops::Add::add(&x, &SifrInt::from_i64(1))
}
fn main() {
    let nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
        SifrInt::from_i64(4),
    ];
    println!(
        "{:?}",
        Box::new(nums.iter().cloned().map(inc)).collect::<Vec<_>>()
    );
    println!(
        "{:?}",
        Box::new(nums.iter().cloned().rev()).collect::<Vec<_>>()
    );
    let list_comp: Vec<SifrInt> = {
        let mut sifr_generated_list_comp = Vec::new();
        for x in nums.iter().cloned() {
            sifr_generated_list_comp.push(x);
        }
        sifr_generated_list_comp
    };
    println!("{list_comp:?}");
    println!("{nums:?}");
}
