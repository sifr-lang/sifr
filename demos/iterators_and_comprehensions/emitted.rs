// src/main.rs
use ::sifr_runtime::SifrInt;
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn main() {
    let nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
        SifrInt::from_i64(4),
        SifrInt::from_i64(5),
    ];
    let doubled: Vec<SifrInt> = Box::new(
        nums.iter()
            .map(|x| ::std::ops::Mul::mul(x, &SifrInt::from_i64(2))),
    )
    .collect::<Vec<_>>();
    println!("{doubled:?}");
    let evens: Vec<SifrInt> = Box::new(
        nums.iter()
            .filter(move |&sifr_generated_filter_item| {
                let x = sifr_generated_filter_item.clone();
                x.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == SifrInt::from_i64(0)
            })
            .cloned(),
    )
    .collect::<Vec<_>>();
    println!("{evens:?}");
    let squares: Vec<SifrInt> = {
        let mut sifr_generated_list_comp = Vec::new();
        #[expect(
            clippy::explicit_iter_loop,
            reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
        )]
        for x in nums.iter() {
            sifr_generated_list_comp.push(::std::ops::Mul::mul(x, x));
        }
        sifr_generated_list_comp
    };
    println!("{squares:?}");
    let big_squares: Vec<SifrInt> = {
        let mut sifr_generated_list_comp = Vec::new();
        #[expect(
            clippy::explicit_iter_loop,
            reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
        )]
        for x in nums.iter() {
            if x > &SifrInt::from_i64(2) {
                sifr_generated_list_comp.push(::std::ops::Mul::mul(x, x));
            }
        }
        sifr_generated_list_comp
    };
    println!("{big_squares:?}");
    let lo: Option<SifrInt> = nums.iter().cloned().min();
    let hi: Option<SifrInt> = nums.iter().cloned().max();
    if let Some(lo) = lo {
        println!("{lo}");
    }
    if let Some(hi) = hi {
        println!("{hi}");
    }
    println!("{}", nums.iter().cloned().sum::<SifrInt>());
    let unsorted: Vec<SifrInt> = vec![
        SifrInt::from_i64(5),
        SifrInt::from_i64(3),
        SifrInt::from_i64(1),
        SifrInt::from_i64(4),
        SifrInt::from_i64(2),
    ];
    println!("{:?}", {
        let mut sifr_generated_sorted_values = unsorted.clone();
        sifr_generated_sorted_values.sort_by(
            |sifr_generated_sorted_left, sifr_generated_sorted_right| {
                sifr_generated_sorted_left.cmp(sifr_generated_sorted_right)
            },
        );
        sifr_generated_sorted_values
    });
    println!(
        "{:?}",
        Box::new(unsorted.iter().cloned().rev()).collect::<Vec<_>>()
    );
    let letters: Vec<String> = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    println!(
        "{:?}",
        Box::new(
            letters
                .iter()
                .cloned()
                .enumerate()
                .map(|sifr_generated_pair| (
                    ::std::ops::Add::add(
                        SifrInt::from(sifr_generated_pair.0),
                        SifrInt::from_i64(0)
                    ),
                    sifr_generated_pair.1
                ))
        )
        .collect::<Vec<_>>()
    );
    let names: Vec<String> = vec!["Alice".to_string(), "Bob".to_string()];
    let ages: Vec<SifrInt> = vec![SifrInt::from_i64(30), SifrInt::from_i64(25)];
    println!(
        "{:?}",
        Box::new(
            names
                .iter()
                .cloned()
                .zip(ages.iter().cloned())
                .map(|sifr_generated_zip_item| (
                    sifr_generated_zip_item.0,
                    sifr_generated_zip_item.1
                ))
        )
        .collect::<Vec<_>>()
    );
    let bools: Vec<bool> = vec![true, false, true];
    println!("{}", bools.iter().copied().any(|x| x));
    println!("{}", bools.iter().copied().all(|x| x));
    println!("{}", vec![true, true, true].into_iter().all(|x| x));
}
