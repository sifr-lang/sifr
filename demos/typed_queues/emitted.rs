// src/main.rs
fn drain_queue(queue: &mut Vec<String>) -> Vec<String> {
    let mut order: Vec<String> = vec![];
    while !queue.is_empty() {
        order.push({
    let Some(__sifr_nonempty_pop_value) = ({
    let __len = queue.len() as i64;
    let __index = {
    let __bound = 0_i64;
    if __bound < 0 { (__len + __bound).max(0).min(__len) } else { __bound.min(__len) }
};
    if (__index < 0) || (__index >= __len) { None } else { Some(queue.remove(__index as usize)) }
}) else {
        unreachable!("compiler-verified non-empty pop should return Some");
    };
    __sifr_nonempty_pop_value
});
    }
    order
}

fn main() {
    assert!((format!("{:?}", drain_queue(&mut vec!["parse".to_string(), "check".to_string(), "emit".to_string()])) == "[\"parse\", \"check\", \"emit\"]"));
    println!("typed_queues: ok");
}
