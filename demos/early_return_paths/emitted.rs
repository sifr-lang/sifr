// src/main.rs
fn pick_value(maybe: Option<i64>) -> i64 {
    let Some(maybe) = maybe else {
        if true {
            return 0_i64;
        } else {
            return 1_i64;
        }
    };
    maybe
}

fn main() {
    println!("early_return_paths cfg integration behavior demo:");
    println!("{}", pick_value(Some(41_i64)));
    println!("{}", pick_value(None));
}
