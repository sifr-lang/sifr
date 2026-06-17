// Reference: control_flow
// Reference: compiler-feature-history
use std::collections::HashMap;

fn sum_range(n: i64) -> i64 {
    let mut total: i64 = 0_i64;
    for i in 0_i64..n {
        total = total + i;
    }
    return total;
}

fn fizzbuzz(n: i64) {
    for i in 1_i64..n + 1_i64 {
        if i % 15_i64 == 0_i64 {
            println!("{}", "FizzBuzz");
        }
        if i % 3_i64 == 0_i64 {
            if i % 5_i64 != 0_i64 {
                println!("{}", "Fizz");
            }
        }
        if i % 5_i64 == 0_i64 {
            if i % 3_i64 != 0_i64 {
                println!("{}", "Buzz");
            }
        }
        if i % 3_i64 != 0_i64 {
            if i % 5_i64 != 0_i64 {
                println!("{}", i);
            }
        }
    }
}

fn countdown(n: i64) {
    let mut i: i64 = n;
    while i > 0_i64 {
        println!("{}", i);
        i = i - 1_i64;
    }
    println!("{}", "Go!");
}

fn main() {
    println!("=== While Loop: Countdown ===");
    countdown(5_i64);
    println!("=== For Loop: Sum of 0..9 ===");
    let s: i64 = sum_range(10_i64);
    println!("Sum of range(10) = {}", s);
    println!("=== Nested Loops: Multiplication Table ===");
    for i in 1_i64..4_i64 {
        for j in 1_i64..4_i64 {
            let product: i64 = i * j;
            println!("{} x {} = {}", i, j, product);
        }
    }
    println!("=== Break/Continue ===");
    let mut i: i64 = 0_i64;
    while i < 10_i64 {
        i = i + 1_i64;
        if i == 3_i64 {
            continue;
        }
        if i == 7_i64 {
            break;
        }
        println!("{}", i);
    }
    println!("=== Lists ===");
    let nums: Vec<i64> = vec![10_i64, 20_i64, 30_i64, 40_i64, 50_i64];
    println!("Length: {}", nums.len() as i64);
    println!("First: {}", ({ let _v = &nums; let _i = 0_i64; let _idx = if _i < 0 { (_v.len() as i64 + _i) as usize } else { _i as usize }; _v.get(_idx).cloned() }).map_or("None".to_string(), |_v| format!("{}", _v)));
    println!("Last: {}", ({ let _v = &nums; let _i = 4_i64; let _idx = if _i < 0 { (_v.len() as i64 + _i) as usize } else { _i as usize }; _v.get(_idx).cloned() }).map_or("None".to_string(), |_v| format!("{}", _v)));
    let mut total: i64 = 0_i64;
    for n in nums.iter().cloned() {
        total = total + n;
    }
    println!("Sum: {}", total);
    let mut fruits: Vec<String> = vec!["apple".to_string(), "banana".to_string()];
    fruits.push("cherry".to_string());
    println!("Fruits count: {}", fruits.len() as i64);
    println!("=== Dict ===");
    let ages: HashMap<String, i64> = HashMap::from([("Alice".to_string(), 30_i64), ("Bob".to_string(), 25_i64), ("Charlie".to_string(), 35_i64)]);
    println!("Alice is {} years old", (ages.get("Alice").cloned()).map_or("None".to_string(), |_v| format!("{}", _v)));
    println!("Bob is {} years old", (ages.get("Bob").cloned()).map_or("None".to_string(), |_v| format!("{}", _v)));
    println!("=== In Operator ===");
    let numbers: Vec<i64> = vec![1_i64, 2_i64, 3_i64, 4_i64, 5_i64];
    let found: bool = numbers.contains(&3_i64);
    println!("3 in list: {}", found);
    let missing: bool = numbers.contains(&9_i64);
    println!("9 in list: {}", missing);
    println!("=== Tuples ===");
    let point: (i64, i64, String) = (10_i64, 20_i64, "origin".to_string());
    println!("Tuple length: {}", 3_i64);
    println!("=== Tuple Unpacking ===");
    let pair: (String, i64) = ("Sifr".to_string(), 2025_i64);
    let (name, year) = pair;
    println!("{} was born in {}", name, year);
    println!("=== F-Strings ===");
    let a: i64 = 7_i64;
    let b: i64 = 8_i64;
    println!("{} * {} = {}", a, b, a * b);
    println!("Is {} > {}? {}", a, b, a > b);
    println!("=== String Operations ===");
    let greeting: String = "  Hello, World!  ".to_string();
    println!("{}", greeting.trim().to_string());
    println!("{}", greeting.trim().to_string().to_uppercase());
    println!("{}", greeting.trim().to_string().to_lowercase());
    let lang: String = "sifr-lang".to_string();
    println!("Starts with 'sifr': {}", lang.starts_with("sifr"));
    println!("Ends with 'lang': {}", lang.ends_with("lang"));
    println!("=== FizzBuzz (1-15) ===");
    fizzbuzz(15_i64);
}
