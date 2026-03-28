#[derive(Debug, Clone, PartialEq)]
struct Entry {
    value: i64,
    next: Option<Box<Entry>>,
}

impl Entry {
    fn new(value: i64, next: Option<Box<Entry>>) -> Self {
        return Self {
            value: value,
            next: next,
        };
    }
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
    return format!("{:?}", values);
}

fn main() {
    let short: Entry = Entry::new(1 as i64, Some(Box::new(Entry::new(2 as i64, None))));
    let long: Entry = Entry::new(
        4 as i64,
        Some(Box::new(Entry::new(5 as i64, Entry::new(6 as i64, None)))),
    );
    assert!(collect_values(&None) == "[]".to_string());
    assert!(collect_values(&Some(short)) == "[1, 2]".to_string());
    assert!(collect_values(&Some(long)) == "[4, 5, 6]".to_string());
    println!("nested_recursive_helpers: ok");
}
