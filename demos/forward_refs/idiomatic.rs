#[derive(Debug, Clone, PartialEq, Eq)]
struct Node {
    value: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Person {
    name: String,
    age: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Item {
    priority: i64,
    label: String,
}

fn get_node_val(node: &Node) -> i64 {
    node.value
}

fn describe_person(person: Person) -> String {
    person.name
}

fn process(item: &Item) -> i64 {
    item.priority
}

fn main() {
    let node = Node { value: 42 };
    assert_eq!(get_node_val(&node), 42);

    let person = Person {
        name: "Alice".to_string(),
        age: 30,
    };
    assert_eq!(describe_person(person), "Alice");

    let item = Item {
        priority: 5,
        label: "urgent".to_string(),
    };
    assert_eq!(process(&item), 5);

    println!("forward_refs: ok");
}
