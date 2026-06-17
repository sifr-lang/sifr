fn json_dumps(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{}\"", escaped)
}

fn main() {
    assert_eq!(std::f64::consts::PI.floor() as i64, 3);

    let payload = json_dumps("ok");
    println!("stdlib_modules stdlib registry split demo:");
    println!("{}", payload);
}
