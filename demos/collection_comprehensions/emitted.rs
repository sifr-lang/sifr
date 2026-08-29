// src/main.rs
use ::std::collections::HashMap;

use ::std::collections::HashSet;

use ::sifr_runtime::SifrInt;

use ::sifr_runtime::SifrRange;

fn main() {
    let squares: Vec<SifrInt> = {
    let mut __sifr_list_comp = vec![];
    for x in SifrRange::new_known_nonzero(SifrInt::from_i64(0), SifrInt::from_i64(6), SifrInt::from_i64(1)) {
        __sifr_list_comp.push(&x * &x);
    }
    __sifr_list_comp
};
    println!("{}", SifrInt::from(squares.len()));
    let square_map: HashMap<SifrInt, SifrInt> = {
    let mut __sifr_dict_comp = HashMap::new();
    for x in SifrRange::new_known_nonzero(SifrInt::from_i64(0), SifrInt::from_i64(4), SifrInt::from_i64(1)) {
        __sifr_dict_comp.insert(x, &x * &x);
    }
    __sifr_dict_comp
};
    println!("{}", SifrInt::from(square_map.len()));
    let unique_mods: HashSet<SifrInt> = {
    let mut __sifr_set_comp = HashSet::new();
    for x in SifrRange::new_known_nonzero(SifrInt::from_i64(0), SifrInt::from_i64(10), SifrInt::from_i64(1)) {
        __sifr_set_comp.insert(x.floor_mod_known_nonzero(&SifrInt::from_i64(3)));
    }
    __sifr_set_comp
};
    println!("{}", SifrInt::from(unique_mods.len()));
    let pairs: Vec<(String, SifrInt)> = vec![("alice".to_string(), SifrInt::from_i64(95)), ("bob".to_string(), SifrInt::from_i64(87))];
    for (name, score) in pairs.iter().cloned() {
        println!("{}", name);
    }
}
