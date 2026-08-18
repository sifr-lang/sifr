// src/main.rs
fn safe_add_one(x: Option<i64>) -> i64 {
    let Some(x) = x else {
        return 0_i64;
    };
    x + (1_i64)
}

fn main() {
    println!("optional_arithmetic optional arithmetic soundness demo:");
    println!("{}", safe_add_one(Some(5_i64)));
    println!("{}", safe_add_one(None));
    let total: Option<i64> = Some(9_i64);
    let count: Option<i64> = Some(3_i64);
    if let Some(total) = total {
        if let Some(count) = count {
            println!("{}", ((total) as f64) / ((count) as f64));
        }
    }
    let missing_total: Option<i64> = None;
    if missing_total.is_none() {
        println!("{}", 0_i64);
    }
}
