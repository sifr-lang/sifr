fn identity<T>(x: T) -> T {
    x
}

fn first<T: Clone>(items: &[T]) -> Option<T> {
    items.first().cloned()
}

fn apply(f: impl Fn(i64) -> i64, x: i64) -> i64 {
    f(x)
}

fn apply_twice(f: impl Fn(i64) -> i64, x: i64) -> i64 {
    f(f(x))
}

fn double(x: i64) -> i64 {
    x * 2
}

fn add_one(x: i64) -> i64 {
    x + 1
}

fn square(x: i64) -> i64 {
    x * x
}

fn main() {
    let a = identity(42_i64);
    let b = identity("hello".to_string());
    println!("{}", a);
    println!("{}", b);

    let nums = vec![10_i64, 20_i64, 30_i64];
    let words = vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()];
    let empty_words: Vec<String> = Vec::new();

    if let Some(first_num) = first(&nums) {
        println!("{}", first_num);
    }
    if let Some(first_word) = first(&words) {
        println!("{}", first_word);
    }

    match first(&empty_words) {
        Some(word) => println!("{}", word),
        None => println!("empty list -> None"),
    }

    println!("{}", apply(double, 5));
    println!("{}", apply(add_one, 99));
    println!("{}", apply_twice(add_one, 5));
    println!("{}", apply_twice(square, 3));
}
