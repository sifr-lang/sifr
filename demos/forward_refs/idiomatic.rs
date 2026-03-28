#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Node {
    value: i64,
}

impl Node {
    fn new(value: i64) -> Self {
        return Self { value: value };
    }
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Node(value={})", self.value);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Person {
    name: String,
    age: i64,
}

impl Person {
    fn new(name: String, age: i64) -> Self {
        return Self {
            name: name,
            age: age,
        };
    }
}

impl std::fmt::Display for Person {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Person(name={}, age={})", self.name, self.age);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Item {
    priority: i64,
    label: String,
}

impl Item {
    fn new(priority: i64, label: String) -> Self {
        return Self {
            priority: priority,
            label: label,
        };
    }
}

impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Item(priority={}, label={})", self.priority, self.label);
    }
}

fn get_node_val(node: &Node) -> i64 {
    return node.value;
}

fn describe_person(p: Person) -> String {
    return p.name;
}

fn process(item: &Item) -> i64 {
    return item.priority;
}

fn main() {
    let n: Node = Node::new(42 as i64);
    assert!(get_node_val(&n) == (42 as i64));
    let p: Person = Person::new("Alice".to_string(), 30 as i64);
    assert!(describe_person(p) == "Alice".to_string());
    let it: Item = Item::new(5 as i64, "urgent".to_string());
    assert!(process(&it) == (5 as i64));
    println!("forward_refs: ok");
}
