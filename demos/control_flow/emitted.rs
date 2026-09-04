// src/main.rs
use ::sifr_runtime::SifrInt;
use ::sifr_runtime::SifrRange;
use ::std::collections::HashMap;
#[expect(
    clippy::needless_pass_by_value,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn sum_range(n: SifrInt) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    for i in SifrRange::new_known_nonzero(SifrInt::from_i64(0), n.clone(), SifrInt::from_i64(1)) {
        total = ::std::ops::Add::add(&total, &i);
    }
    total
}
#[expect(
    clippy::needless_pass_by_value,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn fizzbuzz(n: SifrInt) {
    for i in SifrRange::new_known_nonzero(
        SifrInt::from_i64(1),
        ::std::ops::Add::add(&n, &SifrInt::from_i64(1)),
        SifrInt::from_i64(1),
    ) {
        if i.floor_mod_known_nonzero(&SifrInt::from_i64(15)) == SifrInt::from_i64(0) {
            println!("FizzBuzz");
        }
        if i.floor_mod_known_nonzero(&SifrInt::from_i64(3)) == SifrInt::from_i64(0)
            && i.floor_mod_known_nonzero(&SifrInt::from_i64(5)) != SifrInt::from_i64(0)
        {
            println!("Fizz");
        }
        if i.floor_mod_known_nonzero(&SifrInt::from_i64(5)) == SifrInt::from_i64(0)
            && i.floor_mod_known_nonzero(&SifrInt::from_i64(3)) != SifrInt::from_i64(0)
        {
            println!("Buzz");
        }
        if i.floor_mod_known_nonzero(&SifrInt::from_i64(3)) != SifrInt::from_i64(0)
            && i.floor_mod_known_nonzero(&SifrInt::from_i64(5)) != SifrInt::from_i64(0)
        {
            println!("{i}");
        }
    }
}
fn countdown(n: SifrInt) {
    let mut i: SifrInt = n;
    while i > SifrInt::from_i64(0) {
        println!("{i}");
        i = ::std::ops::Sub::sub(&i, &SifrInt::from_i64(1));
    }
    println!("Go!");
}
static SIFR_GENERATED_SIFR_HOISTED_DICT_0: ::std::sync::LazyLock<HashMap<String, SifrInt>> =
    ::std::sync::LazyLock::new(|| {
        HashMap::from([
            ("Alice".to_string(), SifrInt::from_i64(30)),
            ("Bob".to_string(), SifrInt::from_i64(25)),
            ("Charlie".to_string(), SifrInt::from_i64(35)),
        ])
    });
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn main() {
    println!("=== While Loop: Countdown ===");
    countdown(SifrInt::from_i64(5));
    println!("=== For Loop: Sum of 0..9 ===");
    let s: SifrInt = sum_range(SifrInt::from_i64(10));
    println!("Sum of range(10) = {s}");
    println!("=== Nested Loops: Multiplication Table ===");
    for i in SifrRange::new_known_nonzero(
        SifrInt::from_i64(1),
        SifrInt::from_i64(4),
        SifrInt::from_i64(1),
    ) {
        for j in SifrRange::new_known_nonzero(
            SifrInt::from_i64(1),
            SifrInt::from_i64(4),
            SifrInt::from_i64(1),
        ) {
            let product: SifrInt = ::std::ops::Mul::mul(&i, &j);
            println!("{i} x {j} = {product}");
        }
    }
    println!("=== Break/Continue ===");
    let mut i: SifrInt = SifrInt::from_i64(0);
    while i < SifrInt::from_i64(10) {
        i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
        if i == SifrInt::from_i64(3) {
            continue;
        }
        if i == SifrInt::from_i64(7) {
            break;
        }
        println!("{i}");
    }
    println!("=== Lists ===");
    let nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(10),
        SifrInt::from_i64(20),
        SifrInt::from_i64(30),
        SifrInt::from_i64(40),
        SifrInt::from_i64(50),
    ];
    println!("Length: {}", SifrInt::from(nums.len()));
    let first: Option<SifrInt> = {
        let sifr_generated_checked_read_collection = &nums;
        let sifr_generated_checked_read_index = SifrInt::from_i64(0);
        let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
            .normalize_index_or_len(sifr_generated_checked_read_collection.len());
        sifr_generated_checked_read_collection
            .get(sifr_generated_checked_read_normalized)
            .cloned()
    };
    if let Some(first) = first {
        println!("First: {first}");
    }
    let last: Option<SifrInt> = {
        let sifr_generated_checked_read_collection = &nums;
        let sifr_generated_checked_read_index = SifrInt::from_i64(4);
        let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
            .normalize_index_or_len(sifr_generated_checked_read_collection.len());
        sifr_generated_checked_read_collection
            .get(sifr_generated_checked_read_normalized)
            .cloned()
    };
    if let Some(last) = last {
        println!("Last: {last}");
    }
    let mut total: SifrInt = SifrInt::from_i64(0);
    #[expect(
        clippy::explicit_iter_loop,
        reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
    )]
    for n in nums.iter() {
        total = ::std::ops::Add::add(&total, n);
    }
    println!("Sum: {total}");
    let mut fruits: Vec<String> = vec!["apple".to_string(), "banana".to_string()];
    fruits.push("cherry".to_string());
    println!("Fruits count: {}", SifrInt::from(fruits.len()));
    println!("=== Dict ===");
    let ages = &*SIFR_GENERATED_SIFR_HOISTED_DICT_0;
    let alice_age: Option<SifrInt> = ages.get("Alice").cloned();
    if let Some(alice_age) = alice_age {
        println!("Alice is {alice_age} years old");
    }
    let bob_age: Option<SifrInt> = ages.get("Bob").cloned();
    if let Some(bob_age) = bob_age {
        println!("Bob is {bob_age} years old");
    }
    println!("=== In Operator ===");
    let numbers: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
        SifrInt::from_i64(4),
        SifrInt::from_i64(5),
    ];
    let found: bool = numbers.contains(&SifrInt::from_i64(3));
    println!("3 in list: {found}");
    let missing: bool = numbers.contains(&SifrInt::from_i64(9));
    println!("9 in list: {missing}");
    println!("=== Tuples ===");
    let _ = (
        SifrInt::from_i64(10),
        SifrInt::from_i64(20),
        "origin".to_string(),
    );
    println!("Tuple length: {}", SifrInt::from_i64(3));
    println!("=== Tuple Unpacking ===");
    let pair: (String, SifrInt) = ("Sifr".to_string(), SifrInt::from_i64(2025));
    let (name, year) = pair;
    let _ = name.chars().collect::<Vec<char>>();
    println!("{name} was born in {year}");
    println!("=== F-Strings ===");
    let a: SifrInt = SifrInt::from_i64(7);
    let b: SifrInt = SifrInt::from_i64(8);
    println!("{} * {} = {}", a, b, ::std::ops::Mul::mul(&a, &b));
    println!("Is {} > {}? {}", a, b, a > b);
    println!("=== String Operations ===");
    let greeting: String = "  Hello, World!  ".to_string();
    println!("{}", greeting.trim());
    println!("{}", greeting.trim().to_string().to_uppercase());
    println!("{}", greeting.trim().to_string().to_lowercase());
    let lang: String = "sifr-lang".to_string();
    println!("Starts with \'sifr\': {}", lang.starts_with("sifr"));
    println!("Ends with \'lang\': {}", lang.ends_with("lang"));
    println!("=== FizzBuzz (1-15) ===");
    fizzbuzz(SifrInt::from_i64(15));
}
