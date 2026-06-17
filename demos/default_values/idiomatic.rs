use std::collections::BTreeMap;

struct Payload {
    values: Vec<i64>,
    #[allow(dead_code)]
    counts: BTreeMap<String, i64>,
}

impl Default for Payload {
    fn default() -> Self {
        Self {
            values: vec![1, 2],
            counts: BTreeMap::from([("ok".to_string(), 1)]),
        }
    }
}

fn append_default(mut items: Vec<i64>) -> Vec<i64> {
    items.push(9);
    items
}

fn main() {
    let first = append_default(vec![1]);
    let second = append_default(vec![1]);
    let payload = Payload::default();

    println!("default_values defaults and panic-to-diagnostic conversion demo:");
    println!("{first:?}");
    println!("{second:?}");
    println!("{:?}", payload.values);
}
