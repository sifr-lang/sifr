fn get_length(items: &Vec<i64>) -> i64 {
    return items.len() as i64;
}

fn get_sum(items: &Vec<i64>) -> i64 {
    let mut total: i64 = 0 as i64;
    for item in items.iter().copied() {
        total = total + item;
    }
    return total;
}

fn consume_and_reverse(items: Vec<i64>) -> Vec<i64> {
    return Box::new((items).iter().copied().rev()).collect::<Vec<_>>();
}

fn add_lengths(a: &Vec<i64>, b: &Vec<i64>) -> i64 {
    return (a.len() as i64) + (b.len() as i64);
}

fn double(x: i64) -> i64 {
    return x * (2 as i64);
}

fn negate(x: f64) -> f64 {
    return -x;
}

fn main() {
    let data: Vec<i64> = vec![10 as i64, 20 as i64, 30 as i64, 40 as i64, 50 as i64];
    let length: i64 = get_length(&data);
    let total: i64 = get_sum(&data);
    println!("{}", length);
    println!("{}", total);
    println!("{:?}", data);
    let items: Vec<i64> = vec![1 as i64, 2 as i64, 3 as i64];
    let result: Vec<i64> = consume_and_reverse(items);
    println!("{:?}", result);
    let nums: Vec<i64> = vec![1 as i64, 2 as i64, 3 as i64];
    let combined: i64 = add_lengths(&nums, &nums);
    println!("{}", combined);
    println!("{:?}", nums);
    let x: i64 = 42 as i64;
    let d: i64 = double(x);
    println!("{}", d);
    println!("{}", x);
    let pi: f64 = 3.14 as f64;
    let neg: f64 = negate(pi);
    println!("{}", neg);
    println!("{}", pi);
    let loop_data: Vec<i64> = vec![5 as i64, 10 as i64, 15 as i64];
    let mut loop_total: i64 = 0 as i64;
    for i in 0 as i64..3 as i64 {
        loop_total = loop_total + get_sum(&loop_data);
    }
    println!("{}", loop_total);
    println!("{:?}", loop_data);
}
