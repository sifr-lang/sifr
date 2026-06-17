fn main() {
    let nums: Vec<i64> = vec![2 as i64, 4 as i64, 6 as i64, 8 as i64];
    let doubled: Vec<i64> = Box::new(nums.iter().copied().map(|x| x * (2 as i64))).collect::<Vec<_>>();
    let evens: Vec<i64> = Box::new(nums.iter().copied().filter(|__filter_item| {
    let __filter_value = *__filter_item;
    return {
    let x = __filter_value;
    (x % (4 as i64)) == (0 as i64)
};
})).collect::<Vec<_>>();
    let comp: Vec<i64> = {
    let mut __sifr_list_comp = vec![];
    for x in nums.iter().copied() {
        __sifr_list_comp.push(x + (1 as i64));
    }
    __sifr_list_comp
};
    println!("{:?}", doubled);
    println!("{:?}", evens);
    println!("{:?}", comp);
    println!("{}", nums.len() as i64);
    for n in nums.iter().copied() {
        println!("{}", n);
    }
    println!("{:?}", Box::new(vec![9 as i64, 10 as i64, 11 as i64].into_iter().map(|x| x - (1 as i64))).collect::<Vec<_>>());
    println!("clone_cloned_iterators_comprehension_demo: pass");
}
