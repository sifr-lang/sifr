// src/main.rs
use ::sifr_runtime::SifrInt;
fn main() {
    let values: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
        SifrInt::from_i64(4),
    ];
    let mut it: Box<dyn Iterator<Item = SifrInt>> = Box::new(values.iter().cloned());
    let first: Option<SifrInt> = it.next();
    println!(
        "{}",
        first.map_or_else(
            || "None".to_string(),
            |sifr_generated_v| sifr_generated_v.to_string()
        )
    );
    let mut remaining_total: SifrInt = SifrInt::from_i64(0);
    for item in it {
        remaining_total = &remaining_total + &item;
    }
    println!("{remaining_total}");
    let mut pair_total: SifrInt = SifrInt::from_i64(0);
    for (i, value) in Box::new(
        values
            .iter()
            .cloned()
            .enumerate()
            .map(|sifr_generated_pair| {
                (
                    SifrInt::from(sifr_generated_pair.0) + SifrInt::from_i64(0),
                    sifr_generated_pair.1,
                )
            }),
    ) {
        pair_total = &(&pair_total + &i) + &value;
    }
    println!("{pair_total}");
}
