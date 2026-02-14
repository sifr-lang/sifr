use std::collections::HashMap;

fn demo_augmented_assign() {
    let mut x: i64 = 10_i64;
    x += 5_i64;
    x -= 2_i64;
    x *= 3_i64;
    println!("{}", format!("Augmented assign result: {}", x));
    let mut s: String = "Hello".to_string();
    s.push_str(&" World".to_string());
    println!("{}", format!("String +=: {}", s));
    let mut items: Vec<i64> = vec![1_i64, 2_i64];
    items.extend(vec![3_i64, 4_i64]);
    println!("{}", format!("List += length: {}", items.len() as i64));
}

fn classify(n: i64) -> String {
    return if n > 0_i64 { "positive".to_string() } else { "non-positive".to_string() };
}

fn greet(name: String, greeting: String, punctuation: String) -> String {
    return format!("{}, {}{}", greeting, name, punctuation);
}

fn demo_negative_indexing() {
    let mut items: Vec<i64> = vec![10_i64, 20_i64, 30_i64, 40_i64, 50_i64];
    println!("{}", format!("Last element: {}", ({ let _v = &items; let _i = -1_i64; let _idx = if _i < 0 { (_v.len() as i64 + _i) as usize } else { _i as usize }; _v.get(_idx).cloned() }).map_or("None".to_string(), |_v| format!("{}", _v))));
    println!("{}", format!("Second to last: {}", ({ let _v = &items; let _i = -2_i64; let _idx = if _i < 0 { (_v.len() as i64 + _i) as usize } else { _i as usize }; _v.get(_idx).cloned() }).map_or("None".to_string(), |_v| format!("{}", _v))));
    let mut s: String = "Sifr".to_string();
    println!("{}", format!("Last char: {}", ({ let _s = &s; let _i = -1_i64; let _idx = if _i < 0 { (_s.chars().count() as i64 + _i) as usize } else { _i as usize }; _s.chars().nth(_idx).map(|c| c.to_string()) }).map_or("None".to_string(), |_v| format!("{}", _v))));
}

fn demo_step_slicing() {
    let mut nums: Vec<i64> = vec![0_i64, 1_i64, 2_i64, 3_i64, 4_i64, 5_i64, 6_i64, 7_i64, 8_i64, 9_i64];
    let mut evens: Vec<i64> = { let _v = &nums; let _len = _v.len() as i64; let _step = 2_i64; let _start = if _step > 0 { 0 } else { (_len - 1) as usize }; let _stop = if _step > 0 { _len as usize } else { 0_usize.wrapping_sub(1) }; let mut _result = Vec::new(); if _step > 0 { let mut _i = _start; while _i < _stop { _result.push(_v[_i].clone()); _i += _step as usize; } } else { let mut _i = _start as i64; let _stop_i = _stop as i64; while _i > _stop_i { _result.push(_v[_i as usize].clone()); _i += _step; } }; _result };
    println!("{}", format!("Evens: {} elements", evens.len() as i64));
    let mut reversed: Vec<i64> = { let _v = &nums; let _len = _v.len() as i64; let _step = -1_i64; let _start = if _step > 0 { 0 } else { (_len - 1) as usize }; let _stop = if _step > 0 { _len as usize } else { 0_usize.wrapping_sub(1) }; let mut _result = Vec::new(); if _step > 0 { let mut _i = _start; while _i < _stop { _result.push(_v[_i].clone()); _i += _step as usize; } } else { let mut _i = _start as i64; let _stop_i = _stop as i64; while _i > _stop_i { _result.push(_v[_i as usize].clone()); _i += _step; } }; _result };
    println!("{}", format!("Reversed first: {}, last: {}", ({ let _v = &reversed; let _i = 0_i64; let _idx = if _i < 0 { (_v.len() as i64 + _i) as usize } else { _i as usize }; _v.get(_idx).cloned() }).map_or("None".to_string(), |_v| format!("{}", _v)), ({ let _v = &reversed; let _i = -1_i64; let _idx = if _i < 0 { (_v.len() as i64 + _i) as usize } else { _i as usize }; _v.get(_idx).cloned() }).map_or("None".to_string(), |_v| format!("{}", _v))));
    let mut s: String = "abcdefgh".to_string();
    println!("{}", format!("Every other char: {}", { let _s: Vec<char> = s.chars().collect(); let _len = _s.len() as i64; let _step = 2_i64; let _start = if _step > 0 { 0 } else { (_len - 1) as usize }; let _stop = if _step > 0 { _len as usize } else { 0_usize.wrapping_sub(1) }; let mut _result = String::new(); if _step > 0 { let mut _i = _start; while _i < _stop { _result.push(_s[_i]); _i += _step as usize; } } else { let mut _i = _start as i64; let _stop_i = _stop as i64; while _i > _stop_i { _result.push(_s[_i as usize]); _i += _step; } }; _result }));
    println!("{}", format!("Reversed string: {}", { let _s: Vec<char> = s.chars().collect(); let _len = _s.len() as i64; let _step = -1_i64; let _start = if _step > 0 { 0 } else { (_len - 1) as usize }; let _stop = if _step > 0 { _len as usize } else { 0_usize.wrapping_sub(1) }; let mut _result = String::new(); if _step > 0 { let mut _i = _start; while _i < _stop { _result.push(_s[_i]); _i += _step as usize; } } else { let mut _i = _start as i64; let _stop_i = _stop as i64; while _i > _stop_i { _result.push(_s[_i as usize]); _i += _step; } }; _result }));
}

