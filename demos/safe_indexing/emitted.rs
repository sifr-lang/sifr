// src/main.rs
use ::std::collections::HashMap;

use ::sifr_runtime::SifrInt;

static __SIFR_HOISTED_DICT_0: ::std::sync::LazyLock<HashMap<String, SifrInt>> = ::std::sync::LazyLock::new(|| HashMap::from([("alice".to_string(), SifrInt::from_i64(25)), ("bob".to_string(), SifrInt::from_i64(30))]));

fn main() {
    println!("=== Safe List Indexing ===");
    let nums: Vec<SifrInt> = vec![SifrInt::from_i64(10), SifrInt::from_i64(20), SifrInt::from_i64(30)];
    let val: Option<SifrInt> = {
    let __sifr_index_list = &nums;
    let __sifr_index_i = SifrInt::from_i64(1);
    let __sifr_index_norm = __sifr_index_i.normalize_index_or_len(__sifr_index_list.len());
    __sifr_index_list.get(__sifr_index_norm).cloned()
};
    if let Some(val) = val.clone() {
        println!("nums[1] = {}", val);
    }
    let oob: Option<SifrInt> = {
    let __sifr_index_list = &nums;
    let __sifr_index_i = SifrInt::from_i64(99);
    let __sifr_index_norm = __sifr_index_i.normalize_index_or_len(__sifr_index_list.len());
    __sifr_index_list.get(__sifr_index_norm).cloned()
};
    if let Some(oob) = oob.clone() {
        println!("nums[99] = {}", oob);
    } else {
        println!("nums[99] = None (safe!)");
    }
    println!("=== Safe Dict Indexing ===");
    let ages = &*__SIFR_HOISTED_DICT_0;
    let a: Option<SifrInt> = ages.get("alice").cloned();
    if let Some(a) = a.clone() {
        println!("ages[alice] = {}", a);
    }
    let c: Option<SifrInt> = ages.get("charlie").cloned();
    if let Some(c) = c.clone() {
        println!("ages[charlie] = {}", c);
    } else {
        println!("ages[charlie] = None (safe!)");
    }
    println!("=== Safe String Indexing ===");
    let s: String = "hello".to_string();
    let ch: Option<String> = {
    let __sifr_index_str = &s;
    let __sifr_index_i = SifrInt::from_i64(0);
    let __sifr_index_norm = __sifr_index_i.normalize_index_or_len(__sifr_index_str.chars().count());
    __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
};
    if let Some(ch) = ch {
        println!("s[0] = {}", ch);
    }
    let oob_ch: Option<String> = {
    let __sifr_index_str = &s;
    let __sifr_index_i = SifrInt::from_i64(99);
    let __sifr_index_norm = __sifr_index_i.normalize_index_or_len(__sifr_index_str.chars().count());
    __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
};
    if let Some(oob_ch) = oob_ch {
        println!("s[99] = {}", oob_ch);
    } else {
        println!("s[99] = None (safe!)");
    }
    println!("=== Negative Indexing ===");
    let items: Vec<SifrInt> = vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3), SifrInt::from_i64(4), SifrInt::from_i64(5)];
    let last: Option<SifrInt> = {
    let __sifr_index_list = &items;
    let __sifr_index_i = -&SifrInt::from_i64(1);
    let __sifr_index_norm = __sifr_index_i.normalize_index_or_len(__sifr_index_list.len());
    __sifr_index_list.get(__sifr_index_norm).cloned()
};
    if let Some(last) = last.clone() {
        println!("last item: {}", last);
    }
    let last_ch: Option<String> = {
    let __sifr_index_str = &s;
    let __sifr_index_i = -&SifrInt::from_i64(1);
    let __sifr_index_norm = __sifr_index_i.normalize_index_or_len(__sifr_index_str.chars().count());
    __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
};
    if let Some(last_ch) = last_ch {
        println!("last char: {}", last_ch);
    }
    println!("=== List pop() ===");
    let mut stack: Vec<SifrInt> = vec![SifrInt::from_i64(10), SifrInt::from_i64(20), SifrInt::from_i64(30)];
    let top: Option<SifrInt> = stack.pop();
    if let Some(top) = top.clone() {
        println!("popped: {}", top);
    }
    let _ = stack.pop();
    let _ = stack.pop();
    let empty: Option<SifrInt> = stack.pop();
    if let Some(empty) = empty.clone() {
        println!("got: {}", empty);
    } else {
        println!("empty pop: None");
    }
    println!("=== Dict get() and pop() ===");
    let mut data: HashMap<String, SifrInt> = HashMap::from([("alice".to_string(), SifrInt::from_i64(25)), ("bob".to_string(), SifrInt::from_i64(30))]);
    let g: Option<SifrInt> = data.get(("alice".to_string()).as_str()).cloned();
    if let Some(g) = g.clone() {
        println!("get alice: {}", g);
    }
    let gm: Option<SifrInt> = data.get(("missing".to_string()).as_str()).cloned();
    if let Some(gm) = gm.clone() {
        println!("get missing: {}", gm);
    } else {
        println!("get missing: None");
    }
    let p: Option<SifrInt> = data.remove(("bob".to_string()).as_str());
    if let Some(p) = p.clone() {
        println!("popped bob: {}", p);
    }
    println!("=== String find() ===");
    let text: String = "hello, world!".to_string();
    let pos: Option<SifrInt> = text.find("world").map(|i| SifrInt::from(i));
    if let Some(pos) = pos.clone() {
        println!("found world at: {}", pos);
    }
    let miss: Option<SifrInt> = text.find("xyz").map(|i| SifrInt::from(i));
    if let Some(miss) = miss.clone() {
        println!("found xyz at: {}", miss);
    } else {
        println!("xyz not found");
    }
    println!("=== Del Statement ===");
    let mut config: HashMap<String, SifrInt> = HashMap::from([("a".to_string(), SifrInt::from_i64(1)), ("b".to_string(), SifrInt::from_i64(2)), ("c".to_string(), SifrInt::from_i64(3))]);
    println!("before del: {}", SifrInt::from(config.len()));
    let _ = config.remove(&"b".to_string());
    println!("after del: {}", SifrInt::from(config.len()));
    println!("demo complete!");
}
