// src/main.rs
use ::sifr_runtime::SifrInt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Node {
    value: SifrInt,
}

impl Node {
    fn new(value: SifrInt) -> Self {
        Self { value }
    }
}

impl Node {
}

impl ::std::fmt::Display for Node {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Node(value={})", self.value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Person {
    name: String,
    age: SifrInt,
}

impl Person {
    fn new(name: String, age: SifrInt) -> Self {
        Self { name, age }
    }
}

impl Person {
}

impl ::std::fmt::Display for Person {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Person(name={}, age={})", self.name, self.age)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Item {
    priority: SifrInt,
    label: String,
}

impl Item {
    fn new(priority: SifrInt, label: String) -> Self {
        Self { priority, label }
    }
}

impl Item {
}

impl ::std::fmt::Display for Item {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Item(priority={}, label={})", self.priority, self.label)
    }
}

fn get_node_val(node: &Node) -> SifrInt {
    node.value.clone()
}

fn describe_person(p: Person) -> String {
    p.name.clone()
}

fn process(item: &Item) -> SifrInt {
    item.priority.clone()
}

fn main() {
    let n: Node = Node::new(SifrInt::from_i64(42));
    assert!((&get_node_val(&n) == &SifrInt::from_i64(42)));
    let p: Person = Person::new("Alice".to_string(), SifrInt::from_i64(30));
    assert!((describe_person(p) == "Alice"));
    let it: Item = Item::new(SifrInt::from_i64(5), "urgent".to_string());
    assert!((&process(&it) == &SifrInt::from_i64(5)));
    println!("forward_refs: ok");
}
