// src/main.rs
// @log
fn greet(name: &String) -> String {
    {
    let mut __sifr_concat: String = String::with_capacity((7usize + name.len()) + 1usize);
    __sifr_concat.push_str("Hello, ");
    __sifr_concat.push_str((name).as_str());
    __sifr_concat.push('!');
    __sifr_concat
}
}

// @validate
// @log
fn process(x: i64) -> i64 {
    x * (2_i64)
}

fn sum_all(nums: &Vec<i64>) -> i64 {
    let mut total: i64 = 0_i64;
    for n in nums.iter().copied() {
        total += n;
    }
    total
}

fn max_of(values: &Vec<i64>) -> i64 {
    let result: Option<i64> = (values).iter().copied().max();
    if let Some(result) = result {
        return result;
    }
    0_i64
}

fn join_strings(sep: &String, parts: &Vec<String>) -> String {
    let mut result: String = "".to_string();
    let mut i: i64 = 0_i64;
    for p in parts.iter().cloned() {
        if i > (0_i64) {
            result.push_str((sep).as_str());
        }
        result.push_str((p).as_str());
        i += 1_i64;
    }
    result
}

fn main() {
    println!("{}", greet(&"World".to_string()));
    println!("{}", process(21_i64));
    println!("{}", sum_all(&vec![1_i64, 2_i64, 3_i64, 4_i64, 5_i64]));
    println!("{}", sum_all(&vec![10_i64, 20_i64]));
    println!("{}", max_of(&vec![3_i64, 7_i64, 2_i64, 9_i64, 1_i64]));
    println!("{}", join_strings(&", ".to_string(), &vec!["Alice".to_string(), "Bob".to_string(), "Charlie".to_string()]));
}
