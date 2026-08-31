// src/main.rs
use ::sifr_runtime::SifrInt;

fn identity<T: Clone + 'static>(x: &T) -> T {
    x.clone()
}

fn first<T: Clone + 'static>(items: &[T]) -> Option<T> {
    {
    let __sifr_checked_read_collection = &items;
    let __sifr_checked_read_index = SifrInt::from_i64(0);
    let __sifr_checked_read_normalized = __sifr_checked_read_index.normalize_index_or_len(__sifr_checked_read_collection.len());
    __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
}
}

fn apply(f: impl Fn(SifrInt) -> SifrInt, x: SifrInt) -> SifrInt {
    f(x.clone())
}

fn apply_twice(f: impl Fn(SifrInt) -> SifrInt, x: SifrInt) -> SifrInt {
    f(f(x.clone()))
}

fn double(x: SifrInt) -> SifrInt {
    &x * &SifrInt::from_i64(2)
}

fn add_one(x: SifrInt) -> SifrInt {
    &x + &SifrInt::from_i64(1)
}

fn square(x: SifrInt) -> SifrInt {
    &x * &x
}

fn main() {
    let a: SifrInt = identity(&SifrInt::from_i64(42));
    let b: String = identity(&"hello".to_string());
    println!("{}", a);
    println!("{}", b);
    let nums: Vec<SifrInt> = vec![SifrInt::from_i64(10), SifrInt::from_i64(20), SifrInt::from_i64(30)];
    let words: Vec<String> = vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()];
    let empty_words: Vec<String> = vec![];
    let first_num: Option<SifrInt> = first(&nums);
    if let Some(first_num) = first_num.clone() {
        println!("{}", first_num);
    }
    let first_word: Option<String> = first(&words);
    if let Some(first_word) = first_word {
        println!("{}", first_word);
    }
    let missing_word: Option<String> = first(&empty_words);
    if (missing_word.is_none()) {
        println!("empty list -> None");
    } else {
        if let Some(missing_word) = missing_word {
            println!("{}", missing_word);
        }
    }
    println!("{}", apply(double, SifrInt::from_i64(5)));
    println!("{}", apply(add_one, SifrInt::from_i64(99)));
    println!("{}", apply_twice(add_one, SifrInt::from_i64(5)));
    println!("{}", apply_twice(square, SifrInt::from_i64(3)));
}
