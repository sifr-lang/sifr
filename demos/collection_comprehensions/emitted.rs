// src/main.rs
use ::sifr_runtime::SifrInt;
use ::sifr_runtime::SifrRange;
use ::std::collections::HashMap;
use ::std::collections::HashSet;
fn main() {
    let squares: Vec<SifrInt> = {
        let mut sifr_generated_list_comp = Vec::new();
        for x in SifrRange::new_known_nonzero(
            SifrInt::from_i64(0),
            SifrInt::from_i64(6),
            SifrInt::from_i64(1),
        ) {
            sifr_generated_list_comp.push(::std::ops::Mul::mul(&x, &x));
        }
        sifr_generated_list_comp
    };
    println!("{}", SifrInt::from(squares.len()));
    let square_map: HashMap<SifrInt, SifrInt> = {
        let mut sifr_generated_dict_comp = HashMap::new();
        for x in SifrRange::new_known_nonzero(
            SifrInt::from_i64(0),
            SifrInt::from_i64(4),
            SifrInt::from_i64(1),
        ) {
            sifr_generated_dict_comp.insert(x.clone(), ::std::ops::Mul::mul(&x, &x));
        }
        sifr_generated_dict_comp
    };
    println!("{}", SifrInt::from(square_map.len()));
    let unique_mods: HashSet<SifrInt> = {
        let mut sifr_generated_set_comp = HashSet::new();
        for x in SifrRange::new_known_nonzero(
            SifrInt::from_i64(0),
            SifrInt::from_i64(10),
            SifrInt::from_i64(1),
        ) {
            sifr_generated_set_comp.insert(x.floor_mod_known_nonzero(&SifrInt::from_i64(3)));
        }
        sifr_generated_set_comp
    };
    println!("{}", SifrInt::from(unique_mods.len()));
    let pairs: Vec<(String, SifrInt)> = vec![
        ("alice".to_string(), SifrInt::from_i64(95)),
        ("bob".to_string(), SifrInt::from_i64(87)),
    ];
    for (name, _score) in pairs.iter() {
        println!("{name}");
    }
}
