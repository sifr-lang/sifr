fn inc(value: i64) -> i64 {
    value + 1
}

fn main() {
    let nums = [1_i64, 2, 3, 4];

    println!("{:?}", nums.into_iter().map(inc).collect::<Vec<_>>());
    println!("{:?}", nums.into_iter().rev().collect::<Vec<_>>());

    let list_comp: Vec<i64> = nums.into_iter().collect();
    println!("{list_comp:?}");

    let generator_expr = nums.into_iter();
    println!("{:?}", generator_expr.collect::<Vec<_>>());
}
