// src/main.rs
use ::sifr_runtime::SifrInt;
#[expect(
    clippy::needless_pass_by_value,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn is_even(x: SifrInt) -> bool {
    x.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == SifrInt::from_i64(0)
}
fn main() {
    let nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
        SifrInt::from_i64(4),
    ];
    let evens: Box<dyn Iterator<Item = SifrInt>> = Box::new(
        nums.iter()
            .filter(|&sifr_generated_filter_item| {
                let sifr_generated_filter_value = sifr_generated_filter_item.clone();
                is_even(sifr_generated_filter_value)
            })
            .cloned(),
    );
    println!("{:?}", evens.collect::<Vec<_>>());
    let rev: Box<dyn Iterator<Item = SifrInt>> = Box::new(nums.iter().cloned().rev());
    println!("{:?}", rev.collect::<Vec<_>>());
    let indexed: Box<dyn Iterator<Item = (SifrInt, SifrInt)>> =
        Box::new(nums.iter().cloned().enumerate().map(|sifr_generated_pair| {
            (
                ::std::ops::Add::add(SifrInt::from(sifr_generated_pair.0), SifrInt::from_i64(10)),
                sifr_generated_pair.1,
            )
        }));
    println!("{:?}", indexed.collect::<Vec<_>>());
    println!("{}", Box::new(nums.iter().cloned()).sum::<SifrInt>());
    println!("{:?}", {
        let mut sifr_generated_sorted_values = Box::new(nums.iter().cloned()).collect::<Vec<_>>();
        let sifr_generated_sorted_reverse = true;
        sifr_generated_sorted_values.sort_by(
            |sifr_generated_sorted_left, sifr_generated_sorted_right| {
                if sifr_generated_sorted_reverse {
                    sifr_generated_sorted_right.cmp(sifr_generated_sorted_left)
                } else {
                    sifr_generated_sorted_left.cmp(sifr_generated_sorted_right)
                }
            },
        );
        sifr_generated_sorted_values
    });
    let collected: Vec<SifrInt> = Box::new(
        nums.iter()
            .filter(|&sifr_generated_filter_item| {
                let sifr_generated_filter_value = sifr_generated_filter_item.clone();
                is_even(sifr_generated_filter_value)
            })
            .cloned(),
    )
    .collect::<Vec<_>>();
    println!("{collected:?}");
}
