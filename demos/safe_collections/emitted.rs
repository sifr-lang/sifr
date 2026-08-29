// src/main.rs
use ::sifr_runtime::SifrInt;

fn main() {
    let mut items: Vec<SifrInt> = vec![SifrInt::from_i64(10), SifrInt::from_i64(20), SifrInt::from_i64(30)];
    {
    if let Some(__pos) = items.iter().position(|__x| __x.eq(&SifrInt::from_i64(99))) {
        items.remove(__pos);
    }
};
    println!("After removing missing 99:");
    println!("{:?}", items);
    {
    if let Some(__pos) = items.iter().position(|__x| __x.eq(&SifrInt::from_i64(20))) {
        items.remove(__pos);
    }
};
    println!("After removing 20:");
    println!("{:?}", items);
    let names: Vec<String> = vec!["alice".to_string(), "bob".to_string(), "charlie".to_string()];
    let pos: Option<SifrInt> = {
    let __len = names.len();
    let __start = 0usize;
    let __stop = __len;
    let mut __i = __start;
    let mut __result = None;
    while (__i < __stop) && (__result == None) {
        if let Some(__x) = names.get(__i) {
            if __x.eq(&"bob".to_string()) {
                __result = Some(SifrInt::from(__i));
            }
        }
        __i += 1;
    }
    __result
};
    if let Some(pos) = pos.clone() {
        println!("Found \'bob\' at index {}", pos);
    } else {
        println!("\'bob\' not found");
    }
    let missing: Option<SifrInt> = {
    let __len = names.len();
    let __start = 0usize;
    let __stop = __len;
    let mut __i = __start;
    let mut __result = None;
    while (__i < __stop) && (__result == None) {
        if let Some(__x) = names.get(__i) {
            if __x.eq(&"dave".to_string()) {
                __result = Some(SifrInt::from(__i));
            }
        }
        __i += 1;
    }
    __result
};
    if let Some(missing) = missing.clone() {
        println!("Found \'dave\' at index {}", missing);
    } else {
        println!("\'dave\' not found (safe: returned None)");
    }
    let nums: Vec<SifrInt> = vec![SifrInt::from_i64(5), SifrInt::from_i64(3), SifrInt::from_i64(8), SifrInt::from_i64(1), SifrInt::from_i64(9)];
    let lo: Option<SifrInt> = (nums).iter().cloned().min();
    let hi: Option<SifrInt> = (nums).iter().cloned().max();
    if let Some(lo) = lo.clone() {
        if let Some(hi) = hi.clone() {
            println!("min={}, max={}", lo, hi);
        }
    }
    let empty: Vec<SifrInt> = vec![];
    let empty_min: Option<SifrInt> = (empty).iter().cloned().min();
    let empty_max: Option<SifrInt> = (empty).iter().cloned().max();
    if let Some(empty_min) = empty_min.clone() {
        println!("ERROR: min on empty should be None");
    } else {
        println!("min([]) = None (safe!)");
    }
    if let Some(empty_max) = empty_max.clone() {
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
    let mut stack: Vec<SifrInt> = vec![SifrInt::from_i64(42)];
    let val1: Option<SifrInt> = stack.pop();
    let val2: Option<SifrInt> = stack.pop();
    if let Some(val1) = val1.clone() {
        println!("popped: {}", val1);
    }
    if let Some(val2) = val2.clone() {
        println!("ERROR: pop on empty should be None");
    } else {
        println!("pop on empty = None (safe!)");
    }
    println!("min(3, 7) = {}", ::std::cmp::min(SifrInt::from_i64(3), SifrInt::from_i64(7)));
    println!("max(3, 7) = {}", ::std::cmp::max(SifrInt::from_i64(3), SifrInt::from_i64(7)));
    println!();
    println!("All collection operations are panic-free!");
    println!("  - list.remove(missing) -> no-op");
    println!("  - list.index(missing) -> None");
    println!("  - min/max(empty) -> None");
    println!("  - sorted(floats) -> total_cmp (NaN-safe)");
    println!("  - list.pop(empty) -> None");
}
