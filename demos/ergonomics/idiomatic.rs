use std::collections::HashMap;

fn demo_augmented_assign() {
    let mut x = 10;
    x += 5;
    x -= 2;
    x *= 3;
    println!("Augmented assign result: {x}");

    let mut s = String::from("Hello");
    s += " World";
    println!("String +=: {s}");

    let mut items = vec![1, 2];
    items.extend([3, 4]);
    println!("List += length: {}", items.len());
}

fn classify(n: i64) -> &'static str {
    if n > 0 {
        "positive"
    } else {
        "non-positive"
    }
}

#[derive(Clone, Copy)]
struct GreetOptions<'a> {
    greeting: &'a str,
    punctuation: &'a str,
}

impl Default for GreetOptions<'_> {
    fn default() -> Self {
        Self {
            greeting: "Hello",
            punctuation: "!",
        }
    }
}

fn greet(name: &str, options: GreetOptions<'_>) -> String {
    format!("{}, {name}{}", options.greeting, options.punctuation)
}

fn demo_negative_indexing() {
    let items = [10, 20, 30, 40, 50];
    println!("Last element: {}", items[items.len() - 1]);
    println!("Second to last: {}", items[items.len() - 2]);

    let s = "Sifr";
    println!("Last char: {}", s.chars().last().unwrap_or_default());
}

fn demo_step_slicing() {
    let nums = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let evens = nums.iter().step_by(2).copied().collect::<Vec<_>>();
    println!("Evens: {} elements", evens.len());

    let reversed = nums.iter().rev().copied().collect::<Vec<_>>();
    println!(
        "Reversed first: {}, last: {}",
        reversed.first().copied().unwrap_or_default(),
        reversed.last().copied().unwrap_or_default()
    );

    let s = "abcdefgh";
    println!(
        "Every other char: {}",
        s.chars().step_by(2).collect::<String>()
    );
    println!("Reversed string: {}", s.chars().rev().collect::<String>());
}

fn title_case(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => {
                    let mut out = first.to_uppercase().collect::<String>();
                    out.push_str(&chars.as_str().to_lowercase());
                    out
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn demo_string_methods() {
    let s = "hello world";
    println!("Replace: {}", s.replace("world", "Sifr"));
    println!("Starts with 'hello': {}", s.starts_with("hello"));
    println!("Ends with 'world': {}", s.ends_with("world"));
    println!("Title: {}", title_case(s));
    println!("Is alpha: {}", "abc".chars().all(char::is_alphabetic));
    println!("Is digit: {}", "123".chars().all(|c| c.is_ascii_digit()));
    println!("Join: {}", ["a", "b", "c"].join(", "));
}

fn demo_list_methods() {
    let mut items = vec![3, 1, 4, 1, 5];
    items.push(9);
    println!("After append: length={}", items.len());
    println!(
        "Count of 1: {}",
        items.iter().filter(|&&value| value == 1).count()
    );
    println!("Contains 4: {}", items.contains(&4));

    let mut copy = items.clone();
    copy.reverse();
    println!(
        "Reversed copy first: {}",
        copy.first().copied().unwrap_or_default()
    );
}

fn demo_dict_methods() {
    let mut d = HashMap::from([("a", 1), ("b", 2), ("c", 3)]);
    println!("Dict contains 'a': {}", d.contains_key("a"));
    println!("Dict length: {}", d.len());
    d.clear();
    println!("After clear: {}", d.len());
}

fn demo_chained_comparisons() {
    let x = 5;
    if 1 < x && x < 10 {
        println!("5 is between 1 and 10");
    }

    let y = 15;
    if 1 < y && y < 10 {
        println!("This won't print");
    } else {
        println!("15 is NOT between 1 and 10");
    }
}

fn demo_string_multiply() {
    println!("{}", "=".repeat(30));
    println!("  String Multiplication Demo");
    println!("{}", "-".repeat(30));
}

fn demo_star_unpacking() {
    let items = [1, 2, 3, 4, 5];
    let first = items[0];
    let rest = &items[1..];
    println!("First: {first}, Rest length: {}", rest.len());
}

fn demo_loop_else() {
    let items = [2, 4, 6, 8];
    let target = 5;
    let mut found = false;
    for item in items {
        if item == target {
            found = true;
            println!("Found target!");
            break;
        }
    }
    if !found {
        println!("Target not found in list (loop else)");
    }
}

fn demo_power() {
    println!("2 ** 10 = {}", 2_i64.pow(10));
    println!("3 ** 3 = {}", 3_i64.pow(3));
}

fn divmod(a: i64, b: i64) -> (i64, i64) {
    (a / b, a % b)
}

fn demo_walrus() {
    let items = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let n = items.len();
    if n > 5 {
        println!("List has {n} items (more than 5)");
    }
}

fn placeholder() {}

fn demo_builtins() {
    println!("abs(-42) = {}", (-42_i64).abs());
    println!("round(3.7) = {}", 3.7_f64.round() as i64);
    println!("repr(42) = {}", 42);
}

fn main() {
    demo_augmented_assign();
    println!("classify(5): {}", classify(5));
    println!("classify(-3): {}", classify(-3));
    println!("{}", greet("Alice", GreetOptions::default()));
    println!(
        "{}",
        greet(
            "Bob",
            GreetOptions {
                greeting: "Hi",
                ..GreetOptions::default()
            }
        )
    );
    println!(
        "{}",
        greet(
            "Charlie",
            GreetOptions {
                greeting: "Hey",
                punctuation: "?",
            }
        )
    );
    demo_negative_indexing();
    demo_step_slicing();
    demo_string_methods();
    demo_list_methods();
    demo_dict_methods();
    demo_chained_comparisons();
    demo_string_multiply();
    demo_star_unpacking();
    demo_loop_else();
    demo_power();
    let (q, r) = divmod(17, 5);
    println!("17 divmod 5: quotient={q}, remainder={r}");
    demo_walrus();
    placeholder();
    demo_builtins();
    println!("All ergonomics features working!");
}
