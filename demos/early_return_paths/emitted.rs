fn pick_value(maybe: Option<i64>) -> i64 {
    let Some(maybe) = maybe else {
        if true {
            return 0 as i64;
        } else {
            return 1 as i64;
        }
    };
    return maybe;
}

fn main() {
    println!("early_return_paths cfg integration behavior demo:");
    println!("{}", pick_value(Some(41 as i64)));
    println!("{}", pick_value(None));
}
