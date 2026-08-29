// src/main.rs
use ::sifr_runtime::SifrInt;

fn drain_queue(queue: &mut Vec<String>) -> Vec<String> {
    let mut order: Vec<String> = vec![];
    while !queue.is_empty() {
        order.push({
    let Some(__sifr_nonempty_pop_value) = ({
    let __len = queue.len();
    let __index = SifrInt::from_i64(0).normalize_index_or_len(__len);
    if __index >= __len { None } else { Some(queue.remove(__index)) }
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
