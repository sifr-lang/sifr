// src/main.rs
fn total(data: &Vec<u8>) -> i64 {
    let mut out: i64 = 0_i64;
    let values: Vec<i64> = data.iter().map(|__byte| *__byte as i64).collect::<Vec<i64>>();
    for value in values.iter().copied() {
        out += value;
    }
    out
}

fn main() {
    let payload: Vec<u8> = vec![(115_i64) as u8, (105_i64) as u8, (102_i64) as u8, (114_i64) as u8];
    let suffix: Vec<u8> = vec![(0_i64) as u8, (1_i64) as u8];
    let combined: Vec<u8> = {
    let mut __v = (payload).clone();
    __v.extend((suffix).iter().cloned());
    __v
};
    assert!((combined.len() as i64) == (6_i64));
    let head: Option<u8> = combined.get((0_i64) as usize).map(|__byte| *__byte as u8);
    if let Some(head) = head {
        let expected_head: u8 = 115u8;
        assert!(head == expected_head);
    } else {
        assert!(false);
    }
    let window: Vec<u8> = {
    let _slice_src = &combined;
    let _slice_len_i64 = _slice_src.len() as i64;
    let _slice_start_i64 = if (1_i64) < 0 { (_slice_len_i64 + (1_i64)).max(0) } else { (1_i64).min(_slice_len_i64) };
    let _slice_stop_i64 = if (4_i64) < 0 { (_slice_len_i64 + (4_i64)).max(0) } else { (4_i64).min(_slice_len_i64) };
    Vec::from_iter(_slice_src.iter().skip(_slice_start_i64 as usize).take((_slice_stop_i64 - _slice_start_i64).max(0) as usize).cloned())
};
    assert!((total(&window) == (321_i64)));
    let raw: Vec<i64> = window.iter().map(|__byte| *__byte as i64).collect::<Vec<i64>>();
    assert!(raw == vec![105_i64, 102_i64, 114_i64]);
}
