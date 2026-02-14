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
            println!("{}", "FizzBuzz".to_string());
        }
        if i % 3_i64 == 0_i64 {
            if i % 5_i64 != 0_i64 {
                println!("{}", "Fizz".to_string());
            }
        }
        if i % 5_i64 == 0_i64 {
            if i % 3_i64 != 0_i64 {
                println!("{}", "Buzz".to_string());
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
    println!("{}", "Go!".to_string());
}

fn main() {
    println!("{}", format!("=== While Loop: Countdown ==="));
    countdown(5_i64);
    println!("{}", format!("=== For Loop: Sum of 0..9 ==="));
    let mut s: i64 = sum_range(10_i64);
    println!("{}", format!("Sum of range(10) = {}", s));
    println!("{}", format!("=== Nested Loops: Multiplication Table ==="));
    for i in 1_i64..4_i64 {
        for j in 1_i64..4_i64 {
            let mut product: i64 = i * j;
            println!("{}", format!("{} x {} = {}", i, j, product));
        }
    }
    println!("{}", format!("=== Break/Continue ==="));
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
    println!("{}", format!("=== Lists ==="));
    let mut nums: Vec<i64> = vec![10_i64, 20_i64, 30_i64, 40_i64, 50_i64];
    println!("{}", format!("Length: {}", nums.len() as i64));
    println!("{}", format!("First: {}", ({ let _v = &nums; let _i = 0_i64; let _idx = if _i < 0 { (_v.len() as i64 + _i) as usize } else { _i as usize }; _v.get(_idx).cloned() }).map_or("None".to_string(), |_v| format!("{}", _v))));
    println!("{}", format!("Last: {}", ({ let _v = &nums; let _i = 4_i64; let _idx = if _i < 0 { (_v.len() as i64 + _i) as usize } else { _i as usize }; _v.get(_idx).cloned() }).map_or("None".to_string(), |_v| format!("{}", _v))));
    let mut total: i64 = 0_i64;
    for n in nums.iter().cloned() {
        total = total + n;
    }
    println!("{}", format!("Sum: {}", total));
    let mut fruits: Vec<String> = vec!["apple".to_string(), "banana".to_string()];
    fruits.push("cherry".to_string());
    println!("{}", format!("Fruits count: {}", fruits.len() as i64));
    println!("{}", format!("=== Dict ==="));
    let mut ages: std::collections::HashMap<String, i64> = std::collections::HashMap::from([("Alice".to_string(), 30_i64), ("Bob".to_string(), 25_i64), ("Charlie".to_string(), 35_i64)]);
    println!("{}", format!("Alice is {} years old", (ages.get(&"Alice".to_string()).cloned()).map_or("None".to_string(), |_v| format!("{}", _v))));
    println!("{}", format!("Bob is {} years old", (ages.get(&"Bob".to_string()).cloned()).map_or("None".to_string(), |_v| format!("{}", _v))));
    println!("{}", format!("=== In Operator ==="));
    let mut numbers: Vec<i64> = vec![1_i64, 2_i64, 3_i64, 4_i64, 5_i64];
    let mut found: bool = numbers.contains(&3_i64);
    println!("{}", format!("3 in list: {}", found));
    let mut missing: bool = numbers.contains(&9_i64);
    println!("{}", format!("9 in list: {}", missing));
    println!("{}", format!("=== Tuples ==="));
    let mut point: (i64, i64, String) = (10_i64, 20_i64, "origin".to_string());
    println!("{}", format!("Tuple length: {}", 3_i64));
    println!("{}", format!("=== Tuple Unpacking ==="));
    let mut pair: (String, i64) = ("Sifr".to_string(), 2025_i64);
    let (name, year) = pair;
    println!("{}", format!("{} was born in {}", name, year));
    println!("{}", format!("=== F-Strings ==="));
    let mut a: i64 = 7_i64;
    let mut b: i64 = 8_i64;
    println!("{}", format!("{} * {} = {}", a, b, a * b));
    println!("{}", format!("Is {} > {}? {}", a, b, a > b));
    println!("{}", format!("=== String Operations ==="));
    let mut greeting: String = "  Hello, World!  ".to_string();
    println!("{}", greeting.trim().to_string());
    println!("{}", greeting.trim().to_string().to_uppercase());
    println!("{}", greeting.trim().to_string().to_lowercase());
    let mut lang: String = "sifr-lang".to_string();
    println!("{}", format!("Starts with 'sifr': {}", lang.starts_with("sifr".to_string().as_str())));
    println!("{}", format!("Ends with 'lang': {}", lang.ends_with("lang".to_string().as_str())));
    println!("{}", format!("=== FizzBuzz (1-15) ==="));
    fizzbuzz(15_i64);
}
