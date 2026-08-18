// src/main.rs
fn main() {
    let nums: Vec<i64> = vec![2_i64, 4_i64, 6_i64, 8_i64];
    let doubled: Vec<i64> = Box::new(nums.iter().copied().map(|__sifr_map_item| (|x| x * (2_i64))(__sifr_map_item))).collect::<Vec<_>>();
    let evens: Vec<i64> = Box::new(nums.iter().copied().filter(|__filter_item| {
    let __filter_value = *__filter_item;
    {
    let x = __filter_value;
    (x % (4_i64)) == (0_i64)
}
})).collect::<Vec<_>>();
    let comp: Vec<i64> = {
    let mut __sifr_list_comp = vec![];
    for x in nums.iter().copied() {
        __sifr_list_comp.push(x + (1_i64));
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
    println!("{:?}", Box::new(vec![9_i64, 10_i64, 11_i64].into_iter().map(|__sifr_map_item| (|x| x - (1_i64))(__sifr_map_item))).collect::<Vec<_>>());
    println!("clone_cloned_iterators_comprehension_demo: pass");
}
