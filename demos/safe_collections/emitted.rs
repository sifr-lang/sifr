// src/main.rs
fn main() {
    let mut items: Vec<i64> = vec![10_i64, 20_i64, 30_i64];
    {
    if let Some(__pos) = items.iter().position(|__x| *__x == (99_i64)) {
        items.remove(__pos);
    }
};
    println!("After removing missing 99:");
    println!("{:?}", items);
    {
    if let Some(__pos) = items.iter().position(|__x| *__x == (20_i64)) {
        items.remove(__pos);
    }
};
    println!("After removing 20:");
    println!("{:?}", items);
    let names: Vec<String> = vec!["alice".to_string(), "bob".to_string(), "charlie".to_string()];
    let pos: Option<i64> = {
    let __len = names.len() as i64;
    let __start = 0;
    let __stop = __len;
    let mut __i = __start;
    let mut __result = None;
    while (__i < __stop) && (__result == None) {
        if let Some(__x) = names.get(__i as usize) {
            if __x == &"bob".to_string() {
                __result = Some(__i);
            }
        }
        __i += 1;
    }
    __result
};
    if let Some(pos) = pos {
        println!("Found \'bob\' at index {}", pos);
    } else {
        println!("\'bob\' not found");
    }
    let missing: Option<i64> = {
    let __len = names.len() as i64;
    let __start = 0;
    let __stop = __len;
    let mut __i = __start;
    let mut __result = None;
    while (__i < __stop) && (__result == None) {
        if let Some(__x) = names.get(__i as usize) {
            if __x == &"dave".to_string() {
                __result = Some(__i);
            }
        }
        __i += 1;
    }
    __result
};
    if let Some(missing) = missing {
        println!("Found \'dave\' at index {}", missing);
    } else {
        println!("\'dave\' not found (safe: returned None)");
    }
    let nums: Vec<i64> = vec![5_i64, 3_i64, 8_i64, 1_i64, 9_i64];
    let lo: Option<i64> = (nums).iter().copied().min();
    let hi: Option<i64> = (nums).iter().copied().max();
    if let Some(lo) = lo {
        if let Some(hi) = hi {
            println!("min={}, max={}", lo, hi);
        }
    }
    let empty: Vec<i64> = vec![];
    let empty_min: Option<i64> = (empty).iter().copied().min();
    let empty_max: Option<i64> = (empty).iter().copied().max();
    if let Some(empty_min) = empty_min {
        println!("ERROR: min on empty should be None");
    } else {
        println!("min([]) = None (safe!)");
    }
    if let Some(empty_max) = empty_max {
        println!("ERROR: max on empty should be None");
    } else {
        println!("max([]) = None (safe!)");
    }
    let floats: Vec<f64> = vec![3.14_f64, 1.0_f64, 2.71_f64, 0.5_f64];
    println!("sorted floats:");
    println!("{:?}", {
    let mut __sifr_sorted_v = (floats).iter().copied().collect::<Vec<_>>();
    __sifr_sorted_v.sort_by(f64::total_cmp);
    __sifr_sorted_v
});
    let mut stack: Vec<i64> = vec![42_i64];
    let val1: Option<i64> = stack.pop();
    let val2: Option<i64> = stack.pop();
    if let Some(val1) = val1 {
        println!("popped: {}", val1);
    }
    if let Some(val2) = val2 {
        println!("ERROR: pop on empty should be None");
    } else {
        println!("pop on empty = None (safe!)");
    }
    println!("min(3, 7) = {}", ::std::cmp::min(3_i64, 7_i64));
    println!("max(3, 7) = {}", ::std::cmp::max(3_i64, 7_i64));
    println!();
    println!("All collection operations are panic-free!");
    println!("  - list.remove(missing) -> no-op");
    println!("  - list.index(missing) -> None");
    println!("  - min/max(empty) -> None");
    println!("  - sorted(floats) -> total_cmp (NaN-safe)");
    println!("  - list.pop(empty) -> None");
}
