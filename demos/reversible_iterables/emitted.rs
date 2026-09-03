// src/main.rs
use ::sifr_runtime::SifrInt;
fn tail_first(values: &[SifrInt]) -> SifrInt {
    let mut rev: Box<dyn Iterator<Item = SifrInt>> = Box::new(values.iter().cloned().rev());
    let first: Option<SifrInt> = rev.next();
    let Some(first_value_89d7ed7f996f1d41) = first.clone() else {
        return SifrInt::from_i64(0);
    };
    first_value_89d7ed7f996f1d41
}
fn main() {
    let nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(10),
        SifrInt::from_i64(20),
        SifrInt::from_i64(30),
    ];
    println!("{}", tail_first(&nums.iter().cloned().collect::<Vec<_>>()));
    let tup: (SifrInt, SifrInt, SifrInt) = (
        SifrInt::from_i64(4),
        SifrInt::from_i64(5),
        SifrInt::from_i64(6),
    );
    let mut total: SifrInt = SifrInt::from_i64(0);
    for item in {
        let sifr_generated_tuple_iter_src = tup.clone();
        vec![
            sifr_generated_tuple_iter_src.0.clone(),
            sifr_generated_tuple_iter_src.1.clone(),
            sifr_generated_tuple_iter_src.2.clone(),
        ]
        .into_iter()
    } {
        total = &total + &item;
    }
    println!("{total}");
    let rev_tup: Box<dyn Iterator<Item = SifrInt>> = Box::new(
        {
            let sifr_generated_tuple_iter_src = tup.clone();
            vec![
                sifr_generated_tuple_iter_src.0.clone(),
                sifr_generated_tuple_iter_src.1.clone(),
                sifr_generated_tuple_iter_src.2.clone(),
            ]
            .into_iter()
        }
        .rev(),
    );
    println!("{:?}", rev_tup.collect::<Vec<_>>());
}
