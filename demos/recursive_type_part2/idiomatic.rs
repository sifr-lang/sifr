use std::collections::BTreeMap;

type Payload = Response;
type Response = Vec<i64>;

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
enum Node {
    Branch(Vec<Node>),
}

fn payload_size(data: &Payload) -> i64 {
    data.len() as i64
}

fn main() {
    let payload: Payload = vec![1, 2, 3];
    let _branch = Node::Branch(vec![Node::Branch(vec![])]);

    println!("{}", payload_size(&payload));
    println!("well-formed recursive aliases accepted");
}
