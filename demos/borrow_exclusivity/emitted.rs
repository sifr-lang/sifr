// src/main.rs
fn get_length(items: &Vec<i64>) -> i64 {
    items.len() as i64
}

fn get_sum(items: &Vec<i64>) -> i64 {
    let mut total: i64 = 0_i64;
    for item in items.iter().copied() {
        total += item;
    }
    total
}

fn consume_and_reverse(items: Vec<i64>) -> Vec<i64> {
    Box::new((items).iter().copied().rev()).collect::<Vec<_>>()
}

fn add_lengths(a: &Vec<i64>, b: &Vec<i64>) -> i64 {
    (a.len() as i64) + (b.len() as i64)
}

fn double(x: i64) -> i64 {
    x * (2_i64)
}

fn negate(x: f64) -> f64 {
    -x
}

fn main() {
    let data: Vec<i64> = vec![10_i64, 20_i64, 30_i64, 40_i64, 50_i64];
    let length: i64 = get_length(&data);
    let total: i64 = get_sum(&data);
    println!("{}", length);
    println!("{}", total);
    println!("{:?}", data);
    let items: Vec<i64> = vec![1_i64, 2_i64, 3_i64];
    let result: Vec<i64> = consume_and_reverse(items);
    println!("{:?}", result);
    let nums: Vec<i64> = vec![1_i64, 2_i64, 3_i64];
    let combined: i64 = add_lengths(&nums, &nums);
    println!("{}", combined);
    println!("{:?}", nums);
    let x: i64 = 42_i64;
    let d: i64 = double(x);
    println!("{}", d);
    println!("{}", x);
    let pi: f64 = 3.14_f64;
    let neg: f64 = negate(pi);
    println!("{}", neg);
    println!("{}", pi);
    let loop_data: Vec<i64> = vec![5_i64, 10_i64, 15_i64];
    let mut loop_total: i64 = 0_i64;
    for i in 0_i64..3_i64 {
        loop_total += get_sum(&loop_data);
    }
    println!("{}", loop_total);
    println!("{:?}", loop_data);
}
