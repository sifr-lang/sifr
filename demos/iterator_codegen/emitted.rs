// src/main.rs
use ::sifr_runtime::SifrInt;
fn greater_than_two(x: SifrInt) -> bool {
    &x > &SifrInt::from_i64(2)
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
        Box::new(nums.iter().cloned().filter(|sifr_generated_filter_item| {
            let sifr_generated_filter_value = sifr_generated_filter_item.clone();
            greater_than_two(sifr_generated_filter_value)
        }))
        .collect::<Vec<_>>()
    );
    println!("{:?}", {
        let mut sifr_generated_sorted_v = Box::new(nums.iter().cloned()).collect::<Vec<_>>();
        sifr_generated_sorted_v.sort();
        sifr_generated_sorted_v
    });
}
