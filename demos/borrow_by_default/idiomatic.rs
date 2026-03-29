fn get_length(items: &[i64]) -> i64 {
    items.len() as i64
}

fn get_first_char(text: &str) -> String {
    text.chars()
        .next()
        .map(|ch| ch.to_string())
        .unwrap_or_default()
}

fn consume_and_count(items: Vec<i64>) -> i64 {
    items.len() as i64
}

fn add(x: i64, y: i64) -> i64 {
    x + y
}

fn is_positive(n: f64) -> bool {
    n > 0.0
}

fn process_data(data: &[i64]) -> i64 {
    data.iter().copied().sum()
}

fn sum_multiple_times(items: &[i64], times: i64) -> i64 {
    (0..times).map(|_| get_length(items)).sum()
}

fn apply_and_return(f: impl Fn(&[i64]) -> i64, items: &[i64]) -> i64 {
    f(items)
}

fn compute_sum(nums: &[i64]) -> i64 {
    nums.iter().copied().sum()
}

fn main() {
    let my_list = vec![10_i64, 20, 30];
    println!("{}", get_length(&my_list));
    println!("{my_list:?}");

    let greeting = "Hello, Sifr!".to_string();
    println!("{}", get_first_char(&greeting));
    println!("{greeting}");

    let owned_list = vec![1_i64, 2, 3, 4, 5];
    println!("{}", consume_and_count(owned_list));

    println!("{}", add(10, 20));

    let pi = 3.14_f64;
    println!("{}", is_positive(pi));
    println!("{pi}");

    let data = vec![1_i64, 2, 3, 4, 5];
    println!("{}", process_data(&data));
    println!("{data:?}");

    let items = vec![10_i64, 20, 30];
    println!("{}", sum_multiple_times(&items, 3));
    println!("{items:?}");

    let nums = vec![5_i64, 10, 15];
    println!("{}", apply_and_return(compute_sum, &nums));
    println!("{nums:?}");
}
