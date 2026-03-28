use std::collections::HashMap;

use std::collections::HashSet;

fn main() {
    let mut words: Vec<String> = vec!["core".to_string()];
    words.extend(("xy".to_string()).chars().map(|__sifr_char| __sifr_char.to_string()));
    println!("{}", format!("{:?}", words));
    let mut mapping: HashMap<String, i64> = HashMap::from([("base".to_string(), 1 as i64)]);
    mapping.extend(HashMap::from([("extra".to_string(), 2 as i64)]));
    println!("{}", format!("{}", mapping.remove(&"missing".to_string()).unwrap_or(7 as i64)));
    let mut seen: HashSet<i64> = HashSet::from([1 as i64]);
    {
    seen.extend(((vec![2 as i64, 3 as i64]).into_iter()));
    seen.extend((4 as i64..6 as i64));
    ()
};
    {
    let __other = (vec![3 as i64, 9 as i64]).into_iter().collect::<std::collections::HashSet<_>>();
    seen = seen.symmetric_difference(&__other).cloned().collect::<std::collections::HashSet<_>>();
    ()
};
    println!("{}", format!("{}", seen.contains(&(9 as i64))));
    let pair: (i64, i64, i64) = (4 as i64, 5 as i64, 4 as i64);
    println!("{}", format!("{}", {
    let mut __count = 0;
    if &pair.0 == &(4 as i64) {
        __count += 1;
    }
    if &pair.1 == &(4 as i64) {
        __count += 1;
    }
    if &pair.2 == &(4 as i64) {
        __count += 1;
    }
    __count as i64
}));
    println!("{}", ({
    let __start = {
    let __bound = 1 as i64;
    if __bound < 0 { (3 + __bound).max(0).min(3) } else { __bound.min(3) }
};
    let __stop = 3;
    let mut __result = None;
    if ((__result == None) && ((0 >= __start) && (0 < __stop))) && (&pair.0 == &(4 as i64)) {
        __result = Some(0);
    }
    if ((__result == None) && ((1 >= __start) && (1 < __stop))) && (&pair.1 == &(4 as i64)) {
        __result = Some(1);
    }
    if ((__result == None) && ((2 >= __start) && (2 < __stop))) && (&pair.2 == &(4 as i64)) {
        __result = Some(2);
    }
    __result
}).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{}", format!("{:?}", if (1 as i64) < 0 { "alpha,beta,gamma".to_string().split(&",".to_string()).map(|s| s.to_string()).collect::<Vec<String>>() } else { "alpha,beta,gamma".to_string().splitn(((1 as i64) + 1) as usize, &",".to_string()).map(|s| s.to_string()).collect::<Vec<String>>() }));
    println!("{}", if (2 as i64) < 0 { "aaaa".to_string().replace(&"a".to_string(), &"b".to_string()) } else { "aaaa".to_string().replacen(&"a".to_string(), &"b".to_string(), (2 as i64) as usize) });
}
