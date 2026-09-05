use std::collections::BTreeMap;

#[allow(dead_code)]
enum Json {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    List(Vec<Json>),
    Object(BTreeMap<String, Json>),
}

#[allow(dead_code)]
enum Node<T> {
    Value(T),
    List(Vec<Node<T>>),
}

fn main() {
    let _node: Node<i64> = Node::List(vec![Node::Value(1), Node::List(vec![Node::Value(2)])]);

    println!("recursive type representation preserved");
}
