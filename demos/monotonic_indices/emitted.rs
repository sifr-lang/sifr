// src/main.rs
fn active_indices(flags: &Vec<bool>) -> Vec<i64> {
    let mut out: Vec<i64> = vec![];
    for index in 0_i64..flags.len() as i64 {
        if flags[index as usize] {
            out.push(index);
        }
    }
    out
}

fn main() {
    assert!((format!("{:?}", active_indices(&vec![true, false, true, true])) == "[0, 2, 3]"));
    assert!((format!("{:?}", active_indices(&vec![false, false])) == "[]"));
    println!("monotonic_indices: ok");
}
