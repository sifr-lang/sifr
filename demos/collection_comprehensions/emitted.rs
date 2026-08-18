// src/main.rs
use ::std::collections::HashMap;

use ::std::collections::HashSet;

fn main() {
    let squares: Vec<i64> = {
    let mut __sifr_list_comp = vec![];
    for x in 0_i64..6_i64 {
        __sifr_list_comp.push(x * x);
    }
    __sifr_list_comp
};
    println!("{}", squares.len() as i64);
    let square_map: HashMap<i64, i64> = {
    let mut __sifr_dict_comp = HashMap::new();
    for x in 0_i64..4_i64 {
        __sifr_dict_comp.insert(x, x * x);
    }
    __sifr_dict_comp
};
    println!("{}", square_map.len() as i64);
    let unique_mods: HashSet<i64> = {
    let mut __sifr_set_comp = HashSet::new();
    for x in 0_i64..10_i64 {
        __sifr_set_comp.insert(x % (3_i64));
    }
    __sifr_set_comp
};
    println!("{}", unique_mods.len() as i64);
    let pairs: Vec<(String, i64)> = vec![("alice".to_string(), 95_i64), ("bob".to_string(), 87_i64)];
    for (name, score) in pairs.iter().cloned() {
        println!("{}", name);
    }
}
