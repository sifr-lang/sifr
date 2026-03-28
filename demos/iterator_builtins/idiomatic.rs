fn is_even(x: i64) -> bool {
    return (x % (2 as i64)) == (0 as i64);
}

fn main() {
    let nums: Vec<i64> = vec![1 as i64, 2 as i64, 3 as i64, 4 as i64];
    let mut evens: Box<dyn Iterator<Item = i64>> =
        Box::new(nums.iter().copied().filter(|__filter_item| {
            let __filter_value = *__filter_item;
            return is_even(__filter_value);
        }));
    println!("{:?}", evens.collect::<Vec<_>>());
    let mut rev: Box<dyn Iterator<Item = i64>> = Box::new((nums).iter().copied().rev());
    println!("{:?}", rev.collect::<Vec<_>>());
    let mut indexed: Box<dyn Iterator<Item = (i64, i64)>> = Box::new(
        (nums)
            .iter()
            .copied()
            .enumerate()
            .map(|__pair| ((__pair.0 as i64) + (10 as i64), __pair.1)),
    );
    println!("{:?}", indexed.collect::<Vec<_>>());
    println!("{}", Box::new((nums).iter().copied()).sum::<i64>());
    println!("{:?}", {
        let mut __sifr_sorted_v = Box::new((nums).iter().copied()).collect::<Vec<_>>();
        __sifr_sorted_v.sort();
        if true {
            {
                __sifr_sorted_v.reverse();
            }
        };
        __sifr_sorted_v
    });
    let collected: Vec<i64> = Box::new(nums.iter().copied().filter(|__filter_item| {
        let __filter_value = *__filter_item;
        return is_even(__filter_value);
    }))
    .collect::<Vec<_>>();
    println!("{:?}", collected);
}
