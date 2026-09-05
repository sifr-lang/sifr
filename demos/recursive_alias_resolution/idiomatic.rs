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

fn payload_size(data: &Payload) -> i64 {
    data.len() as i64
}

fn main() {
    let payload: Payload = vec![1, 2, 3];
    let _json: Json = Json::List(vec![Json::Int(1), Json::Object(BTreeMap::new())]);

    println!("{}", payload_size(&payload));
    println!("recursive alias names resolved");
}
