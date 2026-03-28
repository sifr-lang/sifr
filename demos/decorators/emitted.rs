// @log
fn greet(name: &String) -> String {
    return format!("{}{}{}", "Hello, ".to_string(), name, "!".to_string());
}

// @validate
// @log
fn process(x: i64) -> i64 {
    return x * (2 as i64);
}

fn sum_all(nums: &Vec<i64>) -> i64 {
    let mut total: i64 = 0 as i64;
    for n in nums.iter().copied() {
        total = total + n;
    }
    return total;
}

fn max_of(values: &Vec<i64>) -> i64 {
    let result: Option<i64> = (values).iter().copied().max();
    if let Some(result) = result {
        return result;
    }
    return 0 as i64;
}

fn join_strings(sep: &String, parts: &Vec<String>) -> String {
    let mut result: String = "".to_string();
    let mut i: i64 = 0 as i64;
    for p in parts.iter().cloned() {
        if i > (0 as i64) {
            result = format!("{}{}", result, sep);
        }
        result = format!("{}{}", result, p);
        i = i + (1 as i64);
    }
    return result;
}

fn main() {
    println!("{}", greet(&"World".to_string()));
    println!("{}", process(21 as i64));
    println!("{}", sum_all(&vec![1 as i64, 2 as i64, 3 as i64, 4 as i64, 5 as i64]));
    println!("{}", sum_all(&vec![10 as i64, 20 as i64]));
    println!("{}", max_of(&vec![3 as i64, 7 as i64, 2 as i64, 9 as i64, 1 as i64]));
    println!("{}", join_strings(&", ".to_string(), &vec!["Alice".to_string(), "Bob".to_string(), "Charlie".to_string()]));
}
