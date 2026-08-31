// src/main.rs
use ::sifr_runtime::SifrInt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Entry {
    value: SifrInt,
    next: Option<Box<Entry>>,
}

impl Entry {
    fn new(value: SifrInt, next: Option<Box<Entry>>) -> Self {
        let __sifr_field_init_0: SifrInt = value.clone();
        let __sifr_field_init_1: Option<Box<Entry>> = next;
        Self { value: __sifr_field_init_0, next: __sifr_field_init_1 }
    }
}

impl Entry {
}

fn collect_values(root: Option<&Entry>) -> String {
    let mut values: Vec<SifrInt> = vec![];
    fn visit(node: Option<&Entry>, values: &mut Vec<SifrInt>) {
        let Some(node) = node.as_ref() else {
            return;
        };
        values.push(node.value.clone());
        visit((node.next).as_deref(), values);
    }
    visit(root, &mut values);
    format!("{:?}", values)
}

fn main() {
    let short: Entry = Entry::new(SifrInt::from_i64(1), Some(Box::new(Entry::new(SifrInt::from_i64(2), None))));
    let long: Entry = Entry::new(SifrInt::from_i64(4), Some(Box::new(Entry::new(SifrInt::from_i64(5), Some(Box::new(Entry::new(SifrInt::from_i64(6), None)))))));
    assert!((collect_values(None) == "[]"));
    assert!((collect_values(Some(&short)) == "[1, 2]"));
    assert!((collect_values(Some(&long)) == "[4, 5, 6]"));
    println!("nested_recursive_helpers: ok");
}
