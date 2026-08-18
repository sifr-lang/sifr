// src/main.rs
fn tail_first(values: &Vec<i64>) -> i64 {
    let mut rev: Box<dyn Iterator<Item = i64>> = Box::new((values).iter().copied().rev());
    let first: Option<i64> = rev.next();
    let Some(first) = first else {
        return 0_i64;
    };
    first
}

fn main() {
    let nums: Vec<i64> = vec![10_i64, 20_i64, 30_i64];
    println!("{}", tail_first(&(nums).iter().copied().collect::<Vec<_>>()));
    let tup: (i64, i64, i64) = (4_i64, 5_i64, 6_i64);
    let mut total: i64 = 0_i64;
    for item in {
    let __sifr_tuple_iter_src = tup;
    vec![__sifr_tuple_iter_src.0, __sifr_tuple_iter_src.1, __sifr_tuple_iter_src.2].into_iter()
} {
        total += item;
    }
    println!("{}", total);
    let rev_tup: Box<dyn Iterator<Item = i64>> = Box::new(({
    let __sifr_tuple_iter_src = (tup).clone();
    vec![__sifr_tuple_iter_src.0.clone(), __sifr_tuple_iter_src.1.clone(), __sifr_tuple_iter_src.2.clone()].into_iter()
}).rev());
    println!("{:?}", rev_tup.collect::<Vec<_>>());
}
