// src/main.rs
fn drain_queue(queue: &mut Vec<String>) -> Vec<String> {
    let mut order: Vec<String> = vec![];
    while !queue.is_empty() {
        order.push(queue.remove(0_usize));
    }
    order
}

fn main() {
    assert!((format!("{:?}", drain_queue(&mut vec!["parse".to_string(), "check".to_string(), "emit".to_string()])) == "[\"parse\", \"check\", \"emit\"]"));
    println!("typed_queues: ok");
}
