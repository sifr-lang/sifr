// src/main.rs
use ::std::collections::HashMap;

use ::std::collections::HashSet;

use ::sifr_runtime::SifrInt;

use ::sifr_runtime::SifrRange;

fn main() {
    let mut words: Vec<String> = vec!["core".to_string()];
    words.extend(("xy".to_string()).chars().map(|__sifr_char| __sifr_char.to_string()));
    println!("{}", format!("{:?}", words));
    let mut mapping: HashMap<String, SifrInt> = HashMap::from([("base".to_string(), SifrInt::from_i64(1))]);
    mapping.extend(HashMap::from([("extra".to_string(), SifrInt::from_i64(2))]));
    println!("{}", format!("{}", mapping.remove(&"missing".to_string()).unwrap_or(SifrInt::from_i64(7))));
    let mut seen: HashSet<SifrInt> = HashSet::from([SifrInt::from_i64(1)]);
    {
    seen.extend(((vec![SifrInt::from_i64(2), SifrInt::from_i64(3)]).into_iter()));
    seen.extend((SifrRange::new_known_nonzero(SifrInt::from_i64(4), SifrInt::from_i64(6), SifrInt::from_i64(1))));
    ()
};
    {
    let __other = (vec![SifrInt::from_i64(3), SifrInt::from_i64(9)]).into_iter().collect::<std::collections::HashSet<_>>();
    seen = seen.symmetric_difference(&__other).cloned().collect::<std::collections::HashSet<_>>();
    ()
};
    println!("{}", format!("{}", seen.contains(&SifrInt::from_i64(9))));
    let pair: (SifrInt, SifrInt, SifrInt) = (SifrInt::from_i64(4), SifrInt::from_i64(5), SifrInt::from_i64(4));
    println!("{}", format!("{}", {
    let mut __count = 0;
    if &pair.0 == &SifrInt::from_i64(4) {
        __count += 1;
    }
    if &pair.1 == &SifrInt::from_i64(4) {
        __count += 1;
    }
    if &pair.2 == &SifrInt::from_i64(4) {
        __count += 1;
    }
    SifrInt::from(__count)
}));
    println!("{}", ({
    let __start = SifrInt::from_i64(1).clamp_slice_bound(3usize);
    let __stop = 3usize;
    let mut __result = None;
    if ((__result == None) && ((0usize >= __start) && (0usize < __stop))) && (&pair.0 == &SifrInt::from_i64(4)) {
        __result = Some(SifrInt::from(0usize));
    }
    if ((__result == None) && ((1usize >= __start) && (1usize < __stop))) && (&pair.1 == &SifrInt::from_i64(4)) {
        __result = Some(SifrInt::from(1usize));
    }
    if ((__result == None) && ((2usize >= __start) && (2usize < __stop))) && (&pair.2 == &SifrInt::from_i64(4)) {
        __result = Some(SifrInt::from(2usize));
    }
    __result
}).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{}", format!("{:?}", if &SifrInt::from_i64(1) < &0 { "alpha,beta,gamma".to_string().split(',').map(|s| s.to_string()).collect::<Vec<String>>() } else { "alpha,beta,gamma".to_string().splitn(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(1) + 1)), ',').map(|s| s.to_string()).collect::<Vec<String>>() }));
    println!("{}", if &SifrInt::from_i64(2) < &0 { "aaaa".to_string().replace('a', "b") } else { "aaaa".to_string().replacen('a', "b", ::sifr_runtime::to_usize_proven(&SifrInt::from_i64(2))) });
}
