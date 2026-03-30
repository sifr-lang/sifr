fn get_length(items: &[i64]) -> usize {
    items.len()
}

fn get_sum(items: &[i64]) -> i64 {
    items.iter().sum()
}

fn consume_and_reverse(mut items: Vec<i64>) -> Vec<i64> {
    items.reverse();
    items
}

fn add_lengths(a: &[i64], b: &[i64]) -> usize {
    a.len() + b.len()
}

fn double(x: i64) -> i64 {
    x * 2
}

fn negate(x: f64) -> f64 {
    -x
}

fn main() {
    let data = vec![10, 20, 30, 40, 50];
    println!("{}", get_length(&data));
    println!("{}", get_sum(&data));
    println!("{data:?}");

    let items = vec![1, 2, 3];
    println!("{:?}", consume_and_reverse(items));

    let nums = vec![1, 2, 3];
    println!("{}", add_lengths(&nums, &nums));
    println!("{nums:?}");

    let x = 42;
    println!("{}", double(x));
    println!("{x}");

    let pi = 3.14;
    println!("{}", negate(pi));
    println!("{pi}");

    let loop_data = vec![5, 10, 15];
    let mut loop_total = 0;
    for _ in 0..3 {
        loop_total += get_sum(&loop_data);
    }
    println!("{loop_total}");
    println!("{loop_data:?}");
}
