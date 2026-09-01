// src/main.rs
use ::sifr_runtime::SifrInt;
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
#[expect(
    clippy::approx_constant,
    reason = "generated Rust preserves this exact typed Sifr source contract"
)]
fn main() {
    let mut items: Vec<SifrInt> = vec![
        SifrInt::from_i64(10),
        SifrInt::from_i64(20),
        SifrInt::from_i64(30),
    ];
    if let Some(sifr_generated_pos) = items
        .iter()
        .position(|sifr_generated_x| sifr_generated_x.eq(&SifrInt::from_i64(99)))
    {
        items.remove(sifr_generated_pos);
    }
    println!("After removing missing 99:");
    println!("{items:?}");
    if let Some(sifr_generated_pos) = items
        .iter()
        .position(|sifr_generated_x| sifr_generated_x.eq(&SifrInt::from_i64(20)))
    {
        items.remove(sifr_generated_pos);
    }
    println!("After removing 20:");
    println!("{items:?}");
    let names: Vec<String> = vec![
        "alice".to_string(),
        "bob".to_string(),
        "charlie".to_string(),
    ];
    let pos: Option<SifrInt> = {
        let sifr_generated_len = names.len();
        let sifr_generated_start = 0usize;
        let sifr_generated_stop = sifr_generated_len;
        let mut sifr_generated_i = sifr_generated_start;
        let mut sifr_generated_result = None;
        while sifr_generated_i < sifr_generated_stop && sifr_generated_result == None {
            if let Some(sifr_generated_x) = names.get(sifr_generated_i)
                && sifr_generated_x.eq(&"bob".to_string())
            {
                sifr_generated_result = Some(SifrInt::from(sifr_generated_i));
            }
            sifr_generated_i += 1;
        }
        sifr_generated_result
    };
    if let Some(pos) = pos.clone() {
        println!("Found 'bob' at index {pos}");
    } else {
        println!("\'bob\' not found");
    }
    let missing: Option<SifrInt> = {
        let sifr_generated_len = names.len();
        let sifr_generated_start = 0usize;
        let sifr_generated_stop = sifr_generated_len;
        let mut sifr_generated_i = sifr_generated_start;
        let mut sifr_generated_result = None;
        while sifr_generated_i < sifr_generated_stop && sifr_generated_result == None {
            if let Some(sifr_generated_x) = names.get(sifr_generated_i)
                && sifr_generated_x.eq(&"dave".to_string())
            {
                sifr_generated_result = Some(SifrInt::from(sifr_generated_i));
            }
            sifr_generated_i += 1;
        }
        sifr_generated_result
    };
    if let Some(missing) = missing.clone() {
        println!("Found 'dave' at index {missing}");
    } else {
        println!("\'dave\' not found (safe: returned None)");
    }
    let nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(5),
        SifrInt::from_i64(3),
        SifrInt::from_i64(8),
        SifrInt::from_i64(1),
        SifrInt::from_i64(9),
    ];
    let lo: Option<SifrInt> = nums.iter().cloned().min();
    let hi: Option<SifrInt> = nums.iter().cloned().max();
    if let Some(lo) = lo.clone()
        && let Some(hi) = hi.clone()
    {
        println!("min={lo}, max={hi}");
    }
    let empty: Vec<SifrInt> = Vec::new();
    let empty_min: Option<SifrInt> = empty.iter().cloned().min();
    let empty_max_value_4e7fb6460174a48b: Option<SifrInt> = empty.iter().cloned().max();
    if empty_min.clone().is_some() {
        println!("ERROR: min on empty should be None");
    } else {
        println!("min([]) = None (safe!)");
    }
    if empty_max_value_4e7fb6460174a48b.clone().is_some() {
        println!("ERROR: max on empty should be None");
    } else {
        println!("max([]) = None (safe!)");
    }
    let floats: Vec<f64> = vec![3.14_f64, 1.0_f64, 2.71_f64, 0.5_f64];
    println!("sorted floats:");
    println!("{:?}", {
        let mut sifr_generated_sorted_v = floats.iter().copied().collect::<Vec<_>>();
        sifr_generated_sorted_v.sort_by(f64::total_cmp);
        sifr_generated_sorted_v
    });
    let mut stack: Vec<SifrInt> = vec![SifrInt::from_i64(42)];
    let val1: Option<SifrInt> = stack.pop();
    let val2_value_4373ff00edde01ca: Option<SifrInt> = stack.pop();
    if let Some(val1) = val1.clone() {
        println!("popped: {val1}");
    }
    if val2_value_4373ff00edde01ca.clone().is_some() {
        println!("ERROR: pop on empty should be None");
    } else {
        println!("pop on empty = None (safe!)");
    }
    println!(
        "min(3, 7) = {}",
        ::std::cmp::min(SifrInt::from_i64(3), SifrInt::from_i64(7))
    );
    println!(
        "max(3, 7) = {}",
        ::std::cmp::max(SifrInt::from_i64(3), SifrInt::from_i64(7))
    );
    println!();
    println!("All collection operations are panic-free!");
    println!("  - list.remove(missing) -> no-op");
    println!("  - list.index(missing) -> None");
    println!("  - min/max(empty) -> None");
    println!("  - sorted(floats) -> total_cmp (NaN-safe)");
    println!("  - list.pop(empty) -> None");
}
