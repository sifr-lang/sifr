use std::collections::HashMap;

fn show<T: std::fmt::Display>(value: Option<T>) -> String {
    value.map_or_else(|| "None".to_string(), |value| value.to_string())
}

fn main() {
    let nums = vec![3, 6, 9, 12];
    println!("{}", show(nums.first().copied()));
    println!("{}", show(nums.get(99).copied()));

    let scores = HashMap::from([("x", 11), ("y", 22)]);
    println!("{}", show(scores.get("x").copied()));
    println!("{}", show(scores.get("z").copied()));

    let [a, mid @ .., b] = nums.as_slice() else {
        unreachable!("demo fixture always has four numbers");
    };
    println!("{}", a);
    println!("{:?}", mid);
    println!("{}", b);

    println!("{:?}", nums.iter().step_by(2).copied().collect::<Vec<_>>());
    println!("{}", nums.len());
    println!("clone_slice_unpacking_slice_unpack_demo: pass");
}
