// src/main.rs
use ::sifr_runtime::SifrInt;
fn main() {
    let nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(2),
        SifrInt::from_i64(4),
        SifrInt::from_i64(6),
        SifrInt::from_i64(8),
    ];
    let doubled: Vec<SifrInt> = Box::new(
        nums.iter()
            .cloned()
            .map(|x| ::std::ops::Mul::mul(&x, &SifrInt::from_i64(2))),
    )
    .collect::<Vec<_>>();
    let evens: Vec<SifrInt> = Box::new(
        nums.iter()
            .filter(move |&sifr_generated_filter_item| {
                let x = sifr_generated_filter_item.clone();
                x.floor_mod_known_nonzero(&SifrInt::from_i64(4)) == SifrInt::from_i64(0)
            })
            .cloned(),
    )
    .collect::<Vec<_>>();
    let comp: Vec<SifrInt> = {
        let mut sifr_generated_list_comp = Vec::new();
        #[expect(
            clippy::explicit_iter_loop,
            reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
        )]
        for x in nums.iter() {
            sifr_generated_list_comp.push(::std::ops::Add::add(x, &SifrInt::from_i64(1)));
        }
        sifr_generated_list_comp
    };
    println!("{doubled:?}");
    println!("{evens:?}");
    println!("{comp:?}");
    println!("{}", SifrInt::from(nums.len()));
    #[expect(
        clippy::explicit_iter_loop,
        reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
    )]
    for n in nums.iter() {
        println!("{n}");
    }
    println!(
        "{:?}",
        Box::new(
            vec![
                SifrInt::from_i64(9),
                SifrInt::from_i64(10),
                SifrInt::from_i64(11)
            ]
            .into_iter()
            .map(|x| ::std::ops::Sub::sub(&x, &SifrInt::from_i64(1)))
        )
        .collect::<Vec<_>>()
    );
    println!("clone_cloned_iterators_comprehension_demo: pass");
}
