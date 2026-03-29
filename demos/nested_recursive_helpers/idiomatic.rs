#[derive(Clone)]
struct Entry {
    value: i64,
    next: Option<Box<Entry>>,
}

impl Entry {
    fn new(value: i64, next: Option<Entry>) -> Self {
        Self {
            value,
            next: next.map(Box::new),
        }
    }
}

fn collect_values(root: Option<&Entry>) -> String {
    let mut values = Vec::new();

    fn visit(node: Option<&Entry>, values: &mut Vec<i64>) {
        let Some(node) = node else {
            return;
        };
        values.push(node.value);
        visit(node.next.as_deref(), values);
    }

    visit(root, &mut values);
    format!("{values:?}")
}

fn main() {
    let short = Entry::new(1, Some(Entry::new(2, None)));
    let long = Entry::new(4, Some(Entry::new(5, Some(Entry::new(6, None)))));

    assert_eq!(collect_values(None), "[]");
    assert_eq!(collect_values(Some(&short)), "[1, 2]");
    assert_eq!(collect_values(Some(&long)), "[4, 5, 6]");
    println!("nested_recursive_helpers: ok");
}
