fn greater_than_two(x: i64) -> bool {
    return x > (2 as i64);
}

fn main() {
    let nums: Vec<i64> = vec![5 as i64, 1 as i64, 3 as i64, 4 as i64];
    let flags: Vec<bool> = vec![false, true, false];
    println!("{}", Box::new((flags).iter().copied()).any(|x| x));
    println!(
        "{:?}",
        Box::new(nums.iter().copied().filter(|__filter_item| {
            let __filter_value = *__filter_item;
            return greater_than_two(__filter_value);
        }))
        .collect::<Vec<_>>()
    );
    println!("{:?}", {
        let mut __sifr_sorted_v = Box::new((nums).iter().copied()).collect::<Vec<_>>();
        __sifr_sorted_v.sort();
        __sifr_sorted_v
    });
}
