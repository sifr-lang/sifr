// src/main.rs
use ::std::collections::HashMap;

static __SIFR_HOISTED_DICT_0: ::std::sync::LazyLock<HashMap<String, i64>> = ::std::sync::LazyLock::new(|| HashMap::from([("alice".to_string(), 25_i64), ("bob".to_string(), 30_i64)]));

fn main() {
    println!("=== Safe List Indexing ===");
    let nums: Vec<i64> = vec![10_i64, 20_i64, 30_i64];
    let val: Option<i64> = {
    let __sifr_index_list = &nums;
    let __sifr_index_i = 1_i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
};
    if let Some(val) = val {
        println!("nums[1] = {}", val);
    }
    let oob: Option<i64> = {
    let __sifr_index_list = &nums;
    let __sifr_index_i = 99_i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
};
    if let Some(oob) = oob {
        println!("nums[99] = {}", oob);
    } else {
        println!("nums[99] = None (safe!)");
    }
    println!("=== Safe Dict Indexing ===");
    let ages = &*__SIFR_HOISTED_DICT_0;
    let a: Option<i64> = ages.get("alice").copied();
    if let Some(a) = a {
        println!("ages[alice] = {}", a);
    }
    let c: Option<i64> = ages.get("charlie").copied();
    if let Some(c) = c {
        println!("ages[charlie] = {}", c);
    } else {
        println!("ages[charlie] = None (safe!)");
    }
    println!("=== Safe String Indexing ===");
    let s: String = "hello".to_string();
    let ch: Option<String> = {
    let __sifr_index_str = &s;
    let __sifr_index_i = 0_i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
};
    if let Some(ch) = ch {
        println!("s[0] = {}", ch);
    }
    let oob_ch: Option<String> = {
    let __sifr_index_str = &s;
    let __sifr_index_i = 99_i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
};
    if let Some(oob_ch) = oob_ch {
        println!("s[99] = {}", oob_ch);
    } else {
        println!("s[99] = None (safe!)");
    }
    println!("=== Negative Indexing ===");
    let items: Vec<i64> = vec![1_i64, 2_i64, 3_i64, 4_i64, 5_i64];
    let last: Option<i64> = {
    let __sifr_index_list = &items;
    let __sifr_index_i = -(1_i64);
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
};
    if let Some(last) = last {
        println!("last item: {}", last);
    }
    let last_ch: Option<String> = {
    let __sifr_index_str = &s;
    let __sifr_index_i = -(1_i64);
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
};
    if let Some(last_ch) = last_ch {
        println!("last char: {}", last_ch);
    }
    println!("=== List pop() ===");
    let mut stack: Vec<i64> = vec![10_i64, 20_i64, 30_i64];
    let top: Option<i64> = stack.pop();
    if let Some(top) = top {
        println!("popped: {}", top);
    }
    let _ = stack.pop();
    let _ = stack.pop();
    let empty: Option<i64> = stack.pop();
    if let Some(empty) = empty {
        println!("got: {}", empty);
    } else {
        println!("empty pop: None");
    }
    println!("=== Dict get() and pop() ===");
    let mut data: HashMap<String, i64> = HashMap::from([("alice".to_string(), 25_i64), ("bob".to_string(), 30_i64)]);
    let g: Option<i64> = data.get(("alice".to_string()).as_str()).cloned();
    if let Some(g) = g {
        println!("get alice: {}", g);
    }
    let gm: Option<i64> = data.get(("missing".to_string()).as_str()).cloned();
    if let Some(gm) = gm {
        println!("get missing: {}", gm);
    } else {
        println!("get missing: None");
    }
    let p: Option<i64> = data.remove(("bob".to_string()).as_str());
    if let Some(p) = p {
        println!("popped bob: {}", p);
    }
    println!("=== String find() ===");
    let text: String = "hello, world!".to_string();
    let pos: Option<i64> = text.find("world").map(|i| i as i64);
    if let Some(pos) = pos {
        println!("found world at: {}", pos);
    }
    let miss: Option<i64> = text.find("xyz").map(|i| i as i64);
    if let Some(miss) = miss {
        println!("found xyz at: {}", miss);
    } else {
        println!("xyz not found");
    }
    println!("=== Del Statement ===");
    let mut config: HashMap<String, i64> = HashMap::from([("a".to_string(), 1_i64), ("b".to_string(), 2_i64), ("c".to_string(), 3_i64)]);
    println!("before del: {}", config.len() as i64);
    let _ = config.remove(&"b".to_string());
    println!("after del: {}", config.len() as i64);
    println!("demo complete!");
}
