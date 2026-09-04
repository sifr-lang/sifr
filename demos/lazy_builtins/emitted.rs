// src/main.rs
use ::sifr_runtime::SifrInt;
fn main() {
    let nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(3),
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
    ];
    let mut rev_it: Box<dyn Iterator<Item = SifrInt>> = Box::new(nums.iter().cloned().rev());
    println!(
        "{}",
        rev_it.next().map_or_else(
            || "None".to_string(),
            |sifr_generated_v| sifr_generated_v.to_string()
        )
    );
    println!("{:?}", rev_it.collect::<Vec<_>>());
    let indexed_it: Box<dyn Iterator<Item = (SifrInt, String)>> = Box::new(
        vec!["a".to_string(), "b".to_string()]
            .into_iter()
            .enumerate()
            .map(|sifr_generated_pair| {
                (
                    ::std::ops::Add::add(
                        SifrInt::from(sifr_generated_pair.0),
                        SifrInt::from_i64(5),
                    ),
                    sifr_generated_pair.1,
                )
            }),
    );
    println!("{:?}", indexed_it.collect::<Vec<_>>());
    let zipped_it: Box<dyn Iterator<Item = (SifrInt, String, bool)>> = Box::new(
        vec![SifrInt::from_i64(1), SifrInt::from_i64(2)]
            .into_iter()
            .zip(vec!["x".to_string(), "y".to_string()])
            .zip(vec![true, false])
            .map(|sifr_generated_zip_item| {
                (
                    sifr_generated_zip_item.0.0,
                    sifr_generated_zip_item.0.1,
                    sifr_generated_zip_item.1,
                )
            }),
    );
    println!("{:?}", zipped_it.collect::<Vec<_>>());
}
