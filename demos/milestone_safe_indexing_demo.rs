use std::collections::HashMap;

fn main() {
    println!("{}", "=== Safe List Indexing ===".to_string());
    let mut nums: Vec<i64> = vec![10_i64, 20_i64, 30_i64];
    let mut val: Option<i64> = { let _v = &nums; let _i = 1_i64; let _idx = if _i < 0 { (_v.len() as i64 + _i) as usize } else { _i as usize }; _v.get(_idx).cloned() };
    if let Some(val) = val {
        println!("{}", format!("nums[1] = {}", val));
    }
    let mut oob: Option<i64> = { let _v = &nums; let _i = 99_i64; let _idx = if _i < 0 { (_v.len() as i64 + _i) as usize } else { _i as usize }; _v.get(_idx).cloned() };
    if let Some(oob) = oob {
        println!("{}", format!("nums[99] = {}", oob));
    } else {
        println!("{}", "nums[99] = None (safe!)".to_string());
    }
    println!("{}", "=== Safe Dict Indexing ===".to_string());
    let mut ages: std::collections::HashMap<String, i64> = std::collections::HashMap::from([("alice".to_string(), 25_i64), ("bob".to_string(), 30_i64)]);
    let mut a: Option<i64> = ages.get(&"alice".to_string()).cloned();
    if let Some(a) = a {
        println!("{}", format!("ages[alice] = {}", a));
    }
    let mut c: Option<i64> = ages.get(&"charlie".to_string()).cloned();
    if let Some(c) = c {
        println!("{}", format!("ages[charlie] = {}", c));
    } else {
        println!("{}", "ages[charlie] = None (safe!)".to_string());
    }
    println!("{}", "=== Safe String Indexing ===".to_string());
    let mut s: String = "hello".to_string();
    let mut ch: Option<String> = { let _s = &s; let _i = 0_i64; let _idx = if _i < 0 { (_s.chars().count() as i64 + _i) as usize } else { _i as usize }; _s.chars().nth(_idx).map(|c| c.to_string()) };
    if let Some(ch) = ch {
        println!("{}", format!("s[0] = {}", ch));
    }
    let mut oob_ch: Option<String> = { let _s = &s; let _i = 99_i64; let _idx = if _i < 0 { (_s.chars().count() as i64 + _i) as usize } else { _i as usize }; _s.chars().nth(_idx).map(|c| c.to_string()) };
    if let Some(oob_ch) = oob_ch {
        println!("{}", format!("s[99] = {}", oob_ch));
    } else {
        println!("{}", "s[99] = None (safe!)".to_string());
    }
    println!("{}", "=== Negative Indexing ===".to_string());
    let mut items: Vec<i64> = vec![1_i64, 2_i64, 3_i64, 4_i64, 5_i64];
    let mut last: Option<i64> = { let _v = &items; let _i = -1_i64; let _idx = if _i < 0 { (_v.len() as i64 + _i) as usize } else { _i as usize }; _v.get(_idx).cloned() };
    if let Some(last) = last {
        println!("{}", format!("last item: {}", last));
    }
    let mut last_ch: Option<String> = { let _s = &s; let _i = -1_i64; let _idx = if _i < 0 { (_s.chars().count() as i64 + _i) as usize } else { _i as usize }; _s.chars().nth(_idx).map(|c| c.to_string()) };
    if let Some(last_ch) = last_ch {
        println!("{}", format!("last char: {}", last_ch));
    }
    println!("{}", "=== List pop() ===".to_string());
    let mut stack: Vec<i64> = vec![10_i64, 20_i64, 30_i64];
    let mut top: Option<i64> = stack.pop();
    if let Some(top) = top {
        println!("{}", format!("popped: {}", top));
    }
    let _: Option<i64> = stack.pop();
    let _: Option<i64> = stack.pop();
    let mut empty: Option<i64> = stack.pop();
    if let Some(empty) = empty {
        println!("{}", format!("got: {}", empty));
    } else {
        println!("{}", "empty pop: None".to_string());
    }
    println!("{}", "=== Dict get() and pop() ===".to_string());
    let mut data: std::collections::HashMap<String, i64> = std::collections::HashMap::from([("alice".to_string(), 25_i64), ("bob".to_string(), 30_i64)]);
    let mut g: Option<i64> = data.get(&"alice".to_string()).cloned();
    if let Some(g) = g {
        println!("{}", format!("get alice: {}", g));
    }
    let mut gm: Option<i64> = data.get(&"missing".to_string()).cloned();
    if let Some(gm) = gm {
        println!("{}", format!("get missing: {}", gm));
    } else {
        println!("{}", "get missing: None".to_string());
    }
    let mut p: Option<i64> = data.remove(&"bob".to_string());
    if let Some(p) = p {
        println!("{}", format!("popped bob: {}", p));
    }
    println!("{}", "=== String find() ===".to_string());
    let mut text: String = "hello, world!".to_string();
    let mut pos: Option<i64> = text.find("world".to_string().as_str()).map(|i| i as i64);
    if let Some(pos) = pos {
        println!("{}", format!("found world at: {}", pos));
    }
    let mut miss: Option<i64> = text.find("xyz".to_string().as_str()).map(|i| i as i64);
    if let Some(miss) = miss {
        println!("{}", format!("found xyz at: {}", miss));
    } else {
        println!("{}", "xyz not found".to_string());
    }
    println!("{}", "=== Del Statement ===".to_string());
    let mut config: std::collections::HashMap<String, i64> = std::collections::HashMap::from([("a".to_string(), 1_i64), ("b".to_string(), 2_i64), ("c".to_string(), 3_i64)]);
    println!("{}", format!("before del: {}", config.len() as i64));
    let _ = config.remove(&"b".to_string());
    println!("{}", format!("after del: {}", config.len() as i64));
    println!("{}", "demo complete!".to_string());
}
