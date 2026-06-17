fn pick_value(maybe: Option<i64>) -> i64 {
    let Some(maybe) = maybe else {
        return 0;
    };
    maybe
}

fn main() {
    println!("early_return_paths cfg integration behavior demo:");
    println!("{}", pick_value(Some(41)));
    println!("{}", pick_value(None));
}
