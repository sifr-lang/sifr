use std::collections::HashMap;

fn demo_augmented_assign() {
    let mut x: i64 = 10 as i64;
    x += 5 as i64;
    x -= 2 as i64;
    x *= 3 as i64;
    println!("Augmented assign result: {}", x);
    let mut s: String = "Hello".to_string();
    s.push_str(" World");
    println!("String +=: {}", s);
    let mut items: Vec<i64> = vec![1 as i64, 2 as i64];
    items.extend(vec![3 as i64, 4 as i64]);
    println!("List += length: {}", items.len() as i64);
}

fn classify(n: i64) -> String {
    return if n > (0 as i64) { "positive".to_string() } else { "non-positive".to_string() };
}

fn greet(name: &String, greeting: &String, punctuation: &String) -> String {
    return format!("{}, {}{}", greeting, name, punctuation);
}

fn demo_negative_indexing() {
    let items: Vec<i64> = vec![10 as i64, 20 as i64, 30 as i64, 40 as i64, 50 as i64];
    println!("Last element: {}", ({
    let __sifr_index_list = &items;
    let __sifr_index_i = -(1 as i64);
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
}).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("Second to last: {}", ({
    let __sifr_index_list = &items;
    let __sifr_index_i = -(2 as i64);
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
}).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    let s: String = "Sifr".to_string();
    println!("Last char: {}", ({
    let __sifr_index_str = &s;
    let __sifr_index_i = -(1 as i64);
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
}).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
}

fn demo_step_slicing() {
    let nums: Vec<i64> = vec![0 as i64, 1 as i64, 2 as i64, 3 as i64, 4 as i64, 5 as i64, 6 as i64, 7 as i64, 8 as i64, 9 as i64];
    let evens: Vec<i64> = {
    let _v = &(nums);
    let _len = _v.len() as i64;
    let _step = 2 as i64;
    let _start = if _step > 0 { 0 as usize } else { (_len - 1) as usize };
    let _stop = if _step > 0 { _len as usize } else { usize::MAX };
    let mut _result = Vec::new();
    if _step > 0 {
        let mut _i = _start;
        while _i < _stop {
            if let Some(_el) = _v.get(_i) {
                _result.push(*_el);
            }
            _i += _step as usize;
        }
    } else {
        let mut _i = _start as i64;
        let _stop_i = _stop as i64;
        while _i > _stop_i {
            if _i >= 0 {
                if let Some(_el) = _v.get(_i as usize) {
                    _result.push(*_el);
                }
            }
            _i += _step;
        }
    }
    _result
};
    println!("Evens: {} elements", evens.len() as i64);
    let reversed: Vec<i64> = {
    let _v = &(nums);
    let _len = _v.len() as i64;
    let _step = -(1 as i64);
    let _start = if _step > 0 { 0 as usize } else { (_len - 1) as usize };
    let _stop = if _step > 0 { _len as usize } else { usize::MAX };
    let mut _result = Vec::new();
    if _step > 0 {
        let mut _i = _start;
        while _i < _stop {
            if let Some(_el) = _v.get(_i) {
                _result.push(*_el);
            }
            _i += _step as usize;
        }
    } else {
        let mut _i = _start as i64;
        let _stop_i = _stop as i64;
        while _i > _stop_i {
            if _i >= 0 {
                if let Some(_el) = _v.get(_i as usize) {
                    _result.push(*_el);
                }
            }
            _i += _step;
        }
    }
    _result
};
    println!("Reversed first: {}, last: {}", ({
    let __sifr_index_list = &reversed;
    let __sifr_index_i = 0 as i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
}).map_or("None".to_string().to_string(), |__v| format!("{}", __v)), ({
    let __sifr_index_list = &reversed;
    let __sifr_index_i = -(1 as i64);
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
}).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    let s: String = "abcdefgh".to_string();
    println!("Every other char: {}", {
    let _s = &(s);
    let _len = _s.chars().count() as i64;
    let _step = 2 as i64;
    let _start = if _step > 0 { 0 as usize } else { (_len - 1) as usize };
    let _stop = if _step > 0 { _len as usize } else { usize::MAX };
    let mut _result = String::new();
    if _step > 0 {
        let mut _i = _start;
        while _i < _stop {
            if let Some(_ch) = _s.chars().nth(_i) {
                _result.push(_ch);
            }
            _i += _step as usize;
        }
    } else {
        let mut _i = _start as i64;
        let _stop_i = _stop as i64;
        while _i > _stop_i {
            if _i >= 0 {
                if let Some(_ch) = _s.chars().nth(_i as usize) {
                    _result.push(_ch);
                }
            }
            _i += _step;
        }
    }
    _result
});
    println!("Reversed string: {}", {
    let _s = &(s);
    let _len = _s.chars().count() as i64;
    let _step = -(1 as i64);
    let _start = if _step > 0 { 0 as usize } else { (_len - 1) as usize };
    let _stop = if _step > 0 { _len as usize } else { usize::MAX };
    let mut _result = String::new();
    if _step > 0 {
        let mut _i = _start;
        while _i < _stop {
            if let Some(_ch) = _s.chars().nth(_i) {
                _result.push(_ch);
            }
            _i += _step as usize;
        }
    } else {
        let mut _i = _start as i64;
        let _stop_i = _stop as i64;
        while _i > _stop_i {
            if _i >= 0 {
                if let Some(_ch) = _s.chars().nth(_i as usize) {
                    _result.push(_ch);
                }
            }
            _i += _step;
        }
    }
    _result
});
}

fn demo_string_methods() {
    let s: String = "hello world".to_string();
    println!("Replace: {}", s.replace(&"world".to_string(), &"Sifr".to_string()));
    println!("Starts with \'hello\': {}", s.starts_with(&"hello".to_string()));
    println!("Ends with \'world\': {}", s.ends_with(&"world".to_string()));
    println!("Title: {}", s.split_whitespace().map(|w| {
    let mut c = w.chars();
    c.next().map(|f| f.to_uppercase().to_string() + &c.as_str().to_lowercase()).unwrap_or_default()
}).collect::<Vec<_>>().join(&" ".to_string()));
    println!("Is alpha: {}", !"abc".to_string().is_empty() && "abc".to_string().chars().all(|c| c.is_alphabetic()));
    println!("Is digit: {}", !"123".to_string().is_empty() && "123".to_string().chars().all(|c| c.is_ascii_digit()));
    let separator: String = ", ".to_string();
    let items: Vec<String> = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    println!("Join: {}", items.join(&separator));
}

fn demo_list_methods() {
    let mut items: Vec<i64> = vec![3 as i64, 1 as i64, 4 as i64, 1 as i64, 5 as i64];
    items.push(9 as i64);
    println!("After append: length={}", items.len() as i64);
    println!("Count of 1: {}", items.iter().filter(|x| **x == (1 as i64)).count() as i64);
    println!("Contains 4: {}", items.contains(&(4 as i64)));
    let mut copy: Vec<i64> = items.clone();
    copy.reverse();
    println!("Reversed copy first: {}", ({
    let __sifr_index_list = &copy;
    let __sifr_index_i = 0 as i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
}).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
}

fn demo_dict_methods() {
    let mut d: HashMap<String, i64> = HashMap::from([("a".to_string(), 1 as i64), ("b".to_string(), 2 as i64), ("c".to_string(), 3 as i64)]);
    println!("Dict contains \'a\': {}", d.contains_key(("a".to_string()).as_str()));
    println!("Dict length: {}", d.len() as i64);
    d.clear();
    println!("After clear: {}", d.len() as i64);
}

fn demo_chained_comparisons() {
    let x: i64 = 5 as i64;
    if ((1 as i64) < x) && (x < (10 as i64)) {
        println!("5 is between 1 and 10");
    }
    let y: i64 = 15 as i64;
    if ((1 as i64) < y) && (y < (10 as i64)) {
        println!("This won't print");
    } else {
        println!("15 is NOT between 1 and 10");
    }
}

fn demo_string_multiply() {
    println!("{}", {
    let __n = 30 as i64;
    if __n <= 0 { String::new() } else { ("=".to_string()).repeat(__n as usize) }
});
    println!("  String Multiplication Demo");
    println!("{}", {
    let __n = 30 as i64;
    if __n <= 0 { String::new() } else { ("-".to_string()).repeat(__n as usize) }
});
}

fn demo_star_unpacking() {
    let items: Vec<i64> = vec![1 as i64, 2 as i64, 3 as i64, 4 as i64, 5 as i64];
    let _star_tmp = &items;
    let first = _star_tmp[0];
    let rest = _star_tmp[1.._star_tmp.len()].to_vec();
    println!("First: {}, Rest length: {}", first, rest.len() as i64);
}

fn demo_loop_else() {
    let items: Vec<i64> = vec![2 as i64, 4 as i64, 6 as i64, 8 as i64];
    let target: i64 = 5 as i64;
    let mut _broke = false;
    for item in items.iter().copied() {
        if item == target {
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
    println!("2 ** 10 = {}", (2 as i64).pow((10 as i64) as u32));
    println!("3 ** 3 = {}", (3 as i64).pow((3 as i64) as u32));
}

fn divmod(a: i64, b: i64) -> (i64, i64) {
    return (a / b, a % b);
}

fn demo_walrus() {
    let items: Vec<i64> = vec![1 as i64, 2 as i64, 3 as i64, 4 as i64, 5 as i64, 6 as i64, 7 as i64, 8 as i64, 9 as i64, 10 as i64];
    let n = items.len() as i64;
    if n > (5 as i64) {
        println!("List has {} items (more than 5)", n);
    }
}

fn placeholder() {
    return;
}

fn demo_builtins() {
    println!("abs(-42) = {}", (-(42 as i64)).abs());
    println!("round(3.7) = {}", (3.7 as f64).round() as i64);
    println!("repr(42) = {}", format!("{:?}", 42 as i64));
}

fn main() {
    demo_augmented_assign();
    println!("classify(5): {}", classify(5 as i64));
    println!("classify(-3): {}", classify(-(3 as i64)));
    println!("{}", greet(&"Alice".to_string(), &"Hello".to_string(), &"!".to_string()));
    println!("{}", greet(&"Bob".to_string(), &"Hi".to_string(), &"!".to_string()));
    println!("{}", greet(&"Charlie".to_string(), &"Hey".to_string(), &"?".to_string()));
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
    let (q, r) = ((17 as i64) / (5 as i64), (17 as i64) % (5 as i64));
    println!("17 divmod 5: quotient={}, remainder={}", q, r);
    demo_walrus();
    placeholder();
    demo_builtins();
    println!("All ergonomics features working!");
}
