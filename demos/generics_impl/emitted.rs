// src/main.rs
fn identity<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(x: &T) -> T {
    x.clone()
}

fn first<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(items: &Vec<T>) -> Option<T> {
    {
    let __sifr_index_list = &items;
    let __sifr_index_i = 0_i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).cloned()
}
}

fn apply(f: impl Fn(i64) -> i64, x: i64) -> i64 {
    f(x)
}

fn apply_twice(f: impl Fn(i64) -> i64, x: i64) -> i64 {
    f(f(x))
}

fn double(x: i64) -> i64 {
    x * (2_i64)
}

fn add_one(x: i64) -> i64 {
    x + (1_i64)
}

fn square(x: i64) -> i64 {
    x * x
}

fn main() {
    let a: i64 = identity(&(42_i64));
    let b: String = identity(&"hello".to_string());
    println!("{}", a);
    println!("{}", b);
    let nums: Vec<i64> = vec![10_i64, 20_i64, 30_i64];
    let words: Vec<String> = vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()];
    let empty_words: Vec<String> = vec![];
    let first_num: Option<i64> = first(&nums);
    if let Some(first_num) = first_num {
        println!("{}", first_num);
    }
    let first_word: Option<String> = first(&words);
    if let Some(first_word) = first_word {
        println!("{}", first_word);
    }
    let missing_word: Option<String> = first(&empty_words);
    if missing_word.is_none() {
        println!("empty list -> None");
    } else {
        if let Some(missing_word) = missing_word {
            println!("{}", missing_word);
        }
    }
    println!("{}", apply(double, 5_i64));
    println!("{}", apply(add_one, 99_i64));
    println!("{}", apply_twice(add_one, 5_i64));
    println!("{}", apply_twice(square, 3_i64));
}
