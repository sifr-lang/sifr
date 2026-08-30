// src/main.rs
mod __sifr_project_nominals {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ValueError {
        pub message: String,
    }
    impl ValueError {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for ValueError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ValueError {}
}
pub use __sifr_project_nominals::ValueError;
use ::std::collections::HashMap;
use ::sifr_runtime::SifrInt;
fn demo_augmented_assign() {
    let mut x: SifrInt = SifrInt::from_i64(10);
    x = &x + &SifrInt::from_i64(5);
    x = &x - &SifrInt::from_i64(2);
    x = &x * &SifrInt::from_i64(3);
    println!("Augmented assign result: {}", x);
    let mut s: String = "Hello".to_string();
    s.push_str(" World");
    println!("String +=: {}", s);
    let mut items: Vec<SifrInt> = vec![SifrInt::from_i64(1), SifrInt::from_i64(2)];
    items.extend(vec![SifrInt::from_i64(3), SifrInt::from_i64(4)]);
    println!("List += length: {}", SifrInt::from(items.len()));
}
fn classify(n: SifrInt) -> String {
    if &n > &SifrInt::from_i64(0) {
        "positive".to_string()
    } else {
        "non-positive".to_string()
    }
}
fn greet(name: &String, greeting: &String, punctuation: &String) -> String {
    format!("{}, {}{}", greeting, name, punctuation)
}
fn demo_negative_indexing() {
    let items: Vec<SifrInt> = vec![
        SifrInt::from_i64(10), SifrInt::from_i64(20), SifrInt::from_i64(30),
        SifrInt::from_i64(40), SifrInt::from_i64(50)
    ];
    println!(
        "Last element: {}", ({ let __sifr_index_list = & items; let __sifr_index_i = -&
        SifrInt::from_i64(1); let __sifr_index_norm = __sifr_index_i
        .normalize_index_or_len(__sifr_index_list.len()); __sifr_index_list
        .get(__sifr_index_norm).cloned() }).map_or("None".to_string().to_string(), | __v
        | format!("{}", __v))
    );
    println!(
        "Second to last: {}", ({ let __sifr_index_list = & items; let __sifr_index_i = -&
        SifrInt::from_i64(2); let __sifr_index_norm = __sifr_index_i
        .normalize_index_or_len(__sifr_index_list.len()); __sifr_index_list
        .get(__sifr_index_norm).cloned() }).map_or("None".to_string().to_string(), | __v
        | format!("{}", __v))
    );
    let s: String = "Sifr".to_string();
    println!(
        "Last char: {}", ({ let __sifr_index_str = & s; let __sifr_index_i = -&
        SifrInt::from_i64(1); let __sifr_index_norm = __sifr_index_i
        .normalize_index_or_len(__sifr_index_str.chars().count()); __sifr_index_str
        .chars().nth(__sifr_index_norm).map(| c | c.to_string()) }).map_or("None"
        .to_string().to_string(), | __v | format!("{}", __v))
    );
}
fn demo_step_slicing() {
    let nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(0), SifrInt::from_i64(1), SifrInt::from_i64(2),
        SifrInt::from_i64(3), SifrInt::from_i64(4), SifrInt::from_i64(5),
        SifrInt::from_i64(6), SifrInt::from_i64(7), SifrInt::from_i64(8),
        SifrInt::from_i64(9)
    ];
    let evens: Vec<SifrInt> = {
        let _v = &(nums);
        let _len = _v.len();
        ::sifr_runtime::SifrSliceIndices::new_known_nonzero(
                _len,
                None,
                None,
                &SifrInt::from_i64(2),
            )
            .map(|_i| _v[_i].clone())
            .collect::<Vec<_>>()
    };
    println!("Evens: {} elements", SifrInt::from(evens.len()));
    let reversed: Vec<SifrInt> = {
        let _v = &(nums);
        let _len = _v.len();
        ::sifr_runtime::SifrSliceIndices::new_known_nonzero(
                _len,
                None,
                None,
                &-(SifrInt::from_i64(1)),
            )
            .map(|_i| _v[_i].clone())
            .collect::<Vec<_>>()
    };
    println!(
        "Reversed first: {}, last: {}", ({ let __sifr_index_list = & reversed; let
        __sifr_index_i = SifrInt::from_i64(0); let __sifr_index_norm = __sifr_index_i
        .normalize_index_or_len(__sifr_index_list.len()); __sifr_index_list
        .get(__sifr_index_norm).cloned() }).map_or("None".to_string().to_string(), | __v
        | format!("{}", __v)), ({ let __sifr_index_list = & reversed; let __sifr_index_i
        = -& SifrInt::from_i64(1); let __sifr_index_norm = __sifr_index_i
        .normalize_index_or_len(__sifr_index_list.len()); __sifr_index_list
        .get(__sifr_index_norm).cloned() }).map_or("None".to_string().to_string(), | __v
        | format!("{}", __v))
    );
    let s: String = "abcdefgh".to_string();
    println!(
        "Every other char: {}", { let _s = & (s); _s.chars().step_by(2_usize).collect::<
        String > () }
    );
    println!(
        "Reversed string: {}", { let _s = & (s); _s.chars().rev().collect::< String > ()
        }
    );
}
fn demo_string_methods() {
    let s: String = "hello world".to_string();
    println!("Replace: {}", s.replace("world", "Sifr"));
    println!("Starts with \'hello\': {}", s.starts_with("hello"));
    println!("Ends with \'world\': {}", s.ends_with("world"));
    println!(
        "Title: {}", s.split_whitespace().map(| w | { let mut c = w.chars(); c.next()
        .map(| f | f.to_uppercase().to_string() + & c.as_str().to_lowercase())
        .unwrap_or_default() }).collect::< Vec < _ >> ().join(& " ".to_string())
    );
    println!(
        "Is alpha: {}", ! "abc".to_string().is_empty() && "abc".to_string().chars().all(|
        c | c.is_alphabetic())
    );
    println!(
        "Is digit: {}", ! "123".to_string().is_empty() && "123".to_string().chars().all(|
        c | c.is_ascii_digit())
    );
    let separator: String = ", ".to_string();
    let items: Vec<String> = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    println!("Join: {}", items.join(& separator));
}
fn demo_list_methods() {
    let mut items: Vec<SifrInt> = vec![
        SifrInt::from_i64(3), SifrInt::from_i64(1), SifrInt::from_i64(4),
        SifrInt::from_i64(1), SifrInt::from_i64(5)
    ];
    items.push(SifrInt::from_i64(9));
    println!("After append: length={}", SifrInt::from(items.len()));
    println!(
        "Count of 1: {}", SifrInt::from(items.iter().filter(| x | &** x == &
        SifrInt::from_i64(1)).count())
    );
    println!("Contains 4: {}", items.contains(& SifrInt::from_i64(4)));
    let mut copy: Vec<SifrInt> = items.clone();
    copy.reverse();
    println!(
        "Reversed copy first: {}", ({ let __sifr_index_list = & copy; let __sifr_index_i
        = SifrInt::from_i64(0); let __sifr_index_norm = __sifr_index_i
        .normalize_index_or_len(__sifr_index_list.len()); __sifr_index_list
        .get(__sifr_index_norm).cloned() }).map_or("None".to_string().to_string(), | __v
        | format!("{}", __v))
    );
}
fn demo_dict_methods() {
    let mut d: HashMap<String, SifrInt> = HashMap::from([
        ("a".to_string(), SifrInt::from_i64(1)),
        ("b".to_string(), SifrInt::from_i64(2)),
        ("c".to_string(), SifrInt::from_i64(3)),
    ]);
    println!("Dict contains \'a\': {}", d.contains_key(("a".to_string()).as_str()));
    println!("Dict length: {}", SifrInt::from(d.len()));
    d.clear();
    println!("After clear: {}", SifrInt::from(d.len()));
}
fn demo_chained_comparisons() {
    let x: SifrInt = SifrInt::from_i64(5);
    if (&SifrInt::from_i64(1) < &x) && (&x < &SifrInt::from_i64(10)) {
        println!("5 is between 1 and 10");
    }
    let y: SifrInt = SifrInt::from_i64(15);
    if (&SifrInt::from_i64(1) < &y) && (&y < &SifrInt::from_i64(10)) {
        println!("This won't print");
    } else {
        println!("15 is NOT between 1 and 10");
    }
}
fn demo_string_multiply() {
    println!(
        "{}", { let __n = SifrInt::from_i64(30); if & __n <= & 0 { String::new() } else {
        ("=".to_string()).repeat(::sifr_runtime::to_usize_proven(& (__n))) } }
    );
    println!("  String Multiplication Demo");
    println!(
        "{}", { let __n = SifrInt::from_i64(30); if & __n <= & 0 { String::new() } else {
        ("-".to_string()).repeat(::sifr_runtime::to_usize_proven(& (__n))) } }
    );
}
fn demo_star_unpacking() {
    let items: Vec<SifrInt> = vec![
        SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3),
        SifrInt::from_i64(4), SifrInt::from_i64(5)
    ];
    let _star_tmp = &items;
    let first = _star_tmp[0].clone();
    let rest = _star_tmp[1.._star_tmp.len()].to_vec();
    println!("First: {}, Rest length: {}", first, SifrInt::from(rest.len()));
}
fn demo_loop_else() {
    let items: Vec<SifrInt> = vec![
        SifrInt::from_i64(2), SifrInt::from_i64(4), SifrInt::from_i64(6),
        SifrInt::from_i64(8)
    ];
    let target: SifrInt = SifrInt::from_i64(5);
    let mut _broke = false;
    for item in items.iter().cloned() {
        if &item == &target {
            println!("Found target!");
            _broke = true;
            break;
        }
    }
    if !_broke {
        println!("Target not found in list (loop else)");
    }
}
fn demo_power() {
    println!(
        "2 ** 10 = {}", SifrInt::from_i64(2).pow_known_valid(& SifrInt::from_i64(10))
    );
    println!(
        "3 ** 3 = {}", SifrInt::from_i64(3).pow_known_valid(& SifrInt::from_i64(3))
    );
}
fn divmod(a: SifrInt, b: SifrInt) -> (SifrInt, SifrInt) {
    if &b == &SifrInt::from_i64(0) {
        return (SifrInt::from_i64(0), SifrInt::from_i64(0));
    }
    (a.floor_div_known_nonzero(&b), a.floor_mod_known_nonzero(&b))
}
fn demo_walrus() {
    let items: Vec<SifrInt> = vec![
        SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3),
        SifrInt::from_i64(4), SifrInt::from_i64(5), SifrInt::from_i64(6),
        SifrInt::from_i64(7), SifrInt::from_i64(8), SifrInt::from_i64(9),
        SifrInt::from_i64(10)
    ];
    let n = SifrInt::from(items.len());
    if &n > &SifrInt::from_i64(5) {
        println!("List has {} items (more than 5)", n);
    }
}
fn placeholder() {}
fn demo_builtins() {
    println!("abs(-42) = {}", (-& SifrInt::from_i64(42)).abs());
    println!(
        "round(3.7) = {}", SifrInt::from_f64_trunc((3.7_f64).round_ties_even())
        .ok_or_else(|| ValueError { message : "cannot round non-finite float to int"
        .to_string() })
    );
    println!("repr(42) = {}", format!("{:?}", SifrInt::from_i64(42)));
}
fn main() {
    demo_augmented_assign();
    println!("classify(5): {}", classify(SifrInt::from_i64(5)));
    println!("classify(-3): {}", classify(-& SifrInt::from_i64(3)));
    println!(
        "{}", greet(& "Alice".to_string(), & "Hello".to_string(), & "!".to_string())
    );
    println!("{}", greet(& "Bob".to_string(), & "Hi".to_string(), & "!".to_string()));
    println!(
        "{}", greet(& "Charlie".to_string(), & "Hey".to_string(), & "?".to_string())
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
    let (q, r) = (
        SifrInt::from_i64(17).floor_div_known_nonzero(&SifrInt::from_i64(5)),
        SifrInt::from_i64(17).floor_mod_known_nonzero(&SifrInt::from_i64(5)),
    );
    println!("17 divmod 5: quotient={}, remainder={}", q, r);
    demo_walrus();
    placeholder();
    demo_builtins();
    println!("All ergonomics features working!");
}
