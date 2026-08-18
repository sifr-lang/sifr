// src/main.rs
use ::std::collections::HashMap;

fn sum_range(n: i64) -> i64 {
    let mut total: i64 = 0_i64;
    for i in 0_i64..n {
        total += i;
    }
    total
}

fn fizzbuzz(n: i64) {
    for i in 1_i64..n + (1_i64) {
        if (i % (15_i64)) == (0_i64) {
            println!("FizzBuzz");
        }
        if (i % (3_i64)) == (0_i64) {
            if (i % (5_i64)) != (0_i64) {
                println!("Fizz");
            }
        }
        if (i % (5_i64)) == (0_i64) {
            if (i % (3_i64)) != (0_i64) {
                println!("Buzz");
            }
        }
        if (i % (3_i64)) != (0_i64) {
            if (i % (5_i64)) != (0_i64) {
                println!("{}", i);
            }
        }
    }
}

fn countdown(n: i64) {
    let mut i: i64 = n;
    while i > (0_i64) {
        println!("{}", i);
        i -= 1_i64;
    }
    println!("Go!");
}

static __SIFR_HOISTED_DICT_0: ::std::sync::LazyLock<HashMap<String, i64>> = ::std::sync::LazyLock::new(|| HashMap::from([("Alice".to_string(), 30_i64), ("Bob".to_string(), 25_i64), ("Charlie".to_string(), 35_i64)]));

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
    while i < (10_i64) {
        i += 1_i64;
        if i == (3_i64) {
            continue;
        }
        if i == (7_i64) {
            break;
        }
        println!("{}", i);
    }
    println!("=== Lists ===");
    let nums: Vec<i64> = vec![10_i64, 20_i64, 30_i64, 40_i64, 50_i64];
    println!("Length: {}", nums.len() as i64);
    let first: Option<i64> = {
    let __sifr_index_list = &nums;
    let __sifr_index_i = 0_i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
};
    if let Some(first) = first {
        println!("First: {}", first);
    }
    let last: Option<i64> = {
    let __sifr_index_list = &nums;
    let __sifr_index_i = 4_i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
};
    if let Some(last) = last {
        println!("Last: {}", last);
    }
    let mut total: i64 = 0_i64;
    for n in nums.iter().copied() {
        total += n;
    }
    println!("Sum: {}", total);
    let mut fruits: Vec<String> = vec!["apple".to_string(), "banana".to_string()];
    fruits.push("cherry".to_string());
    println!("Fruits count: {}", fruits.len() as i64);
    println!("=== Dict ===");
    let ages = &*__SIFR_HOISTED_DICT_0;
    let alice_age: Option<i64> = ages.get("Alice").copied();
    if let Some(alice_age) = alice_age {
        println!("Alice is {} years old", alice_age);
    }
    let bob_age: Option<i64> = ages.get("Bob").copied();
    if let Some(bob_age) = bob_age {
        println!("Bob is {} years old", bob_age);
    }
    println!("=== In Operator ===");
    let numbers: Vec<i64> = vec![1_i64, 2_i64, 3_i64, 4_i64, 5_i64];
    let found: bool = numbers.contains(&(3_i64));
    println!("3 in list: {}", found);
    let missing: bool = numbers.contains(&(9_i64));
    println!("9 in list: {}", missing);
    println!("=== Tuples ===");
    let point: (i64, i64, String) = (10_i64, 20_i64, "origin".to_string());
    println!("Tuple length: {}", 3_i64);
    println!("=== Tuple Unpacking ===");
    let pair: (String, i64) = ("Sifr".to_string(), 2025_i64);
    let (name, year) = pair;
    let __sifr_chars_name: Vec<char> = name.chars().collect::<Vec<char>>();
    println!("{} was born in {}", name, year);
    println!("=== F-Strings ===");
    let a: i64 = 7_i64;
    let b: i64 = 8_i64;
    println!("{} * {} = {}", a, b, a * b);
    println!("Is {} > {}? {}", a, b, (a > b));
    println!("=== String Operations ===");
    let greeting: String = "  Hello, World!  ".to_string();
    println!("{}", greeting.trim().to_string());
    println!("{}", greeting.trim().to_string().to_uppercase());
    println!("{}", greeting.trim().to_string().to_lowercase());
    let lang: String = "sifr-lang".to_string();
    println!("Starts with \'sifr\': {}", lang.starts_with("sifr"));
    println!("Ends with \'lang\': {}", lang.ends_with("lang"));
    println!("=== FizzBuzz (1-15) ===");
    fizzbuzz(15_i64);
}
