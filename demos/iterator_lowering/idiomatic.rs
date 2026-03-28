fn inc(x: i64) -> i64 {
    return x + (1 as i64);
}

fn main() {
    let nums: Vec<i64> = vec![1 as i64, 2 as i64, 3 as i64, 4 as i64];
    println!(
        "{:?}",
        Box::new(nums.iter().copied().map(inc)).collect::<Vec<_>>()
    );
    println!(
        "{:?}",
        Box::new((nums).iter().copied().rev()).collect::<Vec<_>>()
    );
    let list_comp: Vec<i64> = {
        let mut __sifr_list_comp = vec![];
        for x in nums.iter().copied() {
            __sifr_list_comp.push(x);
        }
        __sifr_list_comp
    };
    println!("{:?}", list_comp);
    println!("{:?}", nums.iter().copied().map(|x| x).collect::<Vec<_>>());
}
