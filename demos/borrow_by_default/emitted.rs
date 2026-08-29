// src/main.rs
use ::sifr_runtime::SifrInt;

use ::sifr_runtime::SifrRange;

fn get_length(items: &Vec<SifrInt>) -> SifrInt {
    SifrInt::from(items.len())
}

fn get_first_char(s: &String) -> String {
    let __sifr_chars_s: Vec<char> = s.chars().collect::<Vec<char>>();
    let result: Option<String> = __sifr_chars_s.get(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(0)))).map(|c| c.to_string());
    if let Some(result) = result {
        return result;
    }
    "".to_string()
}

fn consume_and_count(items: Vec<SifrInt>) -> SifrInt {
    SifrInt::from(items.len())
}

fn add(x: SifrInt, y: SifrInt) -> SifrInt {
    &x + &y
}

fn is_positive(n: f64) -> bool {
    n > (0.0_f64)
}

fn process_data(data: &Vec<SifrInt>) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    for item in data.iter().cloned() {
        total = &total + &item;
    }
    total.clone()
}

fn sum_multiple_times(items: &Vec<SifrInt>, times: SifrInt) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    for i in SifrRange::new_known_nonzero(SifrInt::from_i64(0), times.clone(), SifrInt::from_i64(1)) {
        total = &total + &get_length(items);
    }
    total.clone()
}

fn apply_and_return(f: impl Fn(&Vec<SifrInt>) -> SifrInt, items: &Vec<SifrInt>) -> SifrInt {
    f(items)
}

fn compute_sum(nums: &Vec<SifrInt>) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    for n in nums.iter().cloned() {
        total = &total + &n;
    }
    total.clone()
}

fn main() {
    let my_list: Vec<SifrInt> = vec![SifrInt::from_i64(10), SifrInt::from_i64(20), SifrInt::from_i64(30)];
    let length: SifrInt = get_length(&my_list);
    println!("{}", length);
    println!("{:?}", my_list);
    let greeting: String = "Hello, Sifr!".to_string();
    let first: String = get_first_char(&greeting);
    println!("{}", first);
    println!("{}", greeting);
    let owned_list: Vec<SifrInt> = vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3), SifrInt::from_i64(4), SifrInt::from_i64(5)];
    let count: SifrInt = consume_and_count(owned_list);
    println!("{}", count);
    let result: SifrInt = add(SifrInt::from_i64(10), SifrInt::from_i64(20));
    println!("{}", result);
    let pi: f64 = 3.14_f64;
    println!("{}", is_positive(pi));
    println!("{}", pi);
    let data: Vec<SifrInt> = vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3), SifrInt::from_i64(4), SifrInt::from_i64(5)];
    let total: SifrInt = process_data(&data);
    println!("{}", total);
    println!("{:?}", data);
    let items: Vec<SifrInt> = vec![SifrInt::from_i64(10), SifrInt::from_i64(20), SifrInt::from_i64(30)];
    let loop_total: SifrInt = sum_multiple_times(&items, SifrInt::from_i64(3));
    println!("{}", loop_total);
    println!("{:?}", items);
    let nums: Vec<SifrInt> = vec![SifrInt::from_i64(5), SifrInt::from_i64(10), SifrInt::from_i64(15)];
    let sum_result: SifrInt = apply_and_return(compute_sum, &nums);
    println!("{}", sum_result);
    println!("{:?}", nums);
}
