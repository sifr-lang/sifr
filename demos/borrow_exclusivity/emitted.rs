// src/main.rs
use ::sifr_runtime::SifrInt;
use ::sifr_runtime::SifrRange;
fn get_length(items: &[SifrInt]) -> SifrInt {
    SifrInt::from(items.len())
}
fn get_sum(items: &[SifrInt]) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    #[expect(
        clippy::explicit_iter_loop,
        reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
    )]
    for item in items.iter() {
        total = ::std::ops::Add::add(&total, item);
    }
    total
}
#[expect(
    clippy::needless_pass_by_value,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn consume_and_reverse(items: Vec<SifrInt>) -> Vec<SifrInt> {
    Box::new(items.iter().cloned().rev()).collect::<Vec<_>>()
}
fn add_lengths(a: &[SifrInt], b: &[SifrInt]) -> SifrInt {
    ::std::ops::Add::add(&SifrInt::from(a.len()), &SifrInt::from(b.len()))
}
#[expect(
    clippy::needless_pass_by_value,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn double(x: SifrInt) -> SifrInt {
    ::std::ops::Mul::mul(&x, &SifrInt::from_i64(2))
}
fn negate(x: f64) -> f64 {
    -x
}
#[expect(
    clippy::approx_constant,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn main() {
    let data: Vec<SifrInt> = vec![
        SifrInt::from_i64(10),
        SifrInt::from_i64(20),
        SifrInt::from_i64(30),
        SifrInt::from_i64(40),
        SifrInt::from_i64(50),
    ];
    let length: SifrInt = get_length(&data);
    let total: SifrInt = get_sum(&data);
    println!("{length}");
    println!("{total}");
    println!("{data:?}");
    let items: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
    ];
    let result: Vec<SifrInt> = consume_and_reverse(items);
    println!("{result:?}");
    let nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
    ];
    let combined: SifrInt = add_lengths(&nums, &nums);
    println!("{combined}");
    println!("{nums:?}");
    let x: SifrInt = SifrInt::from_i64(42);
    let d: SifrInt = double(x);
    println!("{d}");
    println!("{x}");
    let pi: f64 = 3.14_f64;
    let neg: f64 = negate(pi);
    println!("{neg}");
    println!("{pi}");
    let loop_data: Vec<SifrInt> = vec![
        SifrInt::from_i64(5),
        SifrInt::from_i64(10),
        SifrInt::from_i64(15),
    ];
    let mut loop_total: SifrInt = SifrInt::from_i64(0);
    for _i in SifrRange::new_known_nonzero(
        SifrInt::from_i64(0),
        SifrInt::from_i64(3),
        SifrInt::from_i64(1),
    ) {
        loop_total = ::std::ops::Add::add(&loop_total, &get_sum(&loop_data));
    }
    println!("{loop_total}");
    println!("{loop_data:?}");
}
