// src/main.rs
fn get_length(items: &Vec<i64>) -> i64 {
    items.len() as i64
}

fn get_first_char(s: &String) -> String {
    let __sifr_chars_s: Vec<char> = s.chars().collect::<Vec<char>>();
    let result: Option<String> = __sifr_chars_s.get((0_i64) as usize).map(|c| c.to_string());
    if let Some(result) = result {
        return result;
    }
    "".to_string()
}

fn consume_and_count(items: Vec<i64>) -> i64 {
    items.len() as i64
}

fn add(x: i64, y: i64) -> i64 {
    x + y
}

fn is_positive(n: f64) -> bool {
    n > (0.0_f64)
}

fn process_data(data: &Vec<i64>) -> i64 {
    let mut total: i64 = 0_i64;
    for item in data.iter().copied() {
        total += item;
    }
    total
}

fn sum_multiple_times(items: &Vec<i64>, times: i64) -> i64 {
    let mut total: i64 = 0_i64;
    for i in 0_i64..times {
        total += get_length(items);
    }
    total
}

fn apply_and_return(f: impl Fn(&Vec<i64>) -> i64, items: &Vec<i64>) -> i64 {
    f(items)
}

fn compute_sum(nums: &Vec<i64>) -> i64 {
    let mut total: i64 = 0_i64;
    for n in nums.iter().copied() {
        total += n;
    }
    total
}

fn main() {
    let my_list: Vec<i64> = vec![10_i64, 20_i64, 30_i64];
    let length: i64 = get_length(&my_list);
    println!("{}", length);
    println!("{:?}", my_list);
    let greeting: String = "Hello, Sifr!".to_string();
    let first: String = get_first_char(&greeting);
    println!("{}", first);
    println!("{}", greeting);
    let owned_list: Vec<i64> = vec![1_i64, 2_i64, 3_i64, 4_i64, 5_i64];
    let count: i64 = consume_and_count(owned_list);
    println!("{}", count);
    let result: i64 = add(10_i64, 20_i64);
    println!("{}", result);
    let pi: f64 = 3.14_f64;
    println!("{}", is_positive(pi));
    println!("{}", pi);
    let data: Vec<i64> = vec![1_i64, 2_i64, 3_i64, 4_i64, 5_i64];
    let total: i64 = process_data(&data);
    println!("{}", total);
    println!("{:?}", data);
    let items: Vec<i64> = vec![10_i64, 20_i64, 30_i64];
    let loop_total: i64 = sum_multiple_times(&items, 3_i64);
    println!("{}", loop_total);
    println!("{:?}", items);
    let nums: Vec<i64> = vec![5_i64, 10_i64, 15_i64];
    let sum_result: i64 = apply_and_return(compute_sum, &nums);
    println!("{}", sum_result);
    println!("{:?}", nums);
}