fn demo_string_methods() {
    let mut s: String = "hello world".to_string();
    println!("{}", format!("Replace: {}", s.replace("world".to_string().as_str(), "Sifr".to_string().as_str())));
    println!("{}", format!("Starts with 'hello': {}", s.starts_with("hello".to_string().as_str())));
    println!("{}", format!("Ends with 'world': {}", s.ends_with("world".to_string().as_str())));
    println!("{}", format!("Title: {}", s.split_whitespace().map(|w| { let mut c = w.chars(); match c.next() { None => String::new(), Some(f) => f.to_uppercase().to_string() + &c.as_str().to_lowercase() } }).collect::<Vec<_>>().join(" ")));
    println!("{}", format!("Is alpha: {}", !"abc".to_string().is_empty() && "abc".to_string().chars().all(|c| c.is_alphabetic())));
    println!("{}", format!("Is digit: {}", !"123".to_string().is_empty() && "123".to_string().chars().all(|c| c.is_ascii_digit())));
    let mut separator: String = ", ".to_string();
    let mut items: Vec<String> = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    println!("{}", format!("Join: {}", items.join(&separator)));
}

fn demo_list_methods() {
    let mut items: Vec<i64> = vec![3_i64, 1_i64, 4_i64, 1_i64, 5_i64];
    items.push(9_i64);
    println!("{}", format!("After append: length={}", items.len() as i64));
    println!("{}", format!("Count of 1: {}", items.iter().filter(|x| **x == 1_i64).count() as i64));
    println!("{}", format!("Contains 4: {}", items.contains(&4_i64)));
    let mut copy: Vec<i64> = items.clone();
    copy.reverse();
    println!("{}", format!("Reversed copy first: {}", ({ let _v = &copy; let _i = 0_i64; let _idx = if _i < 0 { (_v.len() as i64 + _i) as usize } else { _i as usize }; _v.get(_idx).cloned() }).map_or("None".to_string(), |_v| format!("{}", _v))));
}

fn demo_dict_methods() {
    let mut d: std::collections::HashMap<String, i64> = std::collections::HashMap::from([("a".to_string(), 1_i64), ("b".to_string(), 2_i64), ("c".to_string(), 3_i64)]);
    println!("{}", format!("Dict contains 'a': {}", d.contains_key(&"a".to_string())));
    println!("{}", format!("Dict length: {}", d.len() as i64));
    d.clear();
    println!("{}", format!("After clear: {}", d.len() as i64));
}

fn demo_chained_comparisons() {
    let mut x: i64 = 5_i64;
    if (1_i64 < x && x < 10_i64) {
        println!("{}", "5 is between 1 and 10".to_string());
    }
    let mut y: i64 = 15_i64;
    if (1_i64 < y && y < 10_i64) {
        println!("{}", "This won't print".to_string());
    } else {
        println!("{}", "15 is NOT between 1 and 10".to_string());
    }
}

fn demo_string_multiply() {
    println!("{}", "=".to_string().repeat(30_i64 as usize));
    println!("{}", "  String Multiplication Demo".to_string());
    println!("{}", "-".to_string().repeat(30_i64 as usize));
}

fn demo_star_unpacking() {
    let mut items: Vec<i64> = vec![1_i64, 2_i64, 3_i64, 4_i64, 5_i64];
    let _star_tmp = items.clone();
    let first = _star_tmp[0].clone();
    let rest = _star_tmp[1..].to_vec();
    println!("{}", format!("First: {}, Rest length: {}", first, rest.len() as i64));
}

fn demo_loop_else() {
    let mut items: Vec<i64> = vec![2_i64, 4_i64, 6_i64, 8_i64];
    let mut target: i64 = 5_i64;
    let mut _broke = false;
    for item in items.iter().cloned() {
        if item == target {
            println!("{}", "Found target!".to_string());
            _broke = true;
            break;
        }
    }
    if !_broke {
        println!("{}", "Target not found in list (loop else)".to_string());
    }
}

fn demo_power() {
    println!("{}", format!("2 ** 10 = {}", 2_i64.pow(10_i64 as u32)));
    println!("{}", format!("3 ** 3 = {}", 3_i64.pow(3_i64 as u32)));
}

fn divmod(a: i64, b: i64) -> (i64, i64) {
    return (a / b, a % b);
}

fn demo_walrus() {
    let mut items: Vec<i64> = vec![1_i64, 2_i64, 3_i64, 4_i64, 5_i64, 6_i64, 7_i64, 8_i64, 9_i64, 10_i64];
    let n: i64 = items.len() as i64;
    if n > 5_i64 {
        println!("{}", format!("List has {} items (more than 5)", n));
    }
}

fn placeholder() {
}

fn demo_builtins() {
    println!("{}", format!("abs(-42) = {}", (-42_i64).abs()));
    println!("{}", format!("round(3.7) = {}", 3.7_f64.round() as i64));
    println!("{}", format!("repr(42) = {}", format!("{:?}", 42_i64)));
}

fn main() {
    demo_augmented_assign();
    println!("{}", format!("classify(5): {}", classify(5_i64)));
    println!("{}", format!("classify(-3): {}", classify(-3_i64)));
    println!("{}", greet("Alice".to_string(), "Hello".to_string(), "!".to_string()));
    println!("{}", greet("Bob".to_string(), "Hi".to_string(), "!".to_string()));
    println!("{}", greet("Charlie".to_string(), "Hey".to_string(), "?".to_string()));
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
    let (q, r) = divmod(17_i64, 5_i64);
    println!("{}", format!("17 divmod 5: quotient={}, remainder={}", q, r));
    demo_walrus();
    placeholder();
    demo_builtins();
    println!("{}", "All milestone_ergonomics features working!".to_string());
}
