// src/main.rs
mod sifr_generated_project_nominals {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ValueError {
        pub message: String,
    }
    impl ValueError {
        #[must_use]
        pub const fn new(message: String) -> Self {
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
use ::sifr_runtime::SifrInt;
use ::std::collections::HashMap;
pub use sifr_generated_project_nominals::ValueError;
fn demo_augmented_assign() {
    let mut x: SifrInt = SifrInt::from_i64(10);
    x = &x + &SifrInt::from_i64(5);
    x = &x - &SifrInt::from_i64(2);
    x = &x * &SifrInt::from_i64(3);
    println!("Augmented assign result: {x}");
    let mut s: String = "Hello".to_string();
    s.push_str(" World");
    println!("String +=: {s}");
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
fn greet(name: &str, greeting: &str, punctuation: &str) -> String {
    format!("{greeting}, {name}{punctuation}")
}
fn demo_negative_indexing() {
    let items: Vec<SifrInt> = vec![
        SifrInt::from_i64(10),
        SifrInt::from_i64(20),
        SifrInt::from_i64(30),
        SifrInt::from_i64(40),
        SifrInt::from_i64(50),
    ];
    println!(
        "Last element: {}",
        {
            let sifr_generated_index_list = &items;
            let sifr_generated_index_i = -&SifrInt::from_i64(1);
            let sifr_generated_index_norm =
                sifr_generated_index_i.normalize_index_or_len(sifr_generated_index_list.len());
            sifr_generated_index_list
                .get(sifr_generated_index_norm)
                .cloned()
        }
        .map_or_else(
            || "None".to_string(),
            |sifr_generated_v| sifr_generated_v.to_string()
        )
    );
    println!(
        "Second to last: {}",
        {
            let sifr_generated_index_list = &items;
            let sifr_generated_index_i = -&SifrInt::from_i64(2);
            let sifr_generated_index_norm =
                sifr_generated_index_i.normalize_index_or_len(sifr_generated_index_list.len());
            sifr_generated_index_list
                .get(sifr_generated_index_norm)
                .cloned()
        }
        .map_or_else(
            || "None".to_string(),
            |sifr_generated_v| sifr_generated_v.to_string()
        )
    );
    let s: String = "Sifr".to_string();
    println!(
        "Last char: {}",
        {
            let sifr_generated_index_str = &s;
            let sifr_generated_index_i = -&SifrInt::from_i64(1);
            let sifr_generated_index_norm = sifr_generated_index_i
                .normalize_index_or_len(sifr_generated_index_str.chars().count());
            sifr_generated_index_str
                .chars()
                .nth(sifr_generated_index_norm)
                .map(|c| c.to_string())
        }
        .map_or_else(
            || "None".to_string(),
            |sifr_generated_v| sifr_generated_v.to_string()
        )
    );
}
fn demo_step_slicing() {
    let nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(0),
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
        SifrInt::from_i64(4),
        SifrInt::from_i64(5),
        SifrInt::from_i64(6),
        SifrInt::from_i64(7),
        SifrInt::from_i64(8),
        SifrInt::from_i64(9),
    ];
    let evens: Vec<SifrInt> = {
        let sifr_generated_v_5f76 = &nums;
        let sifr_generated_len = sifr_generated_v_5f76.len();
        ::sifr_runtime::SifrSliceIndices::new_known_nonzero(
            sifr_generated_len,
            None,
            None,
            &SifrInt::from_i64(2),
        )
        .filter_map(|sifr_generated_i| sifr_generated_v_5f76.get(sifr_generated_i).cloned())
        .collect::<Vec<_>>()
    };
    println!("Evens: {} elements", SifrInt::from(evens.len()));
    let reversed: Vec<SifrInt> = {
        let sifr_generated_v_5f76 = &nums;
        let sifr_generated_len = sifr_generated_v_5f76.len();
        ::sifr_runtime::SifrSliceIndices::new_known_nonzero(
            sifr_generated_len,
            None,
            None,
            &-SifrInt::from_i64(1),
        )
        .filter_map(|sifr_generated_i| sifr_generated_v_5f76.get(sifr_generated_i).cloned())
        .collect::<Vec<_>>()
    };
    println!(
        "Reversed first: {}, last: {}",
        {
            let sifr_generated_index_list = &reversed;
            let sifr_generated_index_i = SifrInt::from_i64(0);
            let sifr_generated_index_norm =
                sifr_generated_index_i.normalize_index_or_len(sifr_generated_index_list.len());
            sifr_generated_index_list
                .get(sifr_generated_index_norm)
                .cloned()
        }
        .map_or_else(
            || "None".to_string(),
            |sifr_generated_v| sifr_generated_v.to_string()
        ),
        {
            let sifr_generated_index_list = &reversed;
            let sifr_generated_index_i = -&SifrInt::from_i64(1);
            let sifr_generated_index_norm =
                sifr_generated_index_i.normalize_index_or_len(sifr_generated_index_list.len());
            sifr_generated_index_list
                .get(sifr_generated_index_norm)
                .cloned()
        }
        .map_or_else(
            || "None".to_string(),
            |sifr_generated_v| sifr_generated_v.to_string()
        )
    );
    let s: String = "abcdefgh".to_string();
    println!("Every other char: {}", {
        let sifr_generated_s = &s;
        sifr_generated_s
            .chars()
            .step_by(2_usize)
            .collect::<String>()
    });
    println!("Reversed string: {}", {
        let sifr_generated_s = &s;
        sifr_generated_s.chars().rev().collect::<String>()
    });
}
fn demo_string_methods() {
    let s: String = "hello world".to_string();
    println!("Replace: {}", s.replace("world", "Sifr"));
    println!("Starts with \'hello\': {}", s.starts_with("hello"));
    println!("Ends with \'world\': {}", s.ends_with("world"));
    println!(
        "Title: {}",
        s.split_whitespace()
            .map(|w| {
                let mut c = w.chars();
                c.next()
                    .map(|f| f.to_uppercase().to_string() + &c.as_str().to_lowercase())
                    .unwrap_or_default()
            })
            .collect::<Vec<_>>()
            .join(&" ".to_string())
    );
    println!(
        "Is alpha: {}",
        !"abc".to_string().is_empty() && "abc".to_string().chars().all(|c| c.is_alphabetic())
    );
    println!(
        "Is digit: {}",
        !"123".to_string().is_empty() && "123".to_string().chars().all(|c| c.is_ascii_digit())
    );
    let separator: String = ", ".to_string();
    let items: Vec<String> = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    println!("Join: {}", items.join(&separator));
}
fn demo_list_methods() {
    let mut items: Vec<SifrInt> = vec![
        SifrInt::from_i64(3),
        SifrInt::from_i64(1),
        SifrInt::from_i64(4),
        SifrInt::from_i64(1),
        SifrInt::from_i64(5),
    ];
    items.push(SifrInt::from_i64(9));
    println!("After append: length={}", SifrInt::from(items.len()));
    println!(
        "Count of 1: {}",
        SifrInt::from(
            items
                .iter()
                .filter(|x| &**x == &SifrInt::from_i64(1))
                .count()
        )
    );
    println!("Contains 4: {}", items.contains(&SifrInt::from_i64(4)));
    let mut copy: Vec<SifrInt> = items.clone();
    copy.reverse();
    println!(
        "Reversed copy first: {}",
        {
            let sifr_generated_index_list = &copy;
            let sifr_generated_index_i = SifrInt::from_i64(0);
            let sifr_generated_index_norm =
                sifr_generated_index_i.normalize_index_or_len(sifr_generated_index_list.len());
            sifr_generated_index_list
                .get(sifr_generated_index_norm)
                .cloned()
        }
        .map_or_else(
            || "None".to_string(),
            |sifr_generated_v| sifr_generated_v.to_string()
        )
    );
}
fn demo_dict_methods() {
    let mut d: HashMap<String, SifrInt> = HashMap::from([
        ("a".to_string(), SifrInt::from_i64(1)),
        ("b".to_string(), SifrInt::from_i64(2)),
        ("c".to_string(), SifrInt::from_i64(3)),
    ]);
    println!(
        "Dict contains \'a\': {}",
        d.contains_key("a".to_string().as_str())
    );
    println!("Dict length: {}", SifrInt::from(d.len()));
    d.clear();
    println!("After clear: {}", SifrInt::from(d.len()));
}
fn demo_chained_comparisons() {
    let x: SifrInt = SifrInt::from_i64(5);
    if &SifrInt::from_i64(1) < &x && &x < &SifrInt::from_i64(10) {
        println!("5 is between 1 and 10");
    }
    let y: SifrInt = SifrInt::from_i64(15);
    if &SifrInt::from_i64(1) < &y && &y < &SifrInt::from_i64(10) {
        println!("This won't print");
    } else {
        println!("15 is NOT between 1 and 10");
    }
}
fn demo_string_multiply() {
    println!("{}", {
        let sifr_generated_n = SifrInt::from_i64(30);
        if &sifr_generated_n <= &0 {
            String::new()
        } else {
            "=".to_string()
                .repeat(::sifr_runtime::to_usize_proven(&sifr_generated_n))
        }
    });
    println!("  String Multiplication Demo");
    println!("{}", {
        let sifr_generated_n = SifrInt::from_i64(30);
        if &sifr_generated_n <= &0 {
            String::new()
        } else {
            "-".to_string()
                .repeat(::sifr_runtime::to_usize_proven(&sifr_generated_n))
        }
    });
}
fn demo_star_unpacking() {
    let items: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
        SifrInt::from_i64(4),
        SifrInt::from_i64(5),
    ];
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let sifr_generated_unpack_source = &items;
        let [sifr_generated_before_0, sifr_generated_star @ ..] =
            sifr_generated_unpack_source.as_slice()
        else {
            return Err(ValueError::new("not enough values to unpack".to_string()));
        };
        let first = sifr_generated_before_0.clone();
        let rest = sifr_generated_star.to_vec();
        println!(
            "First: {}, Rest length: {}",
            first,
            SifrInt::from(rest.len())
        );
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let error = sifr_generated_try_err.clone();
        println!("Unpack failed: {}", error.message.clone());
    }
}
fn demo_loop_else() {
    let items: Vec<SifrInt> = vec![
        SifrInt::from_i64(2),
        SifrInt::from_i64(4),
        SifrInt::from_i64(6),
        SifrInt::from_i64(8),
    ];
    let target: SifrInt = SifrInt::from_i64(5);
    let mut sifr_generated_broke = false;
    for item in items.iter().cloned() {
        if &item == &target {
            println!("Found target!");
            sifr_generated_broke = true;
            break;
        }
    }
    if !sifr_generated_broke {
        println!("Target not found in list (loop else)");
    }
}
fn demo_power() {
    println!("2 ** 10 = {}", SifrInt::from_i64(2).pow_known_valid(10_u32));
    println!("3 ** 3 = {}", SifrInt::from_i64(3).pow_known_valid(3_u32));
}
fn demo_walrus() {
    let items: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
        SifrInt::from_i64(4),
        SifrInt::from_i64(5),
        SifrInt::from_i64(6),
        SifrInt::from_i64(7),
        SifrInt::from_i64(8),
        SifrInt::from_i64(9),
        SifrInt::from_i64(10),
    ];
    let n = SifrInt::from(items.len());
    if &n > &SifrInt::from_i64(5) {
        println!("List has {n} items (more than 5)");
    }
}
const fn placeholder() {}
fn demo_builtins() {
    println!("abs(-42) = {}", (-&SifrInt::from_i64(42)).abs());
    println!(
        "round(3.7) = {}",
        SifrInt::from_f64_trunc(3.7_f64.round_ties_even()).ok_or_else(|| ValueError {
            message: "cannot round non-finite float to int".to_string()
        })
    );
    println!("repr(42) = {}", format!("{:?}", SifrInt::from_i64(42)));
}
fn main() {
    demo_augmented_assign();
    println!("classify(5): {}", classify(SifrInt::from_i64(5)));
    println!("classify(-3): {}", classify(-&SifrInt::from_i64(3)));
    println!(
        "{}",
        greet(&"Alice".to_string(), &"Hello".to_string(), &"!".to_string())
    );
    println!(
        "{}",
        greet(&"Bob".to_string(), &"Hi".to_string(), &"!".to_string())
    );
    println!(
        "{}",
        greet(&"Charlie".to_string(), &"Hey".to_string(), &"?".to_string())
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
    println!("17 divmod 5: quotient={q}, remainder={r}");
    demo_walrus();
    placeholder();
    demo_builtins();
    println!("All ergonomics features working!");
}
