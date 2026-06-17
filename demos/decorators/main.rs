// Reference: decorators
// Reference: decorators
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.17s
     Running `target/debug/sifr emit demos/decorators_demo.sifr`
// @log
fn greet(name: String) -> String {
    return format!("{}{}{}", "Hello, ".to_string(), name, "!".to_string());
}

// @validate
// @log
fn process(x: i64) -> i64 {
    return x * 2_i64;
}

fn sum_all(nums: Vec<i64>) -> i64 {
    let mut total: i64 = 0_i64;
    for n in nums.iter().cloned() {
        total = total + n;
    }
    return total;
}

fn max_of(values: Vec<i64>) -> i64 {
    return values.iter().max().unwrap().clone();
}

fn join_strings(sep: String, parts: Vec<String>) -> String {
    let mut result: String = "".to_string();
    let mut i: i64 = 0_i64;
    for p in parts.iter().cloned() {
        if i > 0_i64 {
            result = format!("{}{}", result, sep);
        }
        result = format!("{}{}", result, p);
        i = i + 1_i64;
    }
    return result;
}

fn main() {
    println!("{}", greet("World".to_string()));
    println!("{}", process(21_i64));
    println!("{}", sum_all(vec![1_i64, 2_i64, 3_i64, 4_i64, 5_i64]));
    println!("{}", sum_all(vec![10_i64, 20_i64]));
    println!("{}", max_of(vec![3_i64, 7_i64, 2_i64, 9_i64, 1_i64]));
    println!("{}", join_strings(", ".to_string(), vec!["Alice".to_string(), "Bob".to_string(), "Charlie".to_string()]));
}
