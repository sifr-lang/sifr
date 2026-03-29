use std::collections::VecDeque;

fn drain_queue(queue: Vec<String>) -> Vec<String> {
    let mut queue: VecDeque<String> = queue.into();
    let mut order = Vec::new();

    while let Some(item) = queue.pop_front() {
        order.push(item);
    }

    order
}

fn main() {
    assert_eq!(
        format!(
            "{:?}",
            drain_queue(vec![
                "parse".to_string(),
                "check".to_string(),
                "emit".to_string(),
            ])
        ),
        "[\"parse\", \"check\", \"emit\"]"
    );
    println!("typed_queues: ok");
}
