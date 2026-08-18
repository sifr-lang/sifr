// src/main.rs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Node {
    value: i64,
}

impl Node {
    fn new(value: i64) -> Self {
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
    age: i64,
}

impl Person {
    fn new(name: String, age: i64) -> Self {
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
    priority: i64,
    label: String,
}

impl Item {
    fn new(priority: i64, label: String) -> Self {
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

fn get_node_val(node: &Node) -> i64 {
    node.value
}

fn describe_person(p: Person) -> String {
    p.name
}

fn process(item: &Item) -> i64 {
    item.priority
}

fn main() {
    let n: Node = Node::new(42_i64);
    assert!((get_node_val(&n) == (42_i64)));
    let p: Person = Person::new("Alice".to_string(), 30_i64);
    assert!((describe_person(p) == "Alice"));
    let it: Item = Item::new(5_i64, "urgent".to_string());
    assert!((process(&it) == (5_i64)));
    println!("forward_refs: ok");
}
