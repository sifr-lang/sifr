fn main() {
    let nums = vec![2_i64, 4, 6, 8];

    let doubled: Vec<i64> = nums.iter().map(|x| x * 2).collect();
    let evens: Vec<i64> = nums.iter().copied().filter(|x| x % 4 == 0).collect();
    let comp: Vec<i64> = nums.iter().map(|x| x + 1).collect();

    println!("{doubled:?}");
    println!("{evens:?}");
    println!("{comp:?}");

    println!("{}", nums.len());
    for n in &nums {
        println!("{n}");
    }

    let temporary: Vec<i64> = [9_i64, 10, 11].into_iter().map(|x| x - 1).collect();
    println!("{temporary:?}");

    println!("clone_cloned_iterators_comprehension_demo: pass");
}
