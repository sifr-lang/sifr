use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
struct Payload {
    values: Vec<i64>,
    counts: HashMap<String, i64>,
}

impl Payload {
    fn new(values: Vec<i64>, counts: HashMap<String, i64>) -> Self {
        return Self { values: values, counts: counts };
    }
}

fn append_default(mut items: Vec<i64>) -> Vec<i64> {
    items.push(9 as i64);
    return items;
}

fn main() {
    let first: Vec<i64> = append_default(vec![1 as i64]);
    let second: Vec<i64> = append_default(vec![1 as i64]);
    let payload: Payload = Payload::new(vec![1 as i64, 2 as i64], HashMap::from([("ok".to_string(), 1 as i64)]));
    println!("default_values defaults and panic-to-diagnostic conversion demo:");
    println!("{:?}", first);
    println!("{:?}", second);
    println!("{:?}", payload.values);
}
