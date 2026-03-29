use std::collections::{BTreeMap, BTreeSet};

fn main() {
    let squares = (0_i64..6).map(|x| x * x).collect::<Vec<_>>();
    println!("{}", squares.len());

    let square_map = (0_i64..4).map(|x| (x, x * x)).collect::<BTreeMap<_, _>>();
    println!("{}", square_map.len());

    let unique_mods = (0_i64..10).map(|x| x % 3).collect::<BTreeSet<_>>();
    println!("{}", unique_mods.len());

    let pairs = [("alice", 95_i64), ("bob", 87_i64)];
    for (name, _score) in pairs {
        println!("{name}");
    }
}
