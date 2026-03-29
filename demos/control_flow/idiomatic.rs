use std::collections::HashMap;

fn sum_range(n: i64) -> i64 {
    (0..n).sum()
}

fn fizzbuzz(n: i64) {
    for i in 1..=n {
        if i % 15 == 0 {
            println!("FizzBuzz");
        } else if i % 3 == 0 {
            println!("Fizz");
        } else if i % 5 == 0 {
            println!("Buzz");
        } else {
            println!("{i}");
        }
    }
}

fn countdown(n: i64) {
    for i in (1..=n).rev() {
        println!("{i}");
    }
    println!("Go!");
}

fn tuple_len<T1, T2, T3>(_: &(T1, T2, T3)) -> usize {
    3
}

fn main() {
    println!("=== While Loop: Countdown ===");
    countdown(5);

    println!("=== For Loop: Sum of 0..9 ===");
    println!("Sum of range(10) = {}", sum_range(10));

    println!("=== Nested Loops: Multiplication Table ===");
    for i in 1..=3 {
        for j in 1..=3 {
            println!("{i} x {j} = {}", i * j);
        }
    }

    println!("=== Break/Continue ===");
    let mut i = 0;
    while i < 10 {
        i += 1;
        if i == 3 {
            continue;
        }
        if i == 7 {
            break;
        }
        println!("{i}");
    }

    println!("=== Lists ===");
    let nums = vec![10, 20, 30, 40, 50];
    println!("Length: {}", nums.len());
    if let Some(first) = nums.first() {
        println!("First: {first}");
    }
    if let Some(last) = nums.get(4) {
        println!("Last: {last}");
    }
    println!("Sum: {}", nums.iter().sum::<i64>());

    let mut fruits = vec!["apple", "banana"];
    fruits.push("cherry");
    println!("Fruits count: {}", fruits.len());

    println!("=== Dict ===");
    let ages = HashMap::from([("Alice", 30), ("Bob", 25), ("Charlie", 35)]);
    if let Some(alice_age) = ages.get("Alice") {
        println!("Alice is {alice_age} years old");
    }
    if let Some(bob_age) = ages.get("Bob") {
        println!("Bob is {bob_age} years old");
    }

    println!("=== In Operator ===");
    let numbers = [1, 2, 3, 4, 5];
    println!("3 in list: {}", numbers.contains(&3));
    println!("9 in list: {}", numbers.contains(&9));

    println!("=== Tuples ===");
    let point = (10, 20, "origin");
    println!("Tuple length: {}", tuple_len(&point));

    println!("=== Tuple Unpacking ===");
    let (name, year) = ("Sifr", 2025);
    println!("{name} was born in {year}");

    println!("=== F-Strings ===");
    let a = 7;
    let b = 8;
    println!("{a} * {b} = {}", a * b);
    println!("Is {a} > {b}? {}", a > b);

    println!("=== String Operations ===");
    let greeting = "  Hello, World!  ";
    println!("{}", greeting.trim());
    println!("{}", greeting.trim().to_uppercase());
    println!("{}", greeting.trim().to_lowercase());

    let lang = "sifr-lang";
    println!("Starts with 'sifr': {}", lang.starts_with("sifr"));
    println!("Ends with 'lang': {}", lang.ends_with("lang"));

    println!("=== FizzBuzz (1-15) ===");
    fizzbuzz(15);
}
