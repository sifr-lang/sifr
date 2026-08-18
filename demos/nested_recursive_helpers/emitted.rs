// src/main.rs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Entry {
    value: i64,
    next: Option<Box<Entry>>,
}

impl Entry {
    fn new(value: i64, next: Option<Box<Entry>>) -> Self {
        let __sifr_field_init_0: i64 = value;
        let __sifr_field_init_1: Option<Box<Entry>> = next;
        Self { value: __sifr_field_init_0, next: __sifr_field_init_1 }
    }
}

impl Entry {
}

fn collect_values(root: &Option<Entry>) -> String {
    let mut values: Vec<i64> = vec![];
    fn visit(node: &Option<Entry>, values: &mut Vec<i64>) {
        let Some(node) = node else {
            return;
        };
        values.push(node.value);
        visit(&(node.next).as_deref().cloned(), values);
    }
    visit(root, &mut values);
    format!("{:?}", values)
}

fn main() {
    let short: Entry = Entry::new(1_i64, Some(Box::new(Entry::new(2_i64, None))));
    let long: Entry = Entry::new(4_i64, Some(Box::new(Entry::new(5_i64, Some(Box::new(Entry::new(6_i64, None)))))));
    assert!((collect_values(&None) == "[]"));
    assert!((collect_values(&Some((short).clone())) == "[1, 2]"));
    assert!((collect_values(&Some((long).clone())) == "[4, 5, 6]"));
    println!("nested_recursive_helpers: ok");
}
