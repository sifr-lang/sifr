fn total(data: &Vec<u8>) -> i64 {
    let mut out: i64 = 0 as i64;
    for value in data.iter().map(|__byte| *__byte as i64) {
        out += value;
    }
    return out;
}

fn main() {
    let payload: Vec<u8> = vec![(115 as i64) as u8, (105 as i64) as u8, (102 as i64) as u8, (114 as i64) as u8];
    let suffix: Vec<u8> = vec![(0 as i64) as u8, (1 as i64) as u8];
    let combined: Vec<u8> = {
    let mut __v = (payload).clone();
    __v.extend((suffix).iter().cloned());
    __v
};
    assert!((combined.len() as i64) == (6 as i64));
    let head: Option<i64> = combined.get((0 as i64) as usize).map(|__byte| *__byte as i64);
    assert!(head == Some(115 as i64));
    let window: Vec<u8> = Vec::from_iter((combined).iter().skip((1 as i64).max(0) as usize).take(((4 as i64).max(0) - (1 as i64).max(0)).max(0) as usize).cloned());
    assert!(total(&window) == (321 as i64));
    let raw: Vec<i64> = window.iter().map(|__byte| *__byte as i64).collect::<Vec<i64>>();
    assert!(raw == vec![105 as i64, 102 as i64, 114 as i64]);
}
