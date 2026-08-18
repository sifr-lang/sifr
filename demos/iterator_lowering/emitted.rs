// src/main.rs
fn inc(x: i64) -> i64 {
    x + (1_i64)
}

fn main() {
    let nums: Vec<i64> = vec![1_i64, 2_i64, 3_i64, 4_i64];
    println!("{:?}", Box::new(nums.iter().copied().map(|__sifr_map_item| inc(__sifr_map_item))).collect::<Vec<_>>());
    println!("{:?}", Box::new((nums).iter().copied().rev()).collect::<Vec<_>>());
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
